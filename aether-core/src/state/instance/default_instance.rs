use std::path::PathBuf;

use chrono::{DateTime, Utc};
use daedalus::{minecraft, modded};
use tokio::fs::remove_dir_all;

use crate::{
    api,
    state::{Java, LauncherState, MemorySettings, WindowSize},
    utils::io,
};

use super::{InstanceInstallStage, ModLoader};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
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

    pub async fn get_java_version_from_instance(
        &self,
        _version_info: &minecraft::VersionInfo,
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

        // let key = version_info
        //     .java_version
        //     .as_ref()
        //     .map(|it| it.major_version)
        //     .unwrap_or(8);

        // let state = LauncherState::get().await?;

        // TODO: add get java from settings
        // let java_version = Java::get(key, &state.pool).await?;

        let java_version = Some(Java {
            path: r"C:\Program Files\Java\jdk-17\bin\javaw.exe".to_owned(),
            major_version: 17,
            version: "17.0.10".to_owned(),
            architecture: "amd64".to_owned(),
        });

        Ok(java_version)
    }

    pub async fn get_loader_version_from_instance(
        game_version: &str,
        loader: ModLoader,
        loader_version: Option<&str>,
    ) -> anyhow::Result<Option<modded::LoaderVersion>> {
        if loader == ModLoader::Vanilla {
            return Ok(None);
        }

        let version = loader_version.unwrap_or("latest");

        let filter = |it: &modded::LoaderVersion| match version {
            "latest" => true,
            "stable" => it.stable,
            id => it.id == *id,
        };

        let versions = api::metadata::get_loader_versions(loader.as_meta_str()).await?;

        let loaders = versions.game_versions.into_iter().find(|x| {
            x.id.replace(daedalus::modded::DUMMY_REPLACE_STRING, game_version) == game_version
        });

        if let Some(loaders) = loaders {
            let loader_version =
                loaders
                    .loaders
                    .iter()
                    .find(|x| filter(x))
                    .or(if version == "stable" {
                        loaders.loaders.first()
                    } else {
                        None
                    });

            Ok(loader_version.cloned())
        } else {
            Ok(None)
        }
    }

    pub async fn remove(&self, path: &str) -> anyhow::Result<()> {
        remove_dir_all(path).await?;

        Ok(())
    }

    pub async fn save(&self, path: &str) -> anyhow::Result<()> {
        let data = serde_json::to_vec(self)?;
        io::write_async(path, &data).await?;
        Ok(())
    }
}
