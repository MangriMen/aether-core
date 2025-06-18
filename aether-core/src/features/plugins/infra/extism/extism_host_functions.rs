use std::{path::PathBuf, process::Output};

use extism::host_fn;
use path_slash::PathBufExt;

use crate::{
    core::LauncherState,
    features::{
        instance::NewInstance,
        plugins::{
            plugin_utils::{self},
            PluginContext, SerializableOutput,
        },
    },
    shared::domain::SerializableCommand,
};

host_fn!(
pub log(user_data: PluginContext; level: u32, msg: String) -> extism::Result<()> {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    log::log!(target: "plugin", plugin_utils::log_level_from_u32(level), "[{}]: {}", id, msg);
    Ok(())
});

host_fn!(
pub get_id(user_data: PluginContext;) -> extism::Result<String> {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    Ok(id)
});

// host_fn!(
// pub download_file(user_data: PluginContext; url: String, path: String) -> extism::Result<()> {
//     let context = user_data.get()?;
//     let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

//     tokio::task::block_in_place(|| -> crate::Result<()> {
//         let state = tokio::runtime::Handle::current()
//             .block_on(LauncherState::get())?;

//         let validated_path = plugin_path_to_host(&id, &path)?;

//         let response = tokio::runtime::Handle::current()
//             .block_on(fetch_advanced(
//                 Method::GET,
//                 &url,
//                 None,
//                 None,
//                 None,
//                 &state.fetch_semaphore,
//                 None,
//             ))?;

//         tokio::runtime::Handle::current()
//             .block_on(crate::shared::write_async(&validated_path, response))?;

//         Ok(())
//     })?;

//     Ok(())
// });

host_fn!(
pub instance_get_dir(user_data: PluginContext; id: String) -> extism::Result<String> {
   let res = tokio::task::block_in_place(|| -> anyhow::Result<PathBuf> {
        let state = tokio::runtime::Handle::current().block_on(
            LauncherState::get()
        )?;

       let dir = tokio::runtime::Handle::current().block_on(
            crate::api::instance::get_dir(&id)
       )?;

       Ok(dir.strip_prefix(&state.location_info.config_dir)?.to_path_buf())
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

    let dir = state.location_info.instance_plugin_dir(&instance_id, &id);

    let dir = dir.strip_prefix(&state.location_info.config_dir)?.to_path_buf();

    Ok(format!("/{}",dir.to_slash_lossy()))
});

host_fn!(
    pub instance_create(
        user_data: PluginContext;
        create_instance_request: NewInstance
    ) -> extism::Result<String> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(crate::api::instance::create(create_instance_request))
                .map_err(Into::into)
        })
    }
);

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

    let output = tokio::task::block_in_place(|| -> anyhow::Result<Output> {
        let host_command = plugin_utils::plugin_command_to_host(&id, &command)?;
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
