use std::{io::Cursor, path::Path};

use async_trait::async_trait;
use tempfile::TempDir;

use crate::{
    features::plugins::{
        ExtractedPlugin, PluginContent, PluginError, PluginExtractor, PluginManifest,
        ZipPluginExtractorConstants,
    },
    shared::{read_async, IoError},
};

#[derive(Default)]
pub struct ZipPluginExtractor {
    constants: ZipPluginExtractorConstants,
}

impl ZipPluginExtractor {
    pub fn new(constants: ZipPluginExtractorConstants) -> Self {
        Self { constants }
    }

    async fn read_manifest(
        &self,
        archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>,
        get_error: impl Fn() -> PluginError,
    ) -> Result<PluginManifest, PluginError> {
        let manifest_file = archive
            .by_name(self.constants.manifest_filename)
            .map_err(|_| get_error())?;

        Ok(serde_json::from_reader(manifest_file)
            .map_err(|e| IoError::DeserializationError(e.to_string()))?)
    }
}

#[async_trait]
impl PluginExtractor for ZipPluginExtractor {
    async fn extract(&self, file_path: &Path) -> Result<ExtractedPlugin, PluginError> {
        let get_error = || PluginError::ImportError {
            path: file_path.to_path_buf(),
        };

        let file = read_async(file_path).await?;

        let mut archive = zip::ZipArchive::new(Cursor::new(file)).map_err(|_| get_error())?;

        let manifest = self.read_manifest(&mut archive, get_error).await?;

        let plugin_id = manifest.metadata.id.clone();

        let temp_dir = TempDir::new().map_err(IoError::from)?;
        archive.extract(&temp_dir).map_err(|_| get_error())?;

        Ok(ExtractedPlugin {
            plugin_id,
            manifest,
            content: PluginContent::Filesystem { temp_dir },
        })
    }
}
