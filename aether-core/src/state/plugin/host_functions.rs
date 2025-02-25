use std::path::PathBuf;

use extism::host_fn;
use reqwest::Method;

use crate::state::{InstancePluginSettings, LauncherState, ModLoader};

use super::PluginContext;

fn validate_plugin_path(id: &str, path: &str) -> Result<PathBuf, String> {
    let state = tokio::runtime::Handle::current()
        .block_on(LauncherState::get())
        .unwrap();

    let base_dir = state.locations.plugin_cache_dir(id);
    std::fs::create_dir_all(&base_dir).unwrap();

    let requested_path = base_dir.join(path);

    let requested_parent = if requested_path.is_dir() {
        &requested_path
    } else {
        &requested_path.parent().unwrap().to_path_buf()
    };

    std::fs::create_dir_all(requested_parent).unwrap();

    let canonical_base = crate::utils::io::canonicalize(&base_dir).unwrap();
    let canonical_requested = crate::utils::io::canonicalize(requested_parent).unwrap();

    if canonical_requested.starts_with(&canonical_base) {
        Ok(requested_path)
    } else {
        Err(format!(
            "Path {:?} is not in the expected directory {:?}",
            &requested_path, &canonical_base
        ))
    }
}

host_fn!(
pub log_debug(user_data: PluginContext; msg: String) -> extism::Result<()> {
    let context = user_data.get().unwrap();
    let id = context.lock().unwrap().id.to_string();

    log::debug!("[{}]: {}", id, msg);
    Ok(())
});

host_fn!(
pub download_file(user_data: PluginContext; url: String, path: String) -> extism::Result<()> {
    tokio::task::spawn_blocking(move || {
        tokio::task::block_in_place(|| {
            let state = tokio::runtime::Handle::current()
                .block_on(LauncherState::get())
                .unwrap();
            let context = user_data.get().unwrap();
            let id = context.lock().unwrap().id.to_string();
            let validated_path = validate_plugin_path(&id, &path).unwrap();

            let response = tokio::runtime::Handle::current()
                .block_on(crate::utils::fetch::fetch_advanced(
                    Method::GET,
                    &url,
                    None,
                    None,
                    None,
                    &state.fetch_semaphore,
                    None,
                ))
                .unwrap();

            tokio::runtime::Handle::current()
                .block_on(crate::utils::io::write_async(&validated_path, response))
                .unwrap();
        });
    });

    Ok(())
}
);

host_fn!(
    pub file_exists(user_data: PluginContext; path: String) -> extism::Result<i64> {
        let result = tokio::task::block_in_place(|| {

            let context = user_data.get().unwrap();
            let id = context.lock().unwrap().id.to_string();
            let validated_path = validate_plugin_path(&id, &path).unwrap();

            i64::from(validated_path.exists())
        });

        Ok(result)
    }
);

host_fn!(
    pub create_instance(
        user_data: PluginContext;
        name: String,
        game_version: String,
        mod_loader: String,
        loader_version: Option<String>,
        icon_path: Option<String>,
        skip_install_instance: Option<i64>
    ) -> extism::Result<()> {
        tokio::task::spawn_blocking(move || {
            tokio::task::block_in_place(|| {
                let context = user_data.get().unwrap();
                let id = context.lock().unwrap().id.to_string();

                let mod_loader = ModLoader::from_string(&mod_loader);
                let instance_plugin_settings = InstancePluginSettings {
                    pre_launch:Some(id)
                };

                tokio::runtime::Handle::current().block_on(
                    crate::api::instance::create(
                        name,
                        game_version,
                        mod_loader,
                        loader_version,
                        icon_path,
                        skip_install_instance.map(|x| x != 0),
                        Some(instance_plugin_settings)
                    )
                ).unwrap();
            });
        });


        Ok(())
    }
);

host_fn!(
    pub fetch_toml(user_data: PluginContext; url: String) -> extism::Result<String, String> {
        let result = tokio::task::block_in_place(|| {
            let state = tokio::runtime::Handle::current()
                .block_on(LauncherState::get())
                .unwrap();

            let response = tokio::runtime::Handle::current()
                .block_on(crate::utils::fetch::fetch_advanced(
                    Method::GET,
                    &url,
                    None,
                    None,
                    None,
                    &state.fetch_semaphore,
                    None,
                ))
                .unwrap();

            response.to_vec()
        });

        Ok(result)
    }
);
