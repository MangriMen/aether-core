use std::time;

use bytes::Bytes;
use lazy_static::lazy_static;
use reqwest::Method;
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
    pub static ref REQWEST_CLIENT: reqwest_middleware::ClientWithMiddleware = {
        reqwest_middleware::ClientBuilder::new(
            reqwest::Client::builder()
                .tcp_keepalive(Some(time::Duration::from_secs(10)))
                .build()
                .expect("Reqwest Client Building Failed"),
        )
        .with(reqwest_retry::RetryTransientMiddleware::new_with_policy(
            ExponentialBackoff::builder().build_with_max_retries(FETCH_ATTEMPTS),
        ))
        .build()
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

    let mut req = REQWEST_CLIENT.request(method.clone(), url);

    if let Some(body) = body.clone() {
        req = req.body(body);
    }

    if let Some(header) = headers.clone() {
        req = req.headers(header);
    }

    let result = req.send().await;

    match result {
        Ok(res) => {
            let bytes = if let Some((bar, total)) = &loading_bar {
                let length = res.content_length();

                if let Some(total_size) = length {
                    use futures::StreamExt;

                    let mut stream = res.bytes_stream();
                    let mut bytes = Vec::new();
                    while let Some(item) = stream.next().await {
                        let chunk =
                            item.or(Err(crate::ErrorKind::NoValueFor("fetch bytes".to_string())))?;

                        bytes.append(&mut chunk.to_vec());

                        emit_loading(bar, (chunk.len() as f64 / total_size as f64) * total, None)
                            .await?;
                    }

                    Ok(bytes::Bytes::from(bytes))
                } else {
                    res.bytes().await
                }
            } else {
                res.bytes().await
            };

            match bytes {
                Ok(bytes) => {
                    if let Some(sha1) = sha1 {
                        let hash = sha1_async(bytes.clone()).await?;
                        if hash != *sha1 {
                            return Err(crate::ErrorKind::HashError(sha1.to_string(), hash).into());
                        }
                    }

                    Ok(bytes)
                }
                Err(err) => Err(err.into()),
            }
        }
        Err(err) => match err {
            reqwest_middleware::Error::Reqwest(err) => Err(err.into()),
            reqwest_middleware::Error::Middleware(_) => {
                Err(crate::ErrorKind::OtherError(url.to_string()).into())
            }
        },
    }
}

#[tracing::instrument(skip(body, semaphore))]
pub async fn fetch_json<T>(
    method: Method,
    url: &str,
    headers: Option<reqwest::header::HeaderMap>,
    sha1: Option<&str>,
    body: Option<Vec<u8>>,
    semaphore: &FetchSemaphore,
) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    let result = fetch_advanced(method, url, headers, sha1, body, semaphore, None).await?;
    Ok(serde_json::from_slice(&result)?)
}

#[tracing::instrument(skip(body, semaphore))]
pub async fn fetch_toml<T>(
    method: Method,
    url: &str,
    headers: Option<reqwest::header::HeaderMap>,
    sha1: Option<&str>,
    body: Option<Vec<u8>>,
    semaphore: &FetchSemaphore,
) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    fetch_advanced(method, url, headers, sha1, body, semaphore, None)
        .await
        .and_then(|ref it| {
            let toml_str = std::str::from_utf8(it).map_err(|_| {
                crate::ErrorKind::NoValueFor(format!("Can't fetch TOML at {:?}", url)).as_error()
            })?;
            Ok(toml::from_str(toml_str)?)
        })
}

// #[tracing::instrument]
// TODO: add fetch chunks
// pub async fn fetch_chunks
