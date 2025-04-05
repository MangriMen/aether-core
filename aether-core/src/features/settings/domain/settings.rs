use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub launcher_dir: Option<String>,
    pub metadata_dir: Option<String>,

    pub max_concurrent_downloads: usize,

    pub memory: MemorySettings,

    pub game_resolution: WindowSize,
    pub custom_env_vars: Vec<(String, String)>,
    pub extra_launch_args: Vec<String>,

    pub hooks: Hooks,

    pub enabled_plugins: HashSet<String>,
}

/// Memory usage settings for Java.
///
/// Used to define the maximum amount of memory that can be allocated
/// to the JVM when launching a game.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct MemorySettings {
    /// Maximum amount of RAM in megabytes.
    ///
    /// Typically corresponds to the `-Xmx` parameter when starting the JVM.
    pub maximum: u32,
}

/// A 2D size, represented by a tuple of two integers
///
/// First is the width, second is the height of the window (width, height)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct WindowSize(pub u16, pub u16);

/// A struct representing various hooks that can be configured for specific actions
/// in the application lifecycle.
///
/// # Fields
/// - `pre_launch`: An optional string representing a command or script to be executed
///   before the game launches.
/// - `wrapper`: An optional string representing a wrapper command or script to be used
///   during the game's execution.
/// - `post_exit`: An optional string representing a command or script to be executed
///   after the game exits.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Hooks {
    pub pre_launch: Option<String>,
    pub wrapper: Option<String>,
    pub post_exit: Option<String>,
}
