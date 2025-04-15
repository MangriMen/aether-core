use crate::features::settings::{Settings, SettingsStorage};

pub async fn upsert_settings<S>(storage: &S, settings: &Settings) -> crate::Result<()>
where
    S: SettingsStorage + ?Sized,
{
    storage.upsert(settings).await
}
