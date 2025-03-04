use dashmap::DashMap;

use crate::{
    event::{emit::emit_instance, InstancePayloadType},
    state::{Instance, InstanceFile},
};

use super::get;

pub async fn get_contents(id: &str) -> crate::Result<DashMap<String, InstanceFile>> {
    if let Ok(instance) = get(id).await {
        instance.get_contents().await
    } else {
        Err(crate::ErrorKind::UnmanagedProfileError(id.to_string()).as_error())
    }
}

pub async fn remove_content(id: &str, content_path: &str) -> crate::Result<()> {
    Instance::remove_content(id, content_path).await?;
    emit_instance(id, InstancePayloadType::Edited).await?;
    Ok(())
}

pub async fn toggle_disable_content(id: &str, content_path: &str) -> crate::Result<String> {
    let res = Instance::toggle_disable_content(id, content_path).await?;
    emit_instance(id, InstancePayloadType::Edited).await?;
    Ok(res)
}

pub async fn enable_contents<I, D>(id: &str, content_paths: I) -> crate::Result<()>
where
    I: IntoIterator<Item = D>,
    D: AsRef<str>,
{
    Instance::enable_contents(id, content_paths).await?;
    emit_instance(id, InstancePayloadType::Edited).await?;
    Ok(())
}

pub async fn disable_contents<I, D>(id: &str, content_paths: I) -> crate::Result<()>
where
    I: IntoIterator<Item = D>,
    D: AsRef<str>,
{
    Instance::disable_contents(id, content_paths).await?;
    emit_instance(id, InstancePayloadType::Edited).await?;
    Ok(())
}
