use std::{path::Path, process::Command};

use crate::features::java::JREError;

pub fn get_java_version_and_arch_from_jre(path: &Path) -> (Option<String>, Option<String>) {
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
pub fn extract_java_major_minor_version(version: &str) -> Result<(u32, u32), JREError> {
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
