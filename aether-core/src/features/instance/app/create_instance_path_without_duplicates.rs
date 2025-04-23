use std::path::{Path, PathBuf};

use super::sanitize_instance_name;

pub fn create_instance_path_without_duplicate(name: &str, base_dir: &Path) -> (PathBuf, String) {
    let base_sanitized_name = sanitize_instance_name(name);

    let mut sanitized_name = base_sanitized_name.clone();
    let mut full_path = base_dir.join(&sanitized_name);

    let mut counter = 1;
    while full_path.exists() {
        sanitized_name = format!("{}-{}", base_sanitized_name, counter);
        full_path = base_dir.join(&sanitized_name);
        counter += 1;
    }

    (full_path, sanitized_name)
}
