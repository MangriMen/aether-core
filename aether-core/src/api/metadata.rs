use daedalus::modded;

use crate::{launcher::download_loaders_manifests, state::LauncherState};

pub async fn get_loader_versions(loader: &str) -> crate::Result<modded::Manifest> {
    let state = LauncherState::get().await?;

    let loaders = download_loaders_manifests(&state, loader, true).await;

    match loaders {
        Ok(loaders) => Ok(loaders),
        Err(_) => {
            Err(crate::ErrorKind::NoValueFor(format!("{} loader versions", loader)).as_error())
        }
    }
}
