use std::path::Path;

use crate::features::java::{Java, JavaError, JavaStorage};

use super::construct_java_from_jre;

pub async fn get_java<S>(storage: &S, version: u32) -> crate::Result<Java>
where
    S: JavaStorage + ?Sized,
{
    let java = storage.get(version).await?;

    let get_error = || JavaError::NoJREFound { version };

    if let Some(java) = java {
        Ok(construct_java_from_jre(Path::new(&java.path))
            .await
            .ok_or_else(get_error)?)
    } else {
        Err(get_error().into())
    }
}

pub async fn get_java_from_path(path: &Path) -> crate::Result<Java> {
    Ok(construct_java_from_jre(path)
        .await
        .ok_or_else(|| JavaError::NoJREFoundAtPath {
            path: path.to_path_buf(),
        })?)
}
