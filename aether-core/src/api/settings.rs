use sysinfo::System;

use crate::{
    core::LauncherState,
    features::settings::{FsSettingsStorage, Settings, SettingsStorage},
};

pub async fn get() -> crate::Result<Settings> {
    let state = LauncherState::get().await?;

    FsSettingsStorage.get(&state).await
}

pub fn get_max_ram() -> u64 {
    let sys = System::new_all();

    sys.total_memory()
}
