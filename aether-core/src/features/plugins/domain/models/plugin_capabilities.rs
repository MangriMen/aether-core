use serde::{Deserialize, Serialize};

/// Describes the declarative capabilities of a plugin, such as supported importers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginCapabilities {
    /// List of supported modpack importers provided by the plugin.
    pub importers: Vec<ImporterCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImporterCapability {
    /// Unique identifier for the importer (lowercase, kebab/underscore allowed).
    pub id: String,

    /// Display name of the importer.
    pub name: String,

    /// Optional description of what this importer does.
    pub description: Option<String>,

    /// Optional icon file name or URL for the importer.
    pub icon: Option<String>,

    /// Optional field label for the importer
    pub field_label: Option<String>,
}
