use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode};
// TODO: maybe change to notify_debouncer_full
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use tokio::sync::RwLock;

use futures::{channel::mpsc::channel, SinkExt, StreamExt};

use crate::{
    // features::events::InstanceEventType,
    features::{
        instance::{ContentType, InstanceInstallStage},
        settings::{location_info::INSTANCES_FOLDER_NAME, LocationInfo},
    },
};

pub type FsWatcher = RwLock<Debouncer<RecommendedWatcher>>;

fn extract_instance_path(path: &std::path::Path) -> Option<String> {
    let mut found = false;
    for component in path.components() {
        if found {
            return Some(component.as_os_str().to_string_lossy().to_string());
        }
        if component.as_os_str() == INSTANCES_FOLDER_NAME {
            found = true;
        }
    }
    None
}

fn is_crash_report(path: &std::path::Path) -> bool {
    path.components().any(|x| x.as_os_str() == "crash-reports")
        && path.extension().map(|x| x == "txt").unwrap_or(false)
}

fn handle_events(res: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>) {
    match res {
        Ok(events) => {
            let mut visited_profiles = Vec::new();

            for event in events.iter() {
                if let Some(instance_path) = extract_instance_path(&event.path) {
                    if is_crash_report(&event.path) {
                        crash_task(instance_path.to_string());
                    } else if !visited_profiles.contains(&instance_path) {
                        let _path = instance_path.to_string();
                        // tokio::spawn(async move {
                        //     let _ = emit_instance(&path, InstanceEventType::Synced).await;
                        // });
                        visited_profiles.push(instance_path);
                    }
                }
            }
        }
        Err(error) => tracing::warn!("Unable to watch file: {error}"),
    }
}

pub async fn init_watcher() -> crate::Result<FsWatcher> {
    let (mut tx, mut rx) = channel(1);

    let fs_watcher = new_debouncer(
        Duration::from_secs_f32(1.0),
        move |res: DebounceEventResult| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
    )?;

    tokio::task::spawn(async move {
        let span = tracing::span!(tracing::Level::INFO, "init_watcher");
        tracing::info!(parent: &span, "Initializing watcher");
        while let Some(res) = rx.next().await {
            let _span = span.enter();
            handle_events(res);
        }
    });

    Ok(RwLock::new(fs_watcher))
}

pub(crate) async fn watch_instances(watcher: &FsWatcher, dirs: &LocationInfo) {
    if let Ok(instances_dir) = std::fs::read_dir(dirs.instances_dir()) {
        for instance_dir in instances_dir {
            if let Ok(file_name) = instance_dir.map(|x| x.file_name()) {
                if let Some(file_name) = file_name.to_str() {
                    if file_name.starts_with(".DS_Store") {
                        continue;
                    };

                    watch_instance(file_name, watcher, dirs).await;
                }
            }
        }
    }
}

pub(crate) async fn watch_instance(instance_id: &str, watcher: &FsWatcher, dirs: &LocationInfo) {
    let instance_path = dirs.instance_dir(instance_id);

    if instance_path.exists() && instance_path.is_dir() {
        let folders = ContentType::iterator()
            .map(|x| x.get_folder())
            .chain(["crash-reports"]);

        for folder in folders {
            let path = instance_path.join(folder);

            if !path.exists() && !path.is_symlink() {
                if let Err(e) = tokio::fs::create_dir_all(&path).await {
                    tracing::error!("Failed to create directory for watcher {path:?}: {e}");
                    return;
                }
            }

            let mut watcher = watcher.write().await;
            if let Err(e) = watcher.watcher().watch(&path, RecursiveMode::Recursive) {
                tracing::error!("Failed to watch directory for watcher {path:?}: {e}");
                return;
            }
        }
    }
}

fn crash_task(id: String) {
    tokio::task::spawn(async move {
        let res = async {
            let instance = crate::api::instance::get(id).await;

            if let Ok(instance) = instance {
                // Hide warning if profile is not yet installed
                if instance.install_stage == InstanceInstallStage::Installed {
                    // emit_warning(&format!(
                    //     "Profile {} has crashed! Visit the logs page to see a crash report.",
                    //     instance.name
                    // ))
                    // .await?;
                }
            }

            Ok::<(), crate::Error>(())
        }
        .await;

        match res {
            Ok(()) => {}
            Err(err) => {
                tracing::warn!("Unable to send crash report to frontend: {err}")
            }
        };
    });
}
