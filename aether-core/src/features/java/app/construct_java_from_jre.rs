use std::path::Path;

use crate::features::java::{
    utils::{extract_java_major_minor_version, get_java_version_and_arch_from_jre},
    Java,
};

use super::constants::{JAVA_BIN, JAVA_WINDOW_BIN};

// For example filepath 'path', attempt to resolve it and get a Java version at this path
// If no such path exists, or no such valid java at this path exists, returns None
#[tracing::instrument]
pub async fn construct_java_from_jre(path: &Path) -> Option<Java> {
    // Attempt to canonicalize the potential Java filepath
    // If it fails, return None (Java is not here)
    let path = crate::utils::io::canonicalize(path).ok()?;

    // Check if JAVA_WINDOW_BIN is present at the end of the path,
    // if not, append it
    let java_window_bin_path = if path.file_name()?.to_str()? != JAVA_WINDOW_BIN {
        path.join(JAVA_WINDOW_BIN)
    } else {
        path
    };

    // If the path does not exist, return None
    if !java_window_bin_path.exists() {
        return None;
    }

    // Create the path for the Java binary (replacing JAVA_WINDOW_BIN with JAVA_BIN)
    let java_bin_path = java_window_bin_path.with_file_name(JAVA_BIN);

    // Get the Java version and architecture
    let (java_version, java_arch) = get_java_version_and_arch_from_jre(&java_bin_path);

    // Extract version and architecture information
    if let (Some(java_version), Some(java_arch)) = (java_version, java_arch) {
        extract_java_major_minor_version(&java_version)
            .ok()
            .map(|(_, major_version)| Java {
                major_version,
                path: java_window_bin_path.to_string_lossy().to_string(),
                version: java_version.to_string(),
                architecture: java_arch.to_string(),
            })
    } else {
        None
    }
}
