use std::time;

use bytes::Bytes;
use lazy_static::lazy_static;
use reqwest::Method;
use reqwest_retry::policies::ExponentialBackoff;
use serde::de::DeserializeOwned;
use tokio::sync::Semaphore;

const FETCH_ATTEMPTS: usize = 3;

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
            ExponentialBackoff::builder().build_with_max_retries(5),
        ))
        .build()
    };
}

#[tracing::instrument]
pub async fn fetch_advanced(
    method: Method,
    url: &str,
    headers: Option<reqwest::header::HeaderMap>,
    body: Option<Vec<u8>>,
    semaphore: &FetchSemaphore,
) -> anyhow::Result<Bytes> {
    let _permit = semaphore.0.acquire().await?;

    for attempt in 1..=(FETCH_ATTEMPTS + 1) {
        let mut req = REQWEST_CLIENT.request(method.clone(), url);

        if let Some(body) = body.clone() {
            req = req.body(body);
        }

        if let Some(header) = headers.clone() {
            req = req.headers(header);
        }

        let result = req.send().await;

        match result {
            Ok(res) => return Ok(res.bytes().await?),
            Err(_) if attempt <= FETCH_ATTEMPTS => continue,
            Err(err) => return Err(err.into()),
        }
    }

    unreachable!()
}

#[tracing::instrument]
pub async fn fetch_json<T>(
    method: Method,
    url: &str,
    headers: Option<reqwest::header::HeaderMap>,
    body: Option<Vec<u8>>,
    semaphore: &FetchSemaphore,
) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let result = fetch_advanced(method, url, headers, body, semaphore).await?;

    Ok(serde_json::from_slice(&result)?)
}
