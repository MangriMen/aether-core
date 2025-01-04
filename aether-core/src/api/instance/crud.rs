use std::path::{Path, PathBuf};

use crate::{
    state::{Instance, LauncherState},
    utils::io::read_json_async,
};

pub async fn get_dir(id: &str) -> crate::Result<PathBuf> {
    let state = LauncherState::get().await?;
    Ok(state.locations.instance_dir(id))
}

pub async fn get_file_path(id: &str) -> crate::Result<PathBuf> {
    Ok(get_dir(id).await?.join("instance.json"))
}

pub async fn get_by_path(path: &Path) -> crate::Result<Instance> {
    Ok(read_json_async(&path).await?)
}

pub async fn get(id: &str) -> crate::Result<Instance> {
    Ok(get_by_path(&get_file_path(&id).await?).await?)
}

pub async fn get_all() -> anyhow::Result<(Vec<Instance>, Vec<crate::Error>)> {
    let state = LauncherState::get().await?;

    let instances_dir = state.locations.instances_dir();

    if !instances_dir.exists() {
        return Ok((Vec::new(), Vec::new()));
    }

    let mut instances = Vec::new();
    let mut instances_errors: Vec<crate::Error> = Vec::new();

    for entry in instances_dir.read_dir()? {
        match entry {
            Ok(entry) => {
                let instance_file = entry.path().join("instance.json");

                let instance = get_by_path(&instance_file).await;

                match instance {
                    Ok(instance) => {
                        instances.push(instance);
                    }
                    Err(err) => instances_errors.push(err),
                }
            }
            Err(err) => instances_errors.push(err.into()),
        }
    }

    Ok((instances, instances_errors))
}

pub async fn remove(id: &str) -> crate::Result<()> {
    Instance::remove_by_path(&get_file_path(id).await?).await
}
