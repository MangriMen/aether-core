use std::path::{Path, PathBuf};

use super::create_instance_path_without_duplicate;

pub async fn create_instance_dir(name: &str, base_dir: &Path) -> crate::Result<(PathBuf, String)> {
    let (instance_path, sanitized_name) = create_instance_path_without_duplicate(name, base_dir);
    tokio::fs::create_dir_all(&instance_path).await?;
    Ok((instance_path, sanitized_name))
}
