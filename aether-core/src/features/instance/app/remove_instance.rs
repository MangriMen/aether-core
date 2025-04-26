use crate::features::{instance::InstanceManager, settings::LocationInfo};

pub async fn remove_instance<S>(
    storage: &S,
    location_info: &LocationInfo,
    id: &str,
) -> crate::Result<()>
where
    S: InstanceManager + ?Sized,
{
    storage.remove(id).await?;

    let instance_dir = location_info.instance_dir(id);
    if instance_dir.exists() {
        tokio::fs::remove_dir_all(&instance_dir).await?;
    }

    Ok(())
}
