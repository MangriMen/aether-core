use std::{path::PathBuf, process::Output};

use extism::host_fn;
use path_slash::PathBufExt;
use reqwest::Method;

use crate::{
    core::LauncherState,
    state::{ModLoader, PackInfo, SerializableCommand, SerializableOutput},
    utils::plugin::log_level_from_u32,
};

use super::PluginContext;

host_fn!(
pub log(user_data: PluginContext; level: u32, msg: String) -> extism::Result<()> {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    log::log!(target: "plugin", log_level_from_u32(level), "[{}]: {}", id, msg);
    Ok(())
});

host_fn!(
pub get_id(user_data: PluginContext;) -> extism::Result<String> {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    Ok(id)
});

host_fn!(
pub download_file(user_data: PluginContext; url: String, path: String) -> extism::Result<()> {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    tokio::task::block_in_place(|| -> crate::Result<()> {
        let state = tokio::runtime::Handle::current()
            .block_on(LauncherState::get())?;

        let validated_path = crate::utils::plugin::plugin_path_to_host(&id, &path)?;

        let response = tokio::runtime::Handle::current()
            .block_on(crate::utils::fetch::fetch_advanced(
                Method::GET,
                &url,
                None,
                None,
                None,
                &state.fetch_semaphore,
                None,
            ))?;

        tokio::runtime::Handle::current()
            .block_on(crate::utils::io::write_async(&validated_path, response))?;

        Ok(())
    })?;

    Ok(())
});

host_fn!(
pub instance_get_dir(user_data: PluginContext; id: String) -> extism::Result<String> {
   let res = tokio::task::block_in_place(|| -> crate::Result<PathBuf> {
        let state = tokio::runtime::Handle::current().block_on(
            LauncherState::get()
        )?;

       let dir = tokio::runtime::Handle::current().block_on(
            crate::api::instance::get_dir(&id)
       )?;

       Ok(dir.strip_prefix(&state.locations.config_dir)?.to_path_buf())
   })?;

   Ok(format!("/{}",res.to_slash_lossy()))
});

host_fn!(
pub instance_plugin_get_dir(user_data: PluginContext; instance_id: String) -> extism::Result<String> {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    let state = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(
            LauncherState::get()
        )})?;

    let dir = state.locations.instance_plugin_dir(&instance_id, &id);

    let dir = dir.strip_prefix(&state.locations.config_dir)?.to_path_buf();

    Ok(format!("/{}",dir.to_slash_lossy()))
});

host_fn!(
pub instance_create(
    user_data: PluginContext;
    name: String,
    game_version: String,
    mod_loader: String,
    loader_version: Option<String>,
    icon_path: Option<String>,
    skip_install_instance: Option<i64>,
    pack_info: Option<PackInfo>
) -> extism::Result<String> {
    let mod_loader = ModLoader::from_string(&mod_loader);

    let res =
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                crate::api::instance::create(
                    name,
                    game_version,
                    mod_loader,
                    loader_version,
                    icon_path,
                    skip_install_instance.map(|x| x != 0),
                    pack_info
                )
            )
        })?;


    Ok(res)
});

host_fn!(
pub get_java(user_data: PluginContext; version: u32) -> extism::Result<String> {
    let res = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(
            crate::api::java::get(version)
        )
    })?;

    Ok(res)
});

host_fn!(
pub install_java(user_data: PluginContext; version: u32) -> extism::Result<String> {
    let res = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(
            crate::api::java::install(version)
        )
    })?;

    Ok(res)
});

host_fn!(
pub run_command(user_data: PluginContext; command: SerializableCommand) -> extism::Result<SerializableOutput> {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    log::debug!("Processing command from plugin: {:?}", command);

    let output = tokio::task::block_in_place(|| -> crate::Result<Output> {
        let host_command = crate::utils::plugin::plugin_command_to_host(&id, &command)?;
        let mut cmd = host_command.to_tokio_command();

        log::debug!("Running command: {:?}", host_command);

        Ok(tokio::runtime::Handle::current().block_on(cmd.output())?)
    })?;

    if !output.status.success() {
        log::error!("Command failed: {:?}, stderr: {:?}", command, String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::Error::msg("Command execution failed"));
    }

    Ok(SerializableOutput::from_output(&output))
});

// host_fn!(
// pub get_contents(user_data: PluginContext; id: String) -> extism::Result<DashMap<String, InstanceFile>> {
//     let res = tokio::task::block_in_place(|| {
//         tokio::runtime::Handle::current().block_on(
//             crate::api::instance::get_contents(&id)
//         )
//     })?;

//     Ok(res)
// });

// host_fn!(
// pub enable_contents(user_data: PluginContext; instance_id: String, content_paths: Vec<String>) -> extism::Result<()> {
//     let res = tokio::task::block_in_place(|| {
//         tokio::runtime::Handle::current().block_on(
//             crate::api::instance::disable_contents(&instance_id, content_paths)
//         )
//     })?;

//     Ok(res)
// });

// host_fn!(
// pub disable_contents(user_data: PluginContext; instance_id: String, content_paths: Vec<String>) -> extism::Result<()> {
//     let res = tokio::task::block_in_place(|| {
//         tokio::runtime::Handle::current().block_on(
//             crate::api::instance::disable_contents(&instance_id, content_paths)
//         )
//     })?;

//     Ok(res)
// });
