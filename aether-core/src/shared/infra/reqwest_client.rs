use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Response;

use crate::{
    features::events::{ProgressBarId, ProgressService},
    shared::{
        domain::{FetchSemaphore, Request, RequestClient},
        sha1_async,
    },
};

pub struct ReqwestClient<PS: ProgressService> {
    progress_service: Arc<PS>,
    client: Arc<reqwest_middleware::ClientWithMiddleware>,
    semaphore: Arc<FetchSemaphore>,
}

impl<PS: ProgressService> ReqwestClient<PS> {
    pub fn new(
        progress_service: Arc<PS>,
        client: Arc<reqwest_middleware::ClientWithMiddleware>,
        semaphore: Arc<FetchSemaphore>,
    ) -> Self {
        Self {
            progress_service,
            client,
            semaphore,
        }
    }

    async fn fetch_chunks(
        &self,
        response: Response,
        loading_bar: (&ProgressBarId, f64),
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
                self.progress_service
                    .emit_progress(loading_bar_id, progress, None)?;
            }

            Ok(bytes.into())
        } else {
            Ok(response.bytes().await?)
        }
    }
}

#[async_trait]
impl<PS: ProgressService> RequestClient for ReqwestClient<PS> {
    async fn fetch_bytes(&self, request: Request) -> crate::Result<Bytes> {
        self.fetch_bytes_with_progress(request, None).await
    }

    async fn fetch_bytes_with_progress(
        &self,
        request: Request,
        loading_bar: Option<(&ProgressBarId, f64)>,
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
            Some(loading_bar) => self.fetch_chunks(response, loading_bar).await,
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
