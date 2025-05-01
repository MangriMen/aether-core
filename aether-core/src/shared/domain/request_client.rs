use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Method;

use crate::features::events::LoadingBarId;

#[async_trait]
pub trait RequestClient {
    async fn fetch_bytes(
        &self,
        request: Request,
        loading_bar: Option<(&LoadingBarId, f64)>,
    ) -> crate::Result<Bytes>;
}

pub struct Request {
    pub method: Method,
    pub url: String,
    pub headers: Option<reqwest::header::HeaderMap>,
    pub sha1: Option<String>,
    pub body: Option<Vec<u8>>,
}
