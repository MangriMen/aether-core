use std::path::Path;

use extism::ToBytes;
use extism_convert::Json;

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Clone, ToBytes)]
#[encoding(Json)]
pub struct Java {
    pub major_version: u32,
    pub version: String,
    pub architecture: String,
    pub path: String,
}

impl Java {
    #[tracing::instrument]
    pub async fn from_path(path: &Path) -> crate::Result<Java> {
        crate::jre::check_jre_at_filepath(path)
            .await
            .ok_or_else(|| {
                crate::ErrorKind::LauncherError(format!(
                    "Java path invalid or non-functional: {:?}",
                    path
                ))
                .as_error()
            })
    }

    #[tracing::instrument]
    pub async fn get(
        state: &super::LauncherState,
        major_version: u32,
    ) -> crate::Result<Option<Java>> {
        let java_directory = state.locations.java_dir();

        let java_versions_file = java_directory.join("versions.json");

        log::info!("Reading java versions file");

        if !java_versions_file.exists() {
            crate::utils::io::write_json_async(&java_versions_file, Vec::<Java>::new()).await?;
            log::info!(
                "Java versions file created at {}.\n",
                java_versions_file.display()
            );
        }

        let java_versions =
            crate::utils::io::read_json_async::<Vec<Java>>(java_versions_file).await?;

        let version = java_versions
            .iter()
            .find(|x| x.major_version == major_version);

        if let Some(version) = version {
            log::info!("Found java version {}", version.major_version);
            return Ok(Some(version.clone()));
        }

        Ok(None)
    }

    #[tracing::instrument]
    pub async fn upsert(&self, state: &super::LauncherState) -> crate::Result<()> {
        log::info!("Reading java versions file");

        let java_directory = state.locations.java_dir().join("versions.json");

        let mut java_versions =
            crate::utils::io::read_json_async::<Vec<Java>>(&java_directory).await?;

        let java_version = java_versions
            .iter_mut()
            .find(|x| x.major_version == self.major_version);

        log::info!("Updating java versions file");
        if let Some(java_version) = java_version {
            java_version.clone_from(self);
        } else {
            java_versions.push(self.clone());
        }

        crate::utils::io::write_json_async(&java_directory, &java_versions).await?;

        Ok(())
    }
}
