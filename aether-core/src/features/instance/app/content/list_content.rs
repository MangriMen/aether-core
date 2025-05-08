use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use dashmap::DashMap;
use path_slash::PathBufExt;

use crate::{
    features::{
        instance::{ContentType, InstanceFile, PackEntry, PackFile, PackStorage},
        settings::LocationInfo,
    },
    shared::{domain::AsyncUseCaseWithInputAndError, read_async, sha1_async, IOError},
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

    async fn get_entries_by_path(
        &self,
        instance_id: &str,
    ) -> crate::Result<DashMap<String, PackEntry>> {
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
    ) -> crate::Result<()> {
        let content_dir = instance_dir.join(content_type.get_folder());

        if !content_dir.exists() {
            return Ok(());
        }

        for entry in
            std::fs::read_dir(&content_dir).map_err(|e| IOError::with_path(e, &content_dir))?
        {
            let entry_path = entry.map_err(IOError::from)?.path();

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
    ) -> crate::Result<Option<InstanceFile>> {
        let file_name = match file_path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return Ok(None),
        };

        let file_size = file_path.metadata().map_err(IOError::from)?.len();

        let original_path = PathBuf::from(content_type.get_folder())
            .join(file_name)
            .to_slash_lossy()
            .to_string();

        let pack_file_path = original_path.trim_end_matches(".disabled").to_string();

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
            name: pack_file.name,
            hash: pack_file.hash,
            file_name: file_name.to_string(),
            content_type,
            size: file_size,
            disabled: file_name.ends_with(".disabled"),
            path: original_path,
            update: pack_file.update,
        }))
    }
}

#[async_trait]
impl<PS> AsyncUseCaseWithInputAndError for ListContentUseCase<PS>
where
    PS: PackStorage + Send + Sync,
{
    type Input = String;
    type Output = DashMap<String, InstanceFile>;
    type Error = crate::Error;

    async fn execute(&self, instance_id: Self::Input) -> Result<Self::Output, Self::Error> {
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
}

async fn file_to_pack_file(file_path: &Path, file_name: &str) -> crate::Result<PackFile> {
    let file_content = read_async(&file_path).await?;
    let hash = sha1_async(file_content).await?;

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
