use std::sync::Arc;

use async_trait::async_trait;
use extism_convert::Msgpack;
use tokio::sync::Mutex;

use crate::features::{
    instance::{
        ContentFile, ContentInstallParams, ContentProvider, ContentProviderCapabilityMetadata,
        ContentSearchParams, ContentSearchResult, InstanceError,
    },
    plugins::{
        PluginContentProviderCapability, PluginInstallContent, PluginInstance, PluginInstanceExt,
    },
};

pub struct PluginContentProviderProxy {
    instance: Arc<Mutex<dyn PluginInstance>>,
    capability: PluginContentProviderCapability,
}

impl PluginContentProviderProxy {
    pub fn new(
        instance: Arc<Mutex<dyn PluginInstance>>,
        capability: PluginContentProviderCapability,
    ) -> Self {
        Self {
            instance,
            capability,
        }
    }
}

#[async_trait]
impl ContentProvider for PluginContentProviderProxy {
    fn metadata(&self) -> &ContentProviderCapabilityMetadata {
        &self.capability
    }

    async fn search(
        &self,
        search_content: &ContentSearchParams,
    ) -> Result<ContentSearchResult, InstanceError> {
        let mut plugin = self.instance.lock().await;
        let plugin_id = plugin.get_id();

        if !plugin.supports(&self.capability.search_handler) {
            tracing::error!(
                "Plugin '{}' promised handler '{}' for capability '{}', but function not found",
                plugin_id.clone(),
                self.capability.search_handler,
                self.capability.id
            );

            return Err(InstanceError::ContentProviderNotFound {
                provider_id: self.capability.id.clone(),
            });
        }

        Ok(plugin
            .call::<Msgpack<ContentSearchParams>, Msgpack<ContentSearchResult>>(
                &self.capability.search_handler,
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
        let mut plugin = self.instance.lock().await;
        let plugin_id = plugin.get_id();

        if !plugin.supports(&self.capability.install_handler) {
            tracing::error!(
                "Plugin '{}' promised handler '{}' for capability '{}', but function not found",
                plugin_id.clone(),
                self.capability.install_handler,
                self.capability.id
            );

            return Err(InstanceError::ContentProviderNotFound {
                provider_id: self.capability.id.clone(),
            });
        }

        Ok(plugin
            .call::<PluginInstallContent, Msgpack<ContentFile>>(
                &self.capability.install_handler,
                PluginInstallContent {
                    instance_id: instance_id.to_string(),
                    install_params: install_params.clone(),
                },
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
}
