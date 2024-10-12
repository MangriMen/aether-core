use chrono::{Datelike, Utc};
use uuid::Uuid;

use crate::state::{self, Credentials, LauncherState};

use super::get_instance_by_path;

#[tracing::instrument]
pub async fn run(name: &str) -> anyhow::Result<()> {
    run_credentials(
        name,
        &Credentials {
            id: Uuid::new_v4(),
            username: "Test".to_owned(),
            access_token: "".to_owned(),
            refresh_token: "".to_owned(),
            expires: Utc::now().with_year(2025).unwrap(),
            active: true,
        },
    ).await
}

pub async fn run_credentials(name: &str, credentials: &state::Credentials) -> anyhow::Result<()> {
    let state = LauncherState::get().await?;

    let instance_file = state
        .locations
        .instances_dir()
        .join(name)
        .join("instance.json");

    let instance = get_instance_by_path(&instance_file).await?;

    crate::launcher::launch_minecraft(
        &instance,
        &[],
        &[],
        &state::MemorySettings { maximum: 2048 },
        &state::WindowSize(1280, 720),
        credentials,
    )
    .await
}
