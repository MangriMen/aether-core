use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Method;

use crate::features::events::ProgressBarId;

#[async_trait]
pub trait RequestClient: Send + Sync {
    async fn fetch_bytes(&self, request: Request) -> crate::Result<Bytes>;

    async fn fetch_bytes_with_progress(
        &self,
        request: Request,
        loading_bar: Option<(&ProgressBarId, f64)>,
    ) -> crate::Result<Bytes>;
}

pub struct Request {
    pub method: Method,
    pub url: String,
    pub headers: Option<reqwest::header::HeaderMap>,
    pub sha1: Option<String>,
    pub body: Option<Vec<u8>>,
}

impl Request {
    pub fn new(method: Method, url: String) -> Self {
        Self {
            method,
            url,
            headers: None,
            sha1: None,
            body: None,
        }
    }

    pub fn get(url: impl Into<String>) -> Self {
        Self::new(Method::GET, url.into())
    }

    pub fn post(url: impl Into<String>) -> Self {
        Self::new(Method::POST, url.into())
    }

    pub fn with_headers(mut self, headers: reqwest::header::HeaderMap) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn with_sha1(mut self, sha1: impl Into<String>) -> Self {
        self.sha1 = Some(sha1.into());
        self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
}
