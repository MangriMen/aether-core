use std::{
    collections::HashMap,
    future::Future,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use daedalus::modded;
use dashmap::DashMap;
use extism::{FromBytes, ToBytes};
use extism_convert::{encoding, Json};
use path_slash::PathBufExt;

use crate::{
    core::LauncherState,
    features::{
        events::{emit::emit_instance, InstancePayloadType},
        java::Java,
        minecraft::ModLoader,
        settings::{Hooks, MemorySettings, WindowSize},
    },
    shared::{
        read_async, read_toml_async, remove_file, rename, sha1_async, write_async,
        write_toml_async, IOError,
    },
};

use super::{
    ContentMetadata, ContentMetadataEntry, ContentMetadataFile, ContentType, InstanceInstallStage,
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
    pub name: Option<String>,
    pub file_name: String,
    pub size: u64,
    pub content_type: ContentType,
    pub path: String,
    pub disabled: bool,
    pub update: Option<HashMap<String, toml::Value>>,
}

impl Instance {
    /// Get instance full path in the filesystem
    #[tracing::instrument]
    pub async fn get_full_path(id: &str) -> crate::Result<PathBuf> {
        let state = LauncherState::get().await?;

        let profiles_dir = state.locations.instances_dir();

        let full_path = crate::shared::canonicalize(profiles_dir.join(id))?;

        Ok(full_path)
    }

    pub async fn get_java(&self) -> crate::Result<Option<Java>> {
        if let Some(java_path) = self.java_path.as_ref() {
            Ok(Some(
                crate::features::java::get_java_from_path(Path::new(java_path)).await?,
            ))
        } else {
            Ok(None)
        }
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

        let versions =
            crate::api::metadata::get_loader_version_manifest(loader.as_meta_str()).await?;

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

    pub async fn save_path(instance: &Instance, path: &PathBuf) -> crate::Result<()> {
        let data = serde_json::to_vec(instance)?;
        write_async(path, &data).await?;
        Ok(())
    }

    pub async fn save(&self) -> crate::Result<()> {
        let path = crate::api::instance::get_dir(&self.id).await?;
        Instance::save_path(self, &path.join(".metadata").join("instance.json")).await?;
        Ok(())
    }

    pub async fn edit<Fut>(id: &str, action: impl Fn(&mut Instance) -> Fut) -> crate::Result<()>
    where
        Fut: Future<Output = crate::Result<()>>,
    {
        match crate::api::instance::get(id).await {
            Ok(instance) => {
                let mut instance = instance;

                action(&mut instance).await?;

                instance.save().await?;

                emit_instance(id, InstancePayloadType::Edited).await?;

                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_contents(&self) -> crate::Result<DashMap<String, InstanceFile>> {
        let path = crate::api::instance::get_dir(&self.id).await?;

        let files = DashMap::new();

        let content_metadata = Instance::get_content_metadata(&self.id).await?;

        let metadata_entry_by_path = content_metadata
            .files
            .clone()
            .into_iter()
            .map(|it| (it.file.clone(), it))
            .collect::<DashMap<String, ContentMetadataEntry>>();

        for content_type in ContentType::iterator() {
            let folder = content_type.get_folder();
            let path = path.join(folder);

            if !path.exists() {
                continue;
            }

            for subdirectory in
                std::fs::read_dir(&path).map_err(|e| IOError::with_path(e, &path))?
            {
                let subdirectory = subdirectory.map_err(IOError::from)?.path();

                if !subdirectory.is_file() {
                    continue;
                }

                if let Some(file_name) = subdirectory.file_name().and_then(|x| x.to_str()) {
                    let file_size = subdirectory.metadata().map_err(IOError::from)?.len();

                    let original_path = PathBuf::from(&folder)
                        .join(file_name)
                        .to_slash_lossy()
                        .to_string();

                    let path = PathBuf::from(&folder)
                        .join(file_name.trim_end_matches(".disabled"))
                        .to_slash_lossy()
                        .to_string();

                    let metadata = if let Some(metadata_entry) = metadata_entry_by_path.get(&path) {
                        Self::get_content_metadata_file(&self.id, &metadata_entry.file)
                            .await
                            .ok()
                    } else {
                        None
                    };

                    let (name, hash) = if let Some(metadata) = &metadata {
                        (metadata.name.clone(), metadata.hash.to_owned())
                    } else {
                        let file_content = read_async(&subdirectory).await?;
                        let hash = sha1_async(file_content).await?;

                        let content_metadata_file = ContentMetadataFile {
                            file_name: file_name.to_string(),
                            name: None,
                            hash: hash.clone(),
                            download: None,
                            option: None,
                            side: None,
                            update_provider: None,
                            update: None,
                        };

                        Instance::update_content_metadata_file(
                            &self.id,
                            &path,
                            &content_metadata_file,
                        )
                        .await?;

                        (None, hash.clone())
                    };

                    files.insert(
                        original_path.clone(),
                        InstanceFile {
                            hash,
                            name,
                            file_name: file_name.to_string(),
                            content_type,
                            size: file_size,
                            disabled: file_name.ends_with(".disabled"),
                            path: original_path,
                            update: metadata.and_then(|it| it.update),
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
        rename(
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

        rename(
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
        Instance::remove_contents(id, [content_path]).await
    }

    pub async fn remove_contents<I, D>(id: &str, content_paths: I) -> crate::Result<()>
    where
        I: IntoIterator<Item = D>,
        D: AsRef<str>,
    {
        if let Ok(path) = crate::api::instance::get_dir(id).await {
            let content_paths: Vec<String> = content_paths
                .into_iter()
                .map(|p| p.as_ref().to_string())
                .collect();

            for content_path in &content_paths {
                remove_file(path.join(content_path)).await?;
            }

            Instance::remove_contents_from_pack(id, &content_paths).await?;
        }

        Ok(())
    }

    async fn add_contents_to_pack(
        id: &str,
        content_paths: &[String],
        content_metadata_files: &[ContentMetadataFile],
    ) -> crate::Result<()> {
        for (content_path, content_metadata_file) in
            content_paths.iter().zip(content_metadata_files.iter())
        {
            Instance::update_content_metadata_file(id, content_path, content_metadata_file).await?;
        }

        let mut content_metadata = Instance::get_content_metadata(id).await?;
        let content_metadata_entries: Vec<ContentMetadataEntry> = content_paths
            .iter()
            .map(|path| ContentMetadataEntry {
                file: path.to_owned(),
            })
            .collect();

        content_metadata
            .files
            .extend_from_slice(content_metadata_entries.as_slice());

        content_metadata
            .files
            .dedup_by_key(|entry| entry.file.to_owned());

        Instance::update_content_metadata(id, &content_metadata).await?;

        Ok(())
    }

    async fn remove_contents_from_pack(id: &str, content_paths: &[String]) -> crate::Result<()> {
        for content_path in content_paths {
            Instance::remove_content_metadata_file(id, content_path).await?;
        }

        let mut content_metadata = Instance::get_content_metadata(id).await?;
        content_metadata.files.retain(|entry| {
            !content_paths
                .iter()
                .any(|p| p == &entry.file.replace("\\", "/"))
        });

        Instance::update_content_metadata(id, &content_metadata).await?;

        Ok(())
    }

    pub async fn get_content_metadata(id: &str) -> crate::Result<ContentMetadata> {
        let state = LauncherState::get().await?;

        let content_metadata_path = state.locations.instance_content_metadata(id);
        match ContentMetadata::from_file(&content_metadata_path).await {
            Ok(index) => Ok(index),
            Err(_) => {
                let content_metadata = ContentMetadata::default();
                Instance::update_content_metadata(id, &content_metadata).await?;
                Ok(content_metadata.clone())
            }
        }
    }

    pub async fn update_content_metadata(
        id: &str,
        content_metadata: &ContentMetadata,
    ) -> crate::Result<()> {
        let state = LauncherState::get().await?;
        let content_metadata_path = state.locations.instance_content_metadata(id);
        content_metadata.write_file(&content_metadata_path).await
    }

    pub async fn get_content_metadata_file(
        id: &str,
        content_path: &str,
    ) -> crate::Result<ContentMetadataFile> {
        let state = LauncherState::get().await?;
        let instance_pack_dir = state.locations.instance_pack_dir(id);
        let content_metadata_file_path =
            instance_pack_dir.join(content_path).with_extension("toml");
        read_toml_async(&content_metadata_file_path).await
    }

    pub async fn update_content_metadata_file(
        id: &str,
        content_path: &str,
        entry: &ContentMetadataFile,
    ) -> crate::Result<()> {
        let state = LauncherState::get().await?;
        let instance_pack_dir = state.locations.instance_pack_dir(id);
        let content_metadata_file_path =
            instance_pack_dir.join(content_path).with_extension("toml");

        if !content_metadata_file_path.exists() {
            let mut content_metadata = Instance::get_content_metadata(id).await?;
            content_metadata.files.push(ContentMetadataEntry {
                file: content_path.to_string(),
            });
            Instance::update_content_metadata(id, &content_metadata).await?;
        }

        write_toml_async(&content_metadata_file_path, entry).await
    }

    pub async fn remove_content_metadata_file(id: &str, content_path: &str) -> crate::Result<()> {
        let state = LauncherState::get().await?;
        let instance_pack_dir = state.locations.instance_pack_dir(id);
        let content_metadata_file_path =
            instance_pack_dir.join(content_path).with_extension("toml");
        remove_file(&content_metadata_file_path).await?;
        Ok(())
    }

    pub async fn get_import_content_data(
        id: &str,
        path: &Path,
        content_type: ContentType,
    ) -> crate::Result<(String, ContentMetadataFile)> {
        let content_folder = content_type.get_folder();
        let content_file_name = path
            .file_name()
            .ok_or(crate::ErrorKind::NoValueFor(format!(
                "Can't get file name {:?}",
                path
            )))?
            .to_string_lossy()
            .to_string();

        let content_path = Path::new(content_folder)
            .join(&content_file_name)
            .to_string_lossy()
            .to_string();

        let duplicate_error = crate::ErrorKind::ContentImportDuplicateError {
            content_path: content_path.clone(),
        };

        let absolute_content_path = crate::api::instance::get_dir(id).await?.join(&content_path);

        if absolute_content_path.exists() {
            return Err(duplicate_error.as_error());
        }

        let content_metadata_file = Instance::get_content_metadata_file(id, &content_path).await;

        let content_bytes = bytes::Bytes::from(std::fs::read(path)?);

        let content_metadata_file = match content_metadata_file {
            Ok(_) => Err(duplicate_error.as_error()),
            Err(_) => {
                let hash = sha1_async(content_bytes).await?;

                Ok(ContentMetadataFile {
                    file_name: content_file_name.clone(),
                    name: None,
                    hash,
                    download: None,
                    option: None,
                    side: None,
                    update_provider: None,
                    update: None,
                })
            }
        }?;

        Ok((content_path, content_metadata_file))
    }

    pub async fn import_contents(
        id: &str,
        paths: Vec<&Path>,
        content_type: ContentType,
    ) -> crate::Result<()> {
        let mut content_paths = Vec::new();
        let mut content_metadata_files = Vec::new();

        for path in &paths {
            let (content_path, content_metadata_file) =
                Instance::get_import_content_data(id, Path::new(&path), content_type).await?;

            content_paths.push(content_path);
            content_metadata_files.push(content_metadata_file);
        }

        Instance::add_contents_to_pack(id, &content_paths, &content_metadata_files).await?;

        let instance_dir = crate::api::instance::get_dir(id).await?;

        for (path, content_path) in paths.iter().zip(content_paths) {
            let absolute_content_path = instance_dir.join(content_path);
            tokio::fs::copy(path, absolute_content_path).await?;
        }

        Ok(())
    }
}
