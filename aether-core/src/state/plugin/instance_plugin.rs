use std::{fmt, path::PathBuf};

use async_trait::async_trait;

use crate::state::LauncherState;

#[async_trait]
pub trait InstancePlugin: Send + Sync {
    fn get_id(&self) -> String;
    fn get_name(&self) -> String;
    fn get_description(&self) -> String;

    async fn init(&self) -> crate::Result<()>;
    async fn call(&self, data: &str) -> crate::Result<()>;
    async fn unload(&self) -> crate::Result<()>;

    async fn clear_cache(&self) -> crate::Result<()>;
}

impl dyn InstancePlugin {
    pub async fn get_plugin_dir(&self) -> crate::Result<PathBuf> {
        let state = LauncherState::get().await?;
        Ok(state.locations.plugin_dir(&self.get_id()))
    }
}

impl fmt::Debug for dyn InstancePlugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "InstancePlugin(name: {}, description: {})",
            self.get_name(),
            self.get_description()
        )
    }
}
