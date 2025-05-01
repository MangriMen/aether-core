use serde::de::DeserializeOwned;

use crate::{
    features::events::LoadingBarId,
    shared::domain::{Request, RequestClient},
};

pub async fn fetch_json_new<RC, T>(
    client: &RC,
    request: Request,
    loading_bar: Option<(&LoadingBarId, f64)>,
) -> crate::Result<T>
where
    RC: RequestClient + ?Sized,
    T: DeserializeOwned,
{
    let bytes = client.fetch_bytes(request, loading_bar).await?;
    serde_json::from_slice(&bytes).map_err(Into::into)
}

pub async fn fetch_toml_new<RC, T>(
    client: &RC,
    request: Request,
    loading_bar: Option<(&LoadingBarId, f64)>,
) -> crate::Result<T>
where
    RC: RequestClient + ?Sized,
    T: DeserializeOwned,
{
    let response_data = client.fetch_bytes(request, loading_bar).await?;
    let toml_str = std::str::from_utf8(&response_data)
        .map_err(|_| crate::ErrorKind::NoValueFor("Response is not valid UTF-8".to_string()))?;
    toml::from_str(toml_str).map_err(Into::into)
}
