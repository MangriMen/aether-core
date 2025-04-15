use crate::features::settings::{Settings, SettingsStorage};

pub async fn get_settings<S>(storage: &S) -> crate::Result<Settings>
where
    S: SettingsStorage + ?Sized,
{
    storage.get().await
}
