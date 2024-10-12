use daedalus::modded;

use crate::state::LauncherState;

pub async fn get_loader_versions(loader: &str) -> anyhow::Result<modded::Manifest> {
    let state = LauncherState::get().await?;

    todo!("Get loader versions");

    // let loaders = CachedEntry::get_loader_manifest(loader, None, &state.pool, &state.api_semaphore)
    //     .await?
    //     .ok_or_else(|| a::ErrorKind::NoValueFor(format!("{} loader versions", loader)))?;

    // Ok(loaders.manifest)
}
