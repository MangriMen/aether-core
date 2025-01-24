use std::{future::Future, path::PathBuf};

use chrono::{DateTime, Utc};
use daedalus::{minecraft, modded};
use tokio::fs::remove_dir_all;

use crate::{
    api::{self, instance::get},
    state::{Hooks, Java, LauncherState, MemorySettings, WindowSize},
    utils::io,
};

use super::{InstanceInstallStage, ModLoader};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct InstancePluginSettings {
    pub pre_launch: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: String,
    pub path: PathBuf,

    pub name: String,
    pub icon_path: Option<String>,

    pub install_stage: InstanceInstallStage,

    // Main minecraft metadata
    pub game_version: String,
    pub loader: ModLoader,
    pub loader_version: Option<String>,

    // Launch arguments
    pub java_path: Option<String>,
    pub extra_launch_args: Option<Vec<String>>,
    pub custom_env_vars: Option<Vec<(String, String)>>,

    // Minecraft runtime settings
    pub memory: Option<MemorySettings>,
    pub force_fullscreen: Option<bool>,
    pub game_resolution: Option<WindowSize>,

    // Additional information
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub last_played: Option<DateTime<Utc>>,

    pub time_played: u64,
    pub recent_time_played: u64,

    pub hooks: Hooks,
    pub plugin: Option<InstancePluginSettings>,
}

impl Instance {
    /// Get instance full path in the filesystem
    #[tracing::instrument]
    pub async fn get_full_path(id: &str) -> crate::Result<PathBuf> {
        let state = LauncherState::get().await?;

        let profiles_dir = state.locations.instances_dir();

        let full_path = crate::utils::io::canonicalize(profiles_dir.join(id))?;

        Ok(full_path)
    }

    pub async fn get_java_version_from_instance(
        &self,
        version_info: &minecraft::VersionInfo,
    ) -> crate::Result<Option<Java>> {
        if let Some(java) = self.java_path.as_ref() {
            let java = crate::api::jre::check_jre(std::path::PathBuf::from(java))
                .await
                .ok()
                .flatten();

            if let Some(java) = java {
                return Ok(Some(java));
            }
        }

        let compatible_version = version_info
            .java_version
            .as_ref()
            .map(|it| it.major_version)
            .unwrap_or(8);

        let state = LauncherState::get().await?;

        let java_version = Java::get(&state, compatible_version).await?;

        Ok(java_version)
    }

    pub async fn get_loader_version(
        game_version: &str,
        loader: ModLoader,
        loader_version: Option<&str>,
    ) -> crate::Result<Option<modded::LoaderVersion>> {
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

    pub async fn remove_by_path(path: &PathBuf) -> crate::Result<()> {
        remove_dir_all(path).await?;
        Ok(())
    }

    pub async fn remove(&self) -> anyhow::Result<()> {
        Instance::remove_by_path(&self.path.join("instance.json")).await?;
        Ok(())
    }

    pub async fn save_path(instance: &Instance, path: &PathBuf) -> crate::Result<()> {
        let data = serde_json::to_vec(instance)?;
        io::write_async(path, &data).await?;
        Ok(())
    }

    pub async fn save(&self) -> crate::Result<()> {
        Instance::save_path(self, &self.path.join("instance.json")).await?;
        Ok(())
    }

    pub async fn edit<Fut>(id: &str, action: impl Fn(&mut Instance) -> Fut) -> crate::Result<()>
    where
        Fut: Future<Output = crate::Result<()>>,
    {
        match get(id).await {
            Ok(profile) => {
                let mut profile = profile;

                action(&mut profile).await?;

                profile.save().await?;

                // emit_profile(path, ProfilePayloadType::Edited).await?;

                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
