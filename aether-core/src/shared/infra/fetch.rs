use std::{sync::Arc, time};

use bytes::Bytes;
use lazy_static::lazy_static;
use reqwest::{Method, Response};
use reqwest_retry::policies::ExponentialBackoff;
use serde::de::DeserializeOwned;
use tokio::sync::Semaphore;

use crate::{
    features::events::{emit_loading, LoadingBarId},
    shared::sha1_async,
};

const FETCH_ATTEMPTS: u32 = 5;

#[derive(Debug)]
pub struct FetchSemaphore(pub Semaphore);

lazy_static! {
    pub static ref REQWEST_CLIENT: Arc<reqwest_middleware::ClientWithMiddleware> = {
        let client = reqwest::Client::builder()
            .tcp_keepalive(Some(time::Duration::from_secs(10)))
            .build()
            .expect("Failed to build reqwest client");

        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(FETCH_ATTEMPTS);
        let retry_middleware =
            reqwest_retry::RetryTransientMiddleware::new_with_policy(retry_policy);

        let client_with_middlewares = reqwest_middleware::ClientBuilder::new(client)
            .with(retry_middleware)
            .build();

        Arc::new(client_with_middlewares)
    };
}

#[tracing::instrument(level = tracing::Level::TRACE, skip(body, semaphore))]
pub async fn fetch_advanced(
    method: Method,
    url: &str,
    headers: Option<reqwest::header::HeaderMap>,
    sha1: Option<&str>,
    body: Option<Vec<u8>>,
    semaphore: &FetchSemaphore,
    loading_bar: Option<(&LoadingBarId, f64)>,
) -> crate::Result<Bytes> {
    let _permit = semaphore.0.acquire().await?;

    let mut request = REQWEST_CLIENT.request(method, url);

    if let Some(body) = body {
        request = request.body(body);
    }

    if let Some(headers) = headers {
        request = request.headers(headers);
    }

    let response = request.send().await.map_err(|err| match err {
        reqwest_middleware::Error::Reqwest(err) => err.into(),
        reqwest_middleware::Error::Middleware(_) => {
            crate::ErrorKind::OtherError(url.to_string()).as_error()
        }
    })?;

    let bytes = match loading_bar {
        Some(loading_bar) => fetch_chunks(response, loading_bar).await,
        None => Ok(response.bytes().await?),
    }?;

    if let Some(expected_sha1) = sha1 {
        let actual_sha1 = sha1_async(bytes.clone()).await?;
        if actual_sha1 != *expected_sha1 {
            return Err(crate::ErrorKind::HashError(expected_sha1.to_string(), actual_sha1).into());
        }
    }

    Ok(bytes)
}

#[tracing::instrument(skip_all)]
pub async fn fetch_json<T: DeserializeOwned>(
    method: Method,
    url: &str,
    headers: Option<reqwest::header::HeaderMap>,
    sha1: Option<&str>,
    body: Option<Vec<u8>>,
    semaphore: &FetchSemaphore,
) -> crate::Result<T> {
    let bytes = fetch_advanced(method, url, headers, sha1, body, semaphore, None).await?;
    Ok(serde_json::from_slice(&bytes)?)
}

#[tracing::instrument(skip_all)]
pub async fn fetch_toml<T: DeserializeOwned>(
    method: Method,
    url: &str,
    headers: Option<reqwest::header::HeaderMap>,
    sha1: Option<&str>,
    body: Option<Vec<u8>>,
    semaphore: &FetchSemaphore,
) -> crate::Result<T> {
    let bytes = fetch_advanced(method, url, headers, sha1, body, semaphore, None).await?;
    let toml_str = std::str::from_utf8(&bytes)
        .map_err(|_| crate::ErrorKind::NoValueFor(format!("Invalid UTF-8 in TOML at {url}")))?;
    Ok(toml::from_str(toml_str)?)
}

pub async fn fetch_chunks(
    response: Response,
    loading_bar: (&LoadingBarId, f64),
) -> crate::Result<Bytes> {
    if let Some(total_size) = response.content_length() {
        use futures::StreamExt;

        let mut stream = response.bytes_stream();
        let mut bytes = Vec::new();

        let (loading_bar_id, total) = loading_bar;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            bytes.extend_from_slice(&chunk);

            let progress = (chunk.len() as f64 / total_size as f64) * total;
            emit_loading(loading_bar_id, progress, None).await?;
        }

        Ok(bytes.into())
    } else {
        Ok(response.bytes().await?)
    }
}
