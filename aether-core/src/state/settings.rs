#[derive(Debug)]
pub struct Settings {
    pub launcher_dir: Option<String>,
    pub metadata_dir: Option<String>,
}

/// Minecraft memory settings
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct MemorySettings {
    pub maximum: u32,
}

/// Game window size
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct WindowSize(pub u16, pub u16);
