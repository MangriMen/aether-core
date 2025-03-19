use std::{collections::HashMap, path::Path};

use dashmap::DashMap;

use crate::{
    event::{emit::emit_instance, InstancePayloadType},
    state::{
        content_provider, ContentRequest, ContentResponse, InstallContentPayload, Instance,
        InstanceFile, InstancePack, InstancePackFile, InstancePackIndex,
    },
};

use super::get;

pub async fn get_pack(id: &str) -> crate::Result<InstancePack> {
    Instance::get_pack(id).await
}

pub async fn get_pack_index(id: &str) -> crate::Result<InstancePackIndex> {
    InstancePackIndex::from_file(Path::new(&get_pack(id).await?.index)).await
}

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

pub async fn get_content_providers() -> crate::Result<HashMap<String, String>> {
    Ok(HashMap::from([
        // ("Curseforge".to_string(), "curseforge".to_string()),
        ("Modrinth".to_string(), "modrinth".to_string()),
    ]))
}

pub async fn get_content_by_provider(payload: &ContentRequest) -> crate::Result<ContentResponse> {
    match payload.provider.as_str() {
        "modrinth" => content_provider::modrinth::get_content(payload).await,
        _ => Err(crate::ErrorKind::ContentProviderNotFound {
            provider: payload.provider.to_string(),
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

    let pack = get_pack(id).await?;

    let index_path = Path::new(&pack.index);

    let mut pack_index = match InstancePackIndex::from_file(index_path).await {
        Ok(index) => index,
        Err(_) => InstancePackIndex {
            hash_format: "sha1".to_owned(),
            files: Vec::new(),
        },
    };

    pack_index.files.push(InstancePackFile {
        file: instance_file.path,
        hash: instance_file.hash,
        alias: None,
        hash_format: Some("sha1".to_owned()),
        metafile: Some(true),
        preserve: None,
    });

    pack_index.write_file(index_path).await?;

    Ok(())
}
