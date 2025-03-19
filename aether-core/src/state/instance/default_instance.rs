use std::{
    future::Future,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use daedalus::{minecraft, modded};
use dashmap::DashMap;
use extism::{FromBytes, ToBytes};
use extism_convert::{encoding, Json};
use path_slash::PathBufExt;
use tokio::fs::remove_dir_all;

use crate::{
    event::emit::emit_instance,
    state::{Hooks, Java, LauncherState, MemorySettings, WindowSize},
    utils::{
        file::sha1_async,
        io::{self, read_async},
    },
};

use super::{
    ContentType, InstanceInstallStage, InstancePack, InstancePackFile, InstancePackIndex,
    InstancePackIndexField, ModLoader,
};

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
    pub hash: String,
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

        let mut pack_index = Instance::get_pack_index(&self.id).await?;

        let pack_index_by_path = pack_index
            .files
            .clone()
            .into_iter()
            .map(|it| (it.file.clone(), it))
            .collect::<DashMap<String, InstancePackFile>>();

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

                    let path = PathBuf::from(&folder)
                        .join(file_name.trim_end_matches(".disabled"))
                        .to_slash_lossy()
                        .to_string();

                    let hash = if let Some(pack_file) = pack_index_by_path.get(&path) {
                        pack_file.hash.clone()
                    } else {
                        let bytes = read_async(&subdirectory).await?;
                        let hash = sha1_async(bytes::Bytes::from(bytes)).await?;

                        pack_index.files.push(InstancePackFile {
                            file: path.clone(),
                            hash: hash.clone(),
                            alias: None,
                            hash_format: None,
                            metafile: Some(true),
                            preserve: None,
                        });

                        hash.clone()
                    };

                    files.insert(
                        path.clone(),
                        InstanceFile {
                            hash,
                            file_name: file_name.to_string(),
                            content_type,
                            size: file_size,
                            disabled: file_name.ends_with(".disabled"),
                            path,
                        },
                    );
                }
            }
        }

        Instance::set_pack_index(&self.id, pack_index).await?;

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

    pub async fn get_pack(id: &str) -> crate::Result<InstancePack> {
        let state = LauncherState::get().await?;

        let instance_pack_dir = state.locations.instance_pack_dir(id);

        let pack = match InstancePack::from_path(&instance_pack_dir).await {
            Ok(pack) => pack,
            Err(_) => {
                let index = InstancePackIndex {
                    hash_format: "sha1".to_owned(),
                    files: Vec::default(),
                };

                let index_bytes = toml::to_string(&index)?.into_bytes();
                let index_hash = sha1_async(bytes::Bytes::from(index_bytes)).await?;

                let pack = InstancePack {
                    index: InstancePackIndexField {
                        file: "index.toml".to_owned(),
                        hash_format: index.hash_format.clone(),
                        hash: index_hash,
                    },
                };
                pack.write_path(&instance_pack_dir).await?;
                index.write_path(&instance_pack_dir).await?;

                pack
            }
        };

        Ok(pack)
    }

    async fn get_pack_index_path(id: &str, file: &Path) -> crate::Result<PathBuf> {
        let state = LauncherState::get().await?;
        Ok(file
            .exists()
            .then_some(file.to_path_buf())
            .unwrap_or_else(|| {
                let instance_pack_dir = state.locations.instance_pack_dir(id);
                instance_pack_dir.join(file)
            }))
    }

    pub async fn get_pack_index(id: &str) -> crate::Result<InstancePackIndex> {
        let pack = Instance::get_pack(id).await?;

        let index_path = Path::new(&pack.index.file);
        let real_index_path = Self::get_pack_index_path(id, index_path).await?;

        let pack_index = InstancePackIndex::from_file(&real_index_path).await?;

        Ok(pack_index)
    }

    pub async fn set_pack(id: &str, pack: InstancePack) -> crate::Result<()> {
        let state = LauncherState::get().await?;
        let instance_pack_dir = state.locations.instance_pack_dir(id);
        pack.write_path(&instance_pack_dir).await?;
        Ok(())
    }

    pub async fn set_pack_index(id: &str, pack_index: InstancePackIndex) -> crate::Result<()> {
        let mut pack = Instance::get_pack(id).await?;

        let index_bytes = toml::to_string(&pack_index)?.into_bytes();
        let index_hash = sha1_async(bytes::Bytes::from(index_bytes)).await?;

        pack.index.hash = index_hash;

        let index_path = Path::new(&pack.index.file);
        let real_index_path = Self::get_pack_index_path(id, index_path).await?;

        Instance::set_pack(id, pack).await?;
        pack_index.write_file(&real_index_path).await?;
        Ok(())
    }
}
