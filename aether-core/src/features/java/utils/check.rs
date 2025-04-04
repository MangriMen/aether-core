use std::{path::Path, process::Command};

use crate::features::java::{
    application::constants::{JAVA_BIN, JAVA_WINDOW_BIN},
    JREError, Java,
};

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

fn get_java_version_and_arch_from_jre(path: &Path) -> (Option<String>, Option<String>) {
    let output = Command::new(path)
        .arg("-XshowSettings:properties")
        .arg("-version")
        .env_remove("_JAVA_OPTIONS")
        .output();

    let stdout = match &output {
        Ok(output) => String::from_utf8_lossy(if output.stdout.is_empty() {
            &output.stderr
        } else {
            &output.stdout
        }),
        Err(_) => return (None, None),
    };

    let mut java_version = None;
    let mut java_arch = None;

    for line in stdout.lines() {
        let (key, value) = line.split_once('=').unwrap_or(("", ""));

        match key.trim() {
            "os.arch" => java_arch = Some(value.trim().to_string()),
            "java.version" => java_version = Some(value.trim().to_string()),
            _ => {}
        }
    }

    (java_version, java_arch)
}

/// Extracts the major and minor version from a Java version string.
///
/// If the string doesn't contain a minor version, it assumes 1 for the major version.
///
/// Examples:
/// - "1.8.0_361" -> (1, 8)
/// - "20" -> (1, 20)
pub(super) fn extract_java_major_minor_version(version: &str) -> Result<(u32, u32), JREError> {
    let mut split = version.split('.');

    let major_str = split
        .next()
        .ok_or_else(|| JREError::InvalidJREVersion(version.to_string()))?;
    let major = major_str.parse::<u32>()?;

    // Java start should always be 1. If more than 1, it is formatted like "17.0.1.2" and starts with minor version
    // Formatted like "20", only one value means that is minor version
    if major > 1 {
        Ok((1, major))
    } else {
        let minor_str = split
            .next()
            .ok_or_else(|| JREError::InvalidJREVersion(version.to_string()))?;
        let minor = minor_str.parse::<u32>()?;
        Ok((major, minor))
    }
}
