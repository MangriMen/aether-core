use serde::{Deserialize, Serialize};

/// Describes the declarative capabilities of a plugin, such as supported importers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapabilities {
    /// List of supported modpack importers provided by the plugin.
    importers: Vec<Importer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Importer {
    /// Optional description of what this importer does.
    description: Option<String>,

    /// Optional icon file name or URL for the importer.
    icon: Option<String>,

    /// Unique identifier for the importer (lowercase, kebab/underscore allowed).
    id: String,

    /// Display name of the importer.
    name: String,
}
