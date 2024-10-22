use std::path::PathBuf;

use crate::{state::Java, utils::jre};

// Validates JRE at a given path
pub async fn check_jre(path: PathBuf) -> crate::Result<Option<Java>> {
    Ok(jre::check_java_at_filepath(&path).await)
}
