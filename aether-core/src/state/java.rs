use log::info;

#[derive(Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Clone)]
pub struct Java {
    pub major_version: u32,
    pub version: String,
    pub architecture: String,
    pub path: String,
}

impl Java {
    #[tracing::instrument]
    pub async fn get(
        state: &super::LauncherState,
        major_version: u32,
    ) -> crate::Result<Option<Java>> {
        info!("Get Java directory");

        let java_directory = state.locations.java_versions_dir();

        let java_versions_file = java_directory.join("versions.json");

        if !java_versions_file.exists() {
            crate::utils::io::write_json_async(&java_versions_file, Vec::<Java>::new()).await?;
            info!(
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
            return Ok(Some(version.clone()));
        }

        Ok(None)
    }

    #[tracing::instrument]
    pub async fn update(&self, state: &super::LauncherState) -> crate::Result<()> {
        let java_directory = state.locations.java_versions_dir().join("versions.json");

        let mut java_versions =
            crate::utils::io::read_json_async::<Vec<Java>>(&java_directory).await?;

        let java_version = java_versions
            .iter_mut()
            .find(|x| x.major_version == self.major_version);

        if let Some(java_version) = java_version {
            java_version.clone_from(self);
        } else {
            java_versions.push(self.clone());
        }

        crate::utils::io::write_json_async(&java_directory, &java_versions).await?;

        Ok(())
    }
}
