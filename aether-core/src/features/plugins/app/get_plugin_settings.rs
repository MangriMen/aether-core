use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::plugins::{PluginSettings, PluginSettingsStorage},
    shared::domain::AsyncUseCaseWithInputAndError,
};

pub struct GetPluginSettingsUseCase<PSS: PluginSettingsStorage> {
    storage: Arc<PSS>,
}

impl<PSS: PluginSettingsStorage> GetPluginSettingsUseCase<PSS> {
    pub fn new(plugin_settings_storage: Arc<PSS>) -> Self {
        Self {
            storage: plugin_settings_storage,
        }
    }
}

#[async_trait]
impl<PSS> AsyncUseCaseWithInputAndError for GetPluginSettingsUseCase<PSS>
where
    PSS: PluginSettingsStorage + Send + Sync,
{
    type Input = String;
    type Output = Option<PluginSettings>;
    type Error = crate::Error;

    async fn execute(&self, plugin_id: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(self.storage.get(&plugin_id).await?)
    }
}
