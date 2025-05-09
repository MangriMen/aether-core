use daedalus::modded;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModLoader {
    Vanilla,
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

impl ModLoader {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Self::Vanilla => "vanilla",
            Self::Forge => "forge",
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
            Self::NeoForge => "neoforge",
        }
    }

    pub fn as_meta_str(&self) -> &'static str {
        match *self {
            Self::Vanilla => "vanilla",
            Self::Forge => "forge",
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
            Self::NeoForge => "neo",
        }
    }

    pub fn from_string(val: &str) -> Self {
        match val {
            "vanilla" => Self::Vanilla,
            "forge" => Self::Forge,
            "fabric" => Self::Fabric,
            "quilt" => Self::Quilt,
            "neoforge" => Self::NeoForge,
            _ => Self::Vanilla,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModLoaderManifest {
    pub loader: ModLoader,
    pub manifest: modded::Manifest,
}
