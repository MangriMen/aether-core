use std::{future::Future, path::PathBuf};

use chrono::{DateTime, Utc};
use daedalus::{minecraft, modded};
use dashmap::DashMap;
use tokio::fs::remove_dir_all;

use crate::{
    event::emit::emit_instance,
    state::{Hooks, Java, LauncherState, MemorySettings, WindowSize},
    utils::{fetch::FetchSemaphore, io},
};

use super::{ContentType, InstanceInstallStage, ModLoader};

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

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct FileMetadata {
    pub project_id: String,
    pub version_id: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct InstanceFile {
    pub hash: String,
    pub file_name: String,
    pub size: u64,
    pub metadata: Option<FileMetadata>,
    pub update_version_id: Option<String>,
    pub content_type: ContentType,
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

    pub async fn get_contents(
        &self,
        _fetch_semaphore: &FetchSemaphore,
    ) -> crate::Result<DashMap<String, InstanceFile>> {
        let path = crate::api::instance::get_dir(&self.id).await?;

        struct InitialScanFile {
            path: String,
            file_name: String,
            content_type: ContentType,
            size: u64,
            cache_key: String,
        }

        let mut keys = vec![];

        for content_type in ContentType::iterator() {
            let folder = content_type.get_folder();
            let path = path.join(folder);

            if !path.exists() {
                continue;
            }

            for sub_directory in
                std::fs::read_dir(&path).map_err(|e| io::IOError::with_path(e, &path))?
            {
                let sub_directory = sub_directory.map_err(io::IOError::from)?.path();

                if !sub_directory.is_file() {
                    continue;
                }

                if let Some(file_name) = sub_directory.file_name().and_then(|x| x.to_str()) {
                    let file_size = sub_directory.metadata().map_err(io::IOError::from)?.len();

                    keys.push(InitialScanFile {
                        path: format!(
                            "{}/{folder}/{}",
                            self.id,
                            file_name.trim_end_matches(".disabled")
                        ),
                        file_name: file_name.to_string(),
                        content_type,
                        size: file_size,
                        cache_key: format!("{file_size}-{}/{folder}/{file_name}", self.id),
                    });
                }
            }
        }

        // let file_hashes = CachedEntry::get_file_hash_many(
        //     &keys.iter().map(|s| &*s.cache_key).collect::<Vec<_>>(),
        //     None,
        //     pool,
        //     fetch_semaphore,
        // )
        // .await?;

        // let file_updates = file_hashes
        //     .iter()
        //     .map(|x| Self::get_cache_key(x, self))
        //     .collect::<Vec<_>>();

        // let file_hashes_ref = file_hashes.iter().map(|x| &*x.hash).collect::<Vec<_>>();
        // let file_updates_ref = file_updates.iter().map(|x| &**x).collect::<Vec<_>>();
        // let (mut file_info, file_updates) = tokio::try_join!(
        //     CachedEntry::get_file_many(&file_hashes_ref, cache_behaviour, pool, fetch_semaphore,),
        //     CachedEntry::get_file_update_many(
        //         &file_updates_ref,
        //         cache_behaviour,
        //         pool,
        //         fetch_semaphore,
        //     )
        // )?;

        let files = DashMap::new();

        // for hash in file_hashes {
        //     let info_index = file_info.iter().position(|x| x.hash == hash.hash);
        //     let file = info_index.map(|x| file_info.remove(x));

        //     if let Some(initial_file_index) = keys
        //         .iter()
        //         .position(|x| x.path == hash.path.trim_end_matches(".disabled"))
        //     {
        //         let initial_file = keys.remove(initial_file_index);

        //         let path = format!(
        //             "{}/{}",
        //             initial_file.project_type.get_folder(),
        //             initial_file.file_name
        //         );

        //         let update_version_id = if let Some(update) = file_updates
        //             .iter()
        //             .find(|x| x.hash == hash.hash)
        //             .map(|x| x.update_version_id.clone())
        //         {
        //             if let Some(metadata) = &file {
        //                 if metadata.version_id != update {
        //                     Some(update)
        //                 } else {
        //                     None
        //                 }
        //             } else {
        //                 None
        //             }
        //         } else {
        //             None
        //         };

        //         let file = InstanceFile {
        //             update_version_id,
        //             hash: hash.hash,
        //             file_name: initial_file.file_name,
        //             size: initial_file.size,
        //             metadata: file.map(|x| FileMetadata {
        //                 project_id: x.project_id,
        //                 version_id: x.version_id,
        //             }),
        //             project_type: initial_file.project_type,
        //         };
        //         files.insert(path, file);
        //     }
        // }

        Ok(files)
    }

    #[tracing::instrument]
    pub async fn remove_content(instance_id: &str, content_path: &str) -> crate::Result<()> {
        if let Ok(path) = crate::api::instance::get_dir(instance_id).await {
            io::remove_file(path.join(content_path)).await?;
        }

        Ok(())
    }
}
