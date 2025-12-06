use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use dashmap::DashMap;
use log::debug;
use path_slash::PathBufExt;

use crate::{
    features::{
        instance::{ContentType, InstanceError, InstanceFile, PackEntry, PackFile, PackStorage},
        settings::LocationInfo,
    },
    shared::{read_async, sha1_async, IoError},
};

pub struct ListContentUseCase<PS: PackStorage> {
    pack_storage: Arc<PS>,
    location_info: Arc<LocationInfo>,
}

impl<PS: PackStorage> ListContentUseCase<PS> {
    pub fn new(pack_storage: Arc<PS>, location_info: Arc<LocationInfo>) -> Self {
        Self {
            pack_storage,
            location_info,
        }
    }

    pub async fn execute(
        &self,
        instance_id: String,
    ) -> Result<DashMap<String, InstanceFile>, InstanceError> {
        let instance_dir = self.location_info.instance_dir(&instance_id);

        let entries_by_path = self.get_entries_by_path(&instance_id).await?;

        let mut files = DashMap::new();
        for content_type in ContentType::iterator() {
            self.process_content_directory(
                &instance_id,
                &instance_dir,
                content_type,
                &entries_by_path,
                &mut files,
            )
            .await?
        }

        Ok(files)
    }

    async fn get_entries_by_path(
        &self,
        instance_id: &str,
    ) -> Result<DashMap<String, PackEntry>, InstanceError> {
        let metadata = self.pack_storage.get_pack(instance_id).await?;

        Ok(metadata
            .files
            .into_iter()
            .map(|entry| (entry.file.clone(), entry))
            .collect())
    }

    async fn process_content_directory(
        &self,
        instance_id: &str,
        instance_dir: &Path,
        content_type: ContentType,
        entries_by_path: &DashMap<String, PackEntry>,
        files: &mut DashMap<String, InstanceFile>,
    ) -> Result<(), InstanceError> {
        let content_dir = instance_dir.join(content_type.get_folder());

        if !content_dir.exists() {
            return Ok(());
        }

        for entry in
            std::fs::read_dir(&content_dir).map_err(|e| IoError::with_path(e, &content_dir))?
        {
            let entry_path = entry.map_err(IoError::from)?.path();

            if !entry_path.is_file() {
                continue;
            }

            if let Some(file) = self
                .process_content_file(instance_id, &entry_path, content_type, entries_by_path)
                .await?
            {
                files.insert(file.path.clone(), file);
            }
        }

        Ok(())
    }

    async fn process_content_file(
        &self,
        instance_id: &str,
        file_path: &Path,
        content_type: ContentType,
        entries_by_path: &DashMap<String, PackEntry>,
    ) -> Result<Option<InstanceFile>, InstanceError> {
        let file_name = match file_path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return Ok(None),
        };

        let file_size = file_path.metadata().map_err(IoError::from)?.len();

        let original_path = PathBuf::from(content_type.get_folder())
            .join(file_name)
            .to_slash_lossy()
            .to_string();

        let pack_file_path = original_path.trim_end_matches(".disabled").to_string();
        let non_disabled_file_name = PathBuf::from(pack_file_path.clone())
            .file_name()
            .map(|x| x.to_string_lossy().to_string())
            .unwrap_or(file_name.to_string());

        let pack_file = match entries_by_path.get(&pack_file_path) {
            Some(entry) => {
                self.pack_storage
                    .get_pack_file(instance_id, &entry.file)
                    .await?
            }
            None => {
                let pack_file = file_to_pack_file(file_path, file_name).await?;
                self.pack_storage
                    .update_pack_file(instance_id, &pack_file_path, &pack_file)
                    .await?;
                pack_file
            }
        };

        Ok(Some(InstanceFile {
            id: pack_file_path,
            name: pack_file.name,
            hash: pack_file.hash,
            file_name: non_disabled_file_name,
            content_type,
            size: file_size,
            disabled: file_name.ends_with(".disabled"),
            path: original_path,
            update: pack_file.update,
        }))
    }
}

async fn file_to_pack_file(file_path: &Path, file_name: &str) -> Result<PackFile, InstanceError> {
    let file_content = read_async(&file_path).await?;
    let hash = sha1_async(file_content).await.map_err(|error| {
        debug!("Failed to compute sha1: {error}");
        InstanceError::HashConstructError
    })?;

    Ok(PackFile {
        file_name: file_name.to_string(),
        name: None,
        hash: hash.clone(),
        download: None,
        option: None,
        side: None,
        update_provider: None,
        update: None,
    })
}
