use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Response;

use crate::{
    features::events::{emit_loading, LoadingBarId},
    shared::{
        domain::{Request, RequestClient},
        sha1_async,
    },
};

use super::FetchSemaphore;

pub struct ReqwestClient {
    client: Arc<reqwest_middleware::ClientWithMiddleware>,
    semaphore: Arc<FetchSemaphore>,
}

impl ReqwestClient {
    pub fn new(
        client: Arc<reqwest_middleware::ClientWithMiddleware>,
        semaphore: Arc<FetchSemaphore>,
    ) -> Self {
        Self { client, semaphore }
    }

    async fn fetch_chunks(
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
}

#[async_trait]
impl RequestClient for ReqwestClient {
    async fn fetch_bytes(
        &self,
        request: Request,
        loading_bar: Option<(&LoadingBarId, f64)>,
    ) -> crate::Result<Bytes> {
        let Request {
            method,
            url,
            headers,
            sha1,
            body,
        } = request;

        let _permit = self.semaphore.0.acquire().await?;

        let mut request = self.client.request(method, &url);

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
            Some(loading_bar) => Self::fetch_chunks(response, loading_bar).await,
            None => Ok(response.bytes().await?),
        }?;

        if let Some(expected_sha1) = sha1 {
            let actual_sha1 = sha1_async(bytes.clone()).await?;
            if actual_sha1 != *expected_sha1 {
                return Err(
                    crate::ErrorKind::HashError(expected_sha1.to_string(), actual_sha1).into(),
                );
            }
        }

        Ok(bytes)
    }
}
