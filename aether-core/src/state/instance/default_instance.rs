use std::path::PathBuf;

use chrono::{DateTime, Utc};
use daedalus::minecraft;

use crate::state::{Java, LauncherState, MemorySettings, WindowSize};

use super::{InstanceInstallStage, ModLoader};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Instance {
    pub install_stage: InstanceInstallStage,

    pub path: String,

    pub name: String,
    pub icon_path: Option<String>,

    // Main minecraft metadata
    pub game_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<String>,

    // Runtime metadata
    pub java_path: Option<String>,
    pub extra_launch_args: Option<Vec<String>>,
    pub custom_env_vars: Option<Vec<(String, String)>>,

    pub memory: Option<MemorySettings>,
    pub force_fullscreen: Option<bool>,
    pub game_resolution: Option<WindowSize>,

    // Additional information
    pub time_played: u64,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,
}

impl Instance {
    /// Get instance full path in the filesystem
    #[tracing::instrument]
    pub async fn get_full_path(&self) -> anyhow::Result<PathBuf> {
        let state = LauncherState::get().await?;

        let profiles_dir = state.locations.instances_dir();

        let full_path = crate::utils::io::canonicalize(profiles_dir.join(self.path.clone()))?;

        Ok(full_path)
    }

    pub async fn get_java_version_from_profile(
        &self,
        version_info: &minecraft::VersionInfo,
    ) -> anyhow::Result<Option<Java>> {
        if let Some(java) = self.java_path.as_ref() {
            let java = crate::api::jre::check_jre(std::path::PathBuf::from(java))
                .await
                .ok()
                .flatten();

            if let Some(java) = java {
                return Ok(Some(java));
            }
        }

        let key = version_info
            .java_version
            .as_ref()
            .map(|it| it.major_version)
            .unwrap_or(8);

        let state = LauncherState::get().await?;

        // TODO: add get java from settings
        // let java_version = Java::get(key, &state.pool).await?;

        let java_version = Some(Java {
            path: r"C:\Program Files\Java\jdk-17\javaw.exe".to_owned(),
            major_version: 17,
            version: "17.0.10".to_owned(),
            architecture: "amd64".to_owned(),
        });

        Ok(java_version)
    }
}
