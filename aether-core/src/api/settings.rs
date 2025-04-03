use sysinfo::System;

use crate::state::{LauncherState, Settings};

pub async fn get() -> crate::Result<Settings> {
    let state = LauncherState::get().await?;

    Settings::get(&state).await
}

pub fn get_max_ram() -> u64 {
    let sys = System::new_all();

    sys.total_memory()
}
