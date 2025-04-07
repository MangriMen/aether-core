use std::{collections::HashMap, path::Path};

use dashmap::DashMap;

use crate::{
    event::{emit::emit_instance, InstancePayloadType},
    features::instance::{
        content_provider, ContentMetadataFile, ContentRequest, ContentResponse, ContentType,
        InstallContentPayload, Instance, InstanceFile,
    },
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

pub async fn remove_contents<I, D>(id: &str, content_paths: I) -> crate::Result<()>
where
    I: IntoIterator<Item = D>,
    D: AsRef<str>,
{
    Instance::remove_contents(id, content_paths).await?;
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

pub async fn get_content_providers() -> crate::Result<HashMap<String, String>> {
    Ok(HashMap::from([
        // ("Curseforge".to_string(), "curseforge".to_string()),
        ("Modrinth".to_string(), "modrinth".to_string()),
    ]))
}

pub async fn get_content_by_provider(payload: &ContentRequest) -> crate::Result<ContentResponse> {
    match payload.provider.as_str() {
        "modrinth" => content_provider::modrinth::search_content(payload).await,
        _ => Err(crate::ErrorKind::ContentProviderNotFound {
            provider: payload.provider.to_string(),
        }
        .as_error()),
    }
}

pub async fn get_metadata_field_to_check_installed(provider: &str) -> crate::Result<String> {
    match provider {
        "modrinth" => Ok(content_provider::modrinth::get_field_to_check_installed()),
        _ => Err(crate::ErrorKind::ContentProviderNotFound {
            provider: provider.to_string(),
        }
        .as_error()),
    }
}

pub async fn install_content(id: &str, payload: &InstallContentPayload) -> crate::Result<()> {
    let instance_file = match payload.provider.as_str() {
        "modrinth" => content_provider::modrinth::install_content(id, payload).await,
        _ => Err(crate::ErrorKind::ContentProviderNotFound {
            provider: payload.provider.to_string(),
        }
        .as_error()),
    }?;

    Instance::update_content_metadata_file(
        id,
        &instance_file.path,
        &ContentMetadataFile {
            name: instance_file.name.clone(),
            file_name: instance_file.file_name.clone(),
            hash: instance_file.hash,
            download: None,
            option: None,
            side: None,
            update_provider: Some(payload.provider.to_owned()),
            update: instance_file.update,
        },
    )
    .await?;

    Ok(())
}

pub async fn import_contents(
    id: &str,
    paths: Vec<&Path>,
    content_type: ContentType,
) -> crate::Result<()> {
    Instance::import_contents(id, paths, content_type).await
}
