use std::{future::Future, path::PathBuf};

use chrono::{DateTime, Utc};
use daedalus::{minecraft, modded};
use dashmap::DashMap;
use extism::{FromBytes, ToBytes};
use extism_convert::{encoding, Json};
use tokio::fs::remove_dir_all;

use crate::{
    event::emit::emit_instance,
    state::{Hooks, Java, LauncherState, MemorySettings, WindowSize},
    utils::io,
};

use super::{ContentType, InstanceInstallStage, ModLoader};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, FromBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct PackInfo {
    pub pack_type: String,
    pub pack_version: String,
    pub can_update: bool,
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

    pub pack_info: Option<PackInfo>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ToBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct InstanceFile {
    pub file_name: String,
    pub size: u64,
    pub content_type: ContentType,
    pub path: String,
    pub disabled: bool,
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

        let versions = crate::api::metadata::get_loader_versions(loader.as_meta_str()).await?;

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

    pub async fn remove(&self) -> crate::Result<()> {
        remove_dir_all(&self.path).await?;
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
        match crate::api::instance::get(id).await {
            Ok(profile) => {
                let mut profile = profile;

                action(&mut profile).await?;

                profile.save().await?;

                emit_instance(id, crate::event::InstancePayloadType::Edited).await?;

                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_contents(&self) -> crate::Result<DashMap<String, InstanceFile>> {
        let path = &self.path;

        let files = DashMap::new();

        for content_type in ContentType::iterator() {
            let folder = content_type.get_folder();
            let path = path.join(folder);

            if !path.exists() {
                continue;
            }

            for subdirectory in
                std::fs::read_dir(&path).map_err(|e| io::IOError::with_path(e, &path))?
            {
                let subdirectory = subdirectory.map_err(io::IOError::from)?.path();

                if !subdirectory.is_file() {
                    continue;
                }

                if let Some(file_name) = subdirectory.file_name().and_then(|x| x.to_str()) {
                    let file_size = subdirectory.metadata().map_err(io::IOError::from)?.len();

                    let path = format!("{folder}/{}", file_name.trim_end_matches(".disabled"));

                    files.insert(
                        path.to_string(),
                        InstanceFile {
                            file_name: file_name.to_string(),
                            content_type,
                            size: file_size,
                            disabled: file_name.ends_with(".disabled"),
                            path: format!("{folder}/{}", file_name),
                        },
                    );
                }
            }
        }

        Ok(files)
    }

    pub async fn toggle_disable_content(id: &str, content_path: &str) -> crate::Result<String> {
        let path = if content_path.ends_with(".disabled") {
            Instance::enable_content(id, content_path).await
        } else {
            Instance::disable_content(id, content_path).await
        }?;

        if let Some(path) = path {
            Ok(path)
        } else {
            unreachable!()
        }
    }

    pub async fn enable_content(id: &str, content_path: &str) -> crate::Result<Option<String>> {
        if !content_path.ends_with(".disabled") {
            return Ok(None);
        }

        let new_path = content_path.trim_end_matches(".disabled").to_string();

        let instance_path = crate::api::instance::get_dir(id).await?;
        io::rename(
            &instance_path.join(content_path),
            &instance_path.join(&new_path),
        )
        .await?;

        Ok(Some(new_path))
    }

    pub async fn disable_content(id: &str, content_path: &str) -> crate::Result<Option<String>> {
        let instance_path = crate::api::instance::get_dir(id).await?;

        if content_path.ends_with(".disabled") {
            return Ok(None);
        }

        let new_path = format!("{content_path}.disabled");

        io::rename(
            &instance_path.join(content_path),
            &instance_path.join(&new_path),
        )
        .await?;

        Ok(Some(new_path))
    }

    pub async fn enable_contents<I, D>(id: &str, content_paths: I) -> crate::Result<()>
    where
        I: IntoIterator<Item = D>,
        D: AsRef<str>,
    {
        for content_path in content_paths {
            Instance::enable_content(id, content_path.as_ref()).await?;
        }
        Ok(())
    }

    pub async fn disable_contents<I, D>(id: &str, content_paths: I) -> crate::Result<()>
    where
        I: IntoIterator<Item = D>,
        D: AsRef<str>,
    {
        for content_path in content_paths {
            Instance::disable_content(id, content_path.as_ref()).await?;
        }
        Ok(())
    }

    pub async fn remove_content(id: &str, content_path: &str) -> crate::Result<()> {
        if let Ok(path) = crate::api::instance::get_dir(id).await {
            io::remove_file(path.join(content_path)).await?;
        }

        Ok(())
    }
}
