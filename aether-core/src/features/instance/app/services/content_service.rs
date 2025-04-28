use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use dashmap::DashMap;
use path_slash::PathBufExt;

use crate::{
    features::{
        events::{emit_instance, InstancePayloadType},
        instance::{ContentType, InstanceFile, PackEntry, PackFile, PackStorage},
        settings::LocationInfo,
    },
    shared::{read_async, remove_file, rename, sha1_async, IOError},
};

pub struct ContentService<PS>
where
    PS: PackStorage,
{
    pack_storage: PS,
    location_info: Arc<LocationInfo>,
}

impl<PS> ContentService<PS>
where
    PS: PackStorage + Send + Sync,
{
    pub fn new(pack_storage: PS, location_info: Arc<LocationInfo>) -> Self {
        Self {
            pack_storage,
            location_info,
        }
    }

    pub async fn import_many(
        &self,
        instance_id: &str,
        content_type: ContentType,
        source_paths: &[&Path],
    ) -> crate::Result<()> {
        let (content_paths, pack_files) = self
            .prepare_import_data(instance_id, content_type, source_paths)
            .await?;

        self.copy_import_files(instance_id, source_paths, &content_paths)
            .await?;

        self.pack_storage
            .update_pack_file_many(instance_id, &content_paths, &pack_files)
            .await?;

        self.notify_instance_updated(instance_id).await?;

        Ok(())
    }

    pub async fn list(&self, instance_id: &str) -> crate::Result<DashMap<String, InstanceFile>> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        let entries_by_path = self.get_entries_by_path(instance_id).await?;

        let mut files = DashMap::new();
        for content_type in ContentType::iterator() {
            self.process_content_directory(
                instance_id,
                &instance_dir,
                content_type,
                &entries_by_path,
                &mut files,
            )
            .await?
        }

        Ok(files)
    }

    pub async fn remove(&self, instance_id: &str, content_path: &str) -> crate::Result<()> {
        self.remove_many(instance_id, &[content_path.to_string()])
            .await
    }

    pub async fn remove_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> crate::Result<()> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        for content_path in content_paths {
            remove_file(instance_dir.join(content_path)).await?;
        }

        self.pack_storage
            .remove_pack_file_many(instance_id, content_paths)
            .await?;

        self.notify_instance_updated(instance_id).await?;
        Ok(())
    }

    pub async fn toggle_disable_content(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> crate::Result<String> {
        let path = if content_path.ends_with(".disabled") {
            self.enable_internal(instance_id, content_path).await
        } else {
            self.disable_internal(instance_id, content_path).await
        }?;

        self.notify_instance_updated(instance_id).await?;
        // TODO: return error instead of panic
        Ok(path.expect("Path should always be returned for valid input"))
    }

    pub async fn enable_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> crate::Result<()> {
        for content_path in content_paths {
            self.enable_internal(instance_id, content_path).await?;
        }
        self.notify_instance_updated(instance_id).await?;

        Ok(())
    }

    pub async fn disable_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> crate::Result<()> {
        for content_path in content_paths {
            self.disable_internal(instance_id, content_path).await?;
        }
        self.notify_instance_updated(instance_id).await?;

        Ok(())
    }

    async fn notify_instance_updated(&self, instance_id: &str) -> crate::Result<()> {
        emit_instance(instance_id, InstancePayloadType::Edited).await
    }

    async fn enable_internal(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> crate::Result<Option<String>> {
        if !content_path.ends_with(".disabled") {
            return Ok(None);
        }

        let new_path = content_path.trim_end_matches(".disabled").to_string();
        self.rename_content_file(instance_id, content_path, &new_path)
            .await?;

        Ok(Some(new_path))
    }

    async fn disable_internal(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> crate::Result<Option<String>> {
        if content_path.ends_with(".disabled") {
            return Ok(None);
        }

        let new_path = format!("{content_path}.disabled");
        self.rename_content_file(instance_id, content_path, &new_path)
            .await?;

        Ok(Some(new_path))
    }

    async fn rename_content_file(
        &self,
        instance_id: &str,
        from: &str,
        to: &str,
    ) -> crate::Result<()> {
        let instance_dir = self.location_info.instance_dir(instance_id);
        Ok(rename(&instance_dir.join(from), &instance_dir.join(to)).await?)
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

    async fn prepare_import_data(
        &self,
        instance_id: &str,
        content_type: ContentType,
        source_paths: &[&Path],
    ) -> crate::Result<(Vec<String>, Vec<PackFile>)> {
        let mut paths = Vec::with_capacity(source_paths.len());
        let mut metadata_files = Vec::with_capacity(source_paths.len());

        for source_path in source_paths {
            let (content_path, metadata) = self
                .get_import_content_data(instance_id, content_type, source_path)
                .await?;

            paths.push(content_path);
            metadata_files.push(metadata);
        }

        Ok((paths, metadata_files))
    }

    async fn get_import_content_data(
        &self,
        instance_id: &str,
        content_type: ContentType,
        path: &Path,
    ) -> crate::Result<(String, PackFile)> {
        let content_folder = content_type.get_folder();

        let file_name = path.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
            crate::ErrorKind::NoValueFor(format!("Can't get file name {:?}", path))
        })?;

        let content_path = Path::new(content_folder)
            .join(file_name)
            .to_string_lossy()
            .to_string();

        let duplicate_error = crate::ErrorKind::ContentImportDuplicateError {
            content_path: content_path.clone(),
        };

        let absolute_content_path = self
            .location_info
            .instance_dir(instance_id)
            .join(&content_path);

        if absolute_content_path.exists() {
            return Err(duplicate_error.as_error());
        }

        let pack_file = self
            .pack_storage
            .get_pack_file(instance_id, &content_path)
            .await;

        if pack_file.is_ok() {
            return Err(duplicate_error.as_error());
        }

        let pack_file = file_to_pack_file(path, file_name).await?;

        Ok((content_path, pack_file))
    }

    async fn copy_import_files(
        &self,
        instance_id: &str,
        source_paths: &[&Path],
        content_paths: &[String],
    ) -> crate::Result<()> {
        let instance_dir = self.location_info.instance_dir(instance_id);

        futures::future::try_join_all(source_paths.iter().zip(content_paths).map(|(src, dest)| {
            let dest_path = instance_dir.join(dest);
            tokio::fs::copy(src, dest_path)
        }))
        .await?;

        Ok(())
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
