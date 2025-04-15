use crate::{
    core::LauncherState,
    features::java::{Java, JavaStorage},
};

use super::{construct_java_from_jre, install_jre};

pub async fn install_java<S>(
    state: &LauncherState,
    storage: &S,
    version: u32,
) -> crate::Result<Java>
where
    S: JavaStorage + ?Sized,
{
    let installed_jre_path =
        install_jre(version, &state.locations.java_dir(), &state.fetch_semaphore).await?;
    let java = construct_java_from_jre(&installed_jre_path)
        .await
        .ok_or_else(|| {
            crate::ErrorKind::LauncherError(format!("Java {} not found", version)).as_error()
        })?;
    storage.upsert(&java).await?;

    Ok(java)
}
