use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::{
    features::events::LoadingBarId,
    shared::domain::{Request, RequestClient},
};

#[async_trait]
pub trait RequestClientExt: RequestClient {
    async fn fetch_json<T>(
        &self,
        request: Request,
        loading_bar: Option<(&LoadingBarId, f64)>,
    ) -> crate::Result<T>
    where
        T: DeserializeOwned,
    {
        let bytes = self.fetch_bytes(request, loading_bar).await?;
        serde_json::from_slice(&bytes).map_err(Into::into)
    }
    async fn fetch_toml<T>(
        &self,
        request: Request,
        loading_bar: Option<(&LoadingBarId, f64)>,
    ) -> crate::Result<T>
    where
        T: DeserializeOwned,
    {
        let response_data = self.fetch_bytes(request, loading_bar).await?;
        let toml_str = std::str::from_utf8(&response_data)
            .map_err(|_| crate::ErrorKind::NoValueFor("Response is not valid UTF-8".to_string()))?;
        toml::from_str(toml_str).map_err(Into::into)
    }
}

#[async_trait]
impl<T: RequestClient + Sync> RequestClientExt for T {}
