use std::path::Path;

use crate::{
    state::{Instance, LauncherState},
    utils::io::read_json_async,
};

pub enum InstanceRead {
    Instance(Instance),
    Error(String),
}

pub async fn get_instance_by_path(path: &Path) -> anyhow::Result<Instance> {
    let instance = read_json_async(&path).await?;

    Ok(instance)
}

pub async fn get_instance(name_id: &str) -> anyhow::Result<Instance> {
    let state = LauncherState::get().await?;

    let instance_file = state
        .locations
        .instances_dir()
        .join(name_id)
        .join("instance.json");

    let instance = get_instance_by_path(&instance_file).await?;

    Ok(instance)
}

pub async fn get_instances() -> anyhow::Result<Vec<Instance>> {
    let state = LauncherState::get().await?;

    let instances_dir = state.locations.instances_dir();

    if !instances_dir.exists() {
        return Ok(Vec::new());
    }

    let mut instances = Vec::new();

    for entry in instances_dir.read_dir()? {
        match entry {
            Ok(entry) => {
                let instance_file = entry.path().join("instance.json");

                let instance = get_instance_by_path(&instance_file).await;

                match instance {
                    Ok(instance) => {
                        instances.push(InstanceRead::Instance(instance));
                    }
                    Err(err) => instances.push(InstanceRead::Error(err.to_string())),
                }
            }
            Err(err) => instances.push(InstanceRead::Error(err.to_string())),
        }
    }

    let final_instances = instances
        .iter()
        .filter(|instance| match instance {
            InstanceRead::Instance(_) => true,
            InstanceRead::Error(_) => false,
        })
        .map(|instance| match instance {
            InstanceRead::Instance(instance) => instance.clone(),
            InstanceRead::Error(_) => unreachable!(),
        })
        .collect::<Vec<_>>();

    Ok(final_instances)
}

pub async fn remove(name_id: &str) -> anyhow::Result<()> {
    let state = LauncherState::get().await?;

    let path = state.locations.instances_dir().join(name_id);

    Instance::remove_path(&path).await?;

    Ok(())
}

// Get a copy of the instances
// #[tracing::instrument]
// pub async fn get_instances() -> anyhow::Result<Vec<InstanceRead>> {
//     let state = LauncherState::get().await?;

//     let instance_dirs = state.locations.instances_dir().read_dir()?;

//     let instances = stream::iter(instance_dirs)
//         .then(|dir| async move {
//             match dir {
//                 Ok(dir) => {
//                     let instance_file = dir.path().join("instance.json");

//                     let data = tokio::fs::read(&instance_file).await;

//                     match data {
//                         Ok(data) => {
//                             let res = serde_json::from_slice::<Instance>(&data);

//                             match res {
//                                 Ok(res) => InstanceRead::Instance(res),
//                                 Err(err) => InstanceRead::Error(err.to_string()),
//                             }
//                         }
//                         Err(err) => InstanceRead::Error(err.to_string()),
//                     }
//                 }
//                 Err(err) => InstanceRead::Error(err.to_string()),
//             }
//         })
//         .boxed()
//         .try_collect::<Vec<_>>()
//         .await?;

//     // let exists_instances = instances.into_iter().filter_map(|it| it.ok()).collect();

//     Ok(instances)
// }
