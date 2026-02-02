use serde::{Deserialize, Serialize};

use crate::features::instance::{ContentProviderCapability, ImporterCapability, UpdaterCapability};

/// Describes the declarative capabilities of a plugin, such as supported importers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCapabilities {
    /// List of supported modpack importers provided by the plugin.
    pub importers: Vec<ImporterCapability>,
    pub updaters: Vec<UpdaterCapability>,
    pub content_providers: Vec<ContentProviderCapability>,
}
