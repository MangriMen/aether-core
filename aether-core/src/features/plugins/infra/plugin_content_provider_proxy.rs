use std::sync::Arc;

use async_trait::async_trait;
use extism_convert::Msgpack;
use tokio::sync::Mutex;

use crate::features::{
    instance::{
        ContentFile, ContentInstallParams, ContentProvider, ContentProviderCapability,
        ContentSearchParams, ContentSearchResult, InstanceError,
    },
    plugins::{PluginInstance, PluginInstanceExt},
};

pub struct PluginContentProviderProxy {
    instance: Arc<Mutex<dyn PluginInstance>>,
    capability: ContentProviderCapability,
}

impl PluginContentProviderProxy {
    pub fn new(
        instance: Arc<Mutex<dyn PluginInstance>>,
        capability: ContentProviderCapability,
    ) -> Self {
        Self {
            instance,
            capability,
        }
    }
}

#[async_trait]
impl ContentProvider for PluginContentProviderProxy {
    fn info(&self) -> &ContentProviderCapability {
        &self.capability
    }

    fn get_name(&self) -> String {
        self.capability.name.to_owned()
    }

    async fn search(
        &self,
        search_content: &ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError> {
        let mut plugin = self.instance.lock().await;
        let plugin_id = plugin.get_id();

        if !plugin.supports(&self.capability.handler) {
            tracing::error!(
                "Plugin '{}' promised handler '{}' for capability '{}', but function not found",
                plugin_id.clone(),
                self.capability.handler,
                self.capability.id
            );

            return Err(InstanceError::ContentProviderNotFound {
                provider_id: self.capability.id.clone(),
            });
        }

        Ok(plugin
            .call::<Msgpack<ContentSearchParams>, Msgpack<ContentSearchResult>>(
                &self.capability.handler,
                Msgpack(search_content.clone()),
            )
            .map_err(|err| {
                tracing::error!(
                    "Error importing instance by plugin '{}': {:?}",
                    plugin_id,
                    err
                );

                InstanceError::ContentProviderNotFound {
                    provider_id: self.capability.id.clone(),
                }
            })?
            .0)
    }

    async fn install(
        &self,
        instance_id: &str,
        install_params: &ContentInstallParams,
    ) -> Result<ContentFile, InstanceError> {
        todo!("Make install work in plugin")
    }

    fn get_update_data_id_field(&self) -> String {
        todo!("Make update data id field work in plugin")
    }
}
