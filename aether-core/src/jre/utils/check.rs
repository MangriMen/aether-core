use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::state::Java;

use super::JREError;

#[cfg(target_os = "windows")]
pub const JAVA_NO_WINDOW_BIN: &str = "java.exe";

#[cfg(not(target_os = "windows"))]
pub const JAVA_NO_WINDOW_BIN: &str = "java";

#[cfg(target_os = "windows")]
pub const JAVA_BIN: &str = "javaw.exe";

#[cfg(not(target_os = "windows"))]
pub const JAVA_BIN: &str = "java";

// For example filepath 'path', attempt to resolve it and get a Java version at this path
// If no such path exists, or no such valid java at this path exists, returns None
#[tracing::instrument]
pub async fn check_jre_at_filepath(path: &Path) -> Option<Java> {
    // Attempt to canonicalize the potential java filepath
    // If it fails, this path does not exist and None is returned (no Java here)
    let Ok(path) = crate::utils::io::canonicalize(path) else {
        return None;
    };

    // Checks for existence of Java at this filepath
    // Adds JAVA_BIN to the end of the path if it is not already there
    let java = if path.file_name()?.to_str()? != JAVA_BIN {
        path.join(JAVA_BIN)
    } else {
        path
    };

    if !java.exists() {
        return None;
    };

    let java_no_window =
        PathBuf::from(java.to_string_lossy().replace(JAVA_BIN, JAVA_NO_WINDOW_BIN));

    let output = Command::new(java_no_window)
        .arg("-XshowSettings:properties")
        .arg("-version")
        .env_remove("_JAVA_OPTIONS")
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    });

    let mut java_version = None;
    let mut java_arch = None;

    for line in stdout.lines() {
        let mut parts = line.split('=');
        let key = parts.next().unwrap_or_default().trim();
        let value = parts.next().unwrap_or_default().trim();

        if key == "os.arch" {
            java_arch = Some(value);
        } else if key == "java.version" {
            java_version = Some(value);
        }
    }

    // Extract version info from it
    if let Some(arch) = java_arch {
        if let Some(version) = java_version {
            if let Ok((_, major_version)) = extract_java_majorminor_version(version) {
                let path = java.to_string_lossy().to_string();
                return Some(Java {
                    major_version,
                    path,
                    version: version.to_string(),
                    architecture: arch.to_string(),
                });
            }
        }
    }
    None
}

/// Extract major/minor version from a java version string
/// Gets the minor version or an error, and assumes 1 for major version if it could not find
/// "1.8.0_361" -> (1, 8)
/// "20" -> (1, 20)
pub fn extract_java_majorminor_version(version: &str) -> Result<(u32, u32), JREError> {
    let mut split = version.split('.');
    let major_opt = split.next();

    let mut major;
    // Try minor. If doesn't exist, in format like "20" so use major
    let mut minor = if let Some(minor) = split.next() {
        major = major_opt.unwrap_or("1").parse::<u32>()?;
        minor.parse::<u32>()?
    } else {
        // Formatted like "20", only one value means that is minor version
        major = 1;
        major_opt
            .ok_or_else(|| JREError::InvalidJREVersion(version.to_string()))?
            .parse::<u32>()?
    };

    // Java start should always be 1. If more than 1, it is formatted like "17.0.1.2" and starts with minor version
    if major > 1 {
        minor = major;
        major = 1;
    }

    Ok((major, minor))
}
