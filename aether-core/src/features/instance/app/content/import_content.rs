use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;

use crate::{
    features::{
        events::{EventEmitter, EventEmitterExt, InstanceEventType},
        instance::{ContentType, PackFile, PackStorage},
        settings::LocationInfo,
    },
    shared::{domain::AsyncUseCaseWithInputAndError, read_async, sha1_async},
};

pub struct ImportContent {
    instance_id: String,
    content_type: ContentType,
    source_paths: Vec<PathBuf>,
}

impl ImportContent {
    pub fn single(instance_id: String, content_type: ContentType, source_path: PathBuf) -> Self {
        Self {
            instance_id,
            content_type,
            source_paths: vec![source_path],
        }
    }

    pub fn multiple(
        instance_id: String,
        content_type: ContentType,
        source_paths: Vec<PathBuf>,
    ) -> Self {
        Self {
            instance_id,
            content_type,
            source_paths,
        }
    }
}

pub struct ImportContentUseCase<E: EventEmitter, PS: PackStorage> {
    event_emitter: Arc<E>,
    pack_storage: Arc<PS>,
    location_info: Arc<LocationInfo>,
}

impl<E: EventEmitter, PS: PackStorage> ImportContentUseCase<E, PS> {
    pub fn new(
        event_emitter: Arc<E>,
        pack_storage: Arc<PS>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            event_emitter,
            pack_storage,
            location_info,
        }
    }

    async fn prepare_import_data(
        &self,
        instance_id: &str,
        content_type: ContentType,
        source_paths: &[PathBuf],
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
        source_paths: &[PathBuf],
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

#[async_trait]
impl<E, PS> AsyncUseCaseWithInputAndError for ImportContentUseCase<E, PS>
where
    E: EventEmitter,
    PS: PackStorage + Send + Sync,
{
    type Input = ImportContent;
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let ImportContent {
            instance_id,
            content_type,
            source_paths,
        } = input;

        let (content_paths, pack_files) = self
            .prepare_import_data(&instance_id, content_type, source_paths.as_slice())
            .await?;

        self.copy_import_files(&instance_id, source_paths.as_slice(), &content_paths)
            .await?;

        self.pack_storage
            .update_pack_file_many(&instance_id, &content_paths, &pack_files)
            .await?;

        self.event_emitter
            .emit_instance(instance_id.to_string(), InstanceEventType::Edited)?;

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
