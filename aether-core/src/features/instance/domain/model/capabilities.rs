use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseCapability {
    /// Identifier (lowercase, kebab/underscore allowed).
    pub id: String,

    /// Display name.
    pub name: String,

    /// Optional description.
    pub description: Option<String>,

    /// Optional icon file name or URL.
    pub icon: Option<String>,

    /// Plugin function name to handle this capability call.
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImporterCapability {
    #[serde(flatten)]
    pub base: BaseCapability,

    /// Optional field label for the importer.
    pub field_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdaterCapability {
    #[serde(flatten)]
    pub base: BaseCapability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentProviderCapability {
    #[serde(flatten)]
    pub base: BaseCapability,
}

impl Deref for ImporterCapability {
    type Target = BaseCapability;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl Deref for UpdaterCapability {
    type Target = BaseCapability;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl Deref for ContentProviderCapability {
    type Target = BaseCapability;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
