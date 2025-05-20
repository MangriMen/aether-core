use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::{
    features::java::{
        utils::{extract_java_major_minor_version, get_java_properties, JavaProperties},
        Java, JavaInstallationService,
    },
    shared,
};

use super::{JAVA_BIN, JAVA_WINDOW_BIN};

pub struct FsJavaInstallationService;

impl FsJavaInstallationService {
    /// Ensures that the given path ends with `JAVA_WINDOW_BIN`.
    ///
    /// If the provided path does not already end with `JAVA_WINDOW_BIN`,
    /// this function appends it to the path. Otherwise, it returns the path unchanged.
    fn get_java_window_bin_path(path: &Path) -> Option<PathBuf> {
        let java_window_bin_path = if path.file_name()?.to_str()? != JAVA_WINDOW_BIN {
            path.join(JAVA_WINDOW_BIN)
        } else {
            path.to_path_buf()
        };
        Some(java_window_bin_path)
    }
}

#[async_trait]
impl JavaInstallationService for FsJavaInstallationService {
    /// Attempts to resolve the given file path and retrieve the Java version located at this path.
    ///
    /// Returns `None` if the path does not exist or if a valid Java installation is not found at the specified path.
    async fn locate_java(&self, path: &Path) -> Option<Java> {
        // Attempt to canonicalize the potential Java filepath
        // If it fails, return None (Java is not here)
        let canonical_path = shared::canonicalize(path).ok()?;

        let java_window_bin_path = Self::get_java_window_bin_path(&canonical_path)?;
        if !java_window_bin_path.exists() {
            return None;
        }

        // Create the path for the Java binary (replacing JAVA_WINDOW_BIN with JAVA_BIN)
        let java_bin_path = java_window_bin_path.with_file_name(JAVA_BIN);

        // Get the Java version and architecture
        let JavaProperties {
            version,
            architecture,
        } = get_java_properties(&java_bin_path).ok()?;

        // Extract version and architecture information
        if let (Some(version), Some(architecture)) = (version, architecture) {
            extract_java_major_minor_version(&version)
                .ok()
                .map(|(_, major_version)| Java {
                    major_version,
                    path: java_window_bin_path.to_string_lossy().to_string(),
                    version: version.to_string(),
                    architecture: architecture.to_string(),
                })
        } else {
            None
        }
    }
}
