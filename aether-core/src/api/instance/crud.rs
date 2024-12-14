use std::path::Path;

use crate::{
    state::{Instance, LauncherState},
    utils::io::read_json_async,
};

pub async fn get_instance_by_path(path: &Path) -> crate::Result<Instance> {
    let instance = read_json_async(&path).await?;

    Ok(instance)
}

pub async fn get_instance(name_id: &str) -> crate::Result<Instance> {
    let state = LauncherState::get().await?;

    let instance_file = state
        .locations
        .instances_dir()
        .join(name_id)
        .join("instance.json");

    let instance = get_instance_by_path(&instance_file).await?;

    Ok(instance)
}

pub async fn get_instances() -> anyhow::Result<(Vec<Instance>, Vec<crate::Error>)> {
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

                let instance = get_instance_by_path(&instance_file).await;

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

pub async fn remove(name_id: &str) -> anyhow::Result<()> {
    let state = LauncherState::get().await?;

    let path = state.locations.instances_dir().join(name_id);

    Instance::remove_path(&path).await?;

    Ok(())
}
