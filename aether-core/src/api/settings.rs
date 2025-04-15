use crate::{
    core::LauncherState,
    features::settings::{self, FsSettingsStorage, Settings},
};

async fn get_storage() -> crate::Result<FsSettingsStorage> {
    Ok(FsSettingsStorage::new(
        &LauncherState::get().await?.locations.settings_dir,
    ))
}

pub async fn get() -> crate::Result<Settings> {
    settings::get_settings(&get_storage().await?).await
}

pub async fn upsert(settings: &Settings) -> crate::Result<()> {
    settings::upsert_settings(&get_storage().await?, settings).await
}

pub fn get_max_ram() -> u64 {
    settings::get_max_ram()
}
