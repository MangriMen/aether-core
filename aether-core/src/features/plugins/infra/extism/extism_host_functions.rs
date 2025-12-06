use extism::{convert::Msgpack, host_fn};
use path_slash::PathBufExt;

use crate::{
    core::{domain::HostResult, LauncherState},
    features::{
        instance::NewInstance,
        plugins::{
            plugin_utils::{self},
            PluginContext, SerializableOutput,
        },
    },
    shared::{domain::SerializableCommand, execute_async},
};

pub fn to_extism_res<T>(res: crate::Result<T>) -> Result<HostResult<T>, extism::Error> {
    Ok(HostResult::from(res))
}

host_fn!(
pub log(user_data: PluginContext; level: u32, msg: String) -> () {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    log::log!(target: "plugin", plugin_utils::log_level_from_u32(level), "[{}]: {}", id, msg);
    Ok(())
});

host_fn!(
pub get_id(user_data: PluginContext;) -> String {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    Ok(id)
});

host_fn!(
pub instance_get_dir(user_data: PluginContext; id: String) -> HostResult<String> {
    to_extism_res(
        execute_async(async move {
            let state =  LauncherState::get().await?;
            let dir = crate::api::instance::get_dir(&id).await?;
            let relative_path = dir.strip_prefix(&state.location_info.config_dir)
                .map_err(|_| crate::ErrorKind::CoreError("Strip prefix error".to_owned()))?
                .to_path_buf();

            Ok(format!("/{}",relative_path.to_slash_lossy()))
        })
    )
});

host_fn!(
pub instance_plugin_get_dir(user_data: PluginContext; instance_id: String) -> HostResult<String> {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    to_extism_res(
        execute_async(async move {
            let state = LauncherState::get().await?;

            let dir = state.location_info.instance_plugin_dir(&instance_id, &id);

            let dir = dir.strip_prefix(&state.location_info.config_dir)
                .map_err(|_| crate::ErrorKind::CoreError("Strip prefix error".to_owned()))?
                .to_path_buf();

            Ok(format!("/{}",dir.to_slash_lossy()))
        })
    )
});

host_fn!(
    pub instance_create(
        user_data: PluginContext;
        new_instance: NewInstance
    ) -> HostResult<String> {
        to_extism_res(
            execute_async(async move {
                crate::api::instance::create(new_instance).await
            })
        )
    }
);

host_fn!(
pub get_java(user_data: PluginContext; version: u32) -> HostResult<Java> {
    to_extism_res(
        execute_async(async move {
            crate::api::java::get(version).await
        })
    )
});

host_fn!(
pub install_java(user_data: PluginContext; version: u32) -> HostResult<Java> {
    to_extism_res(
        execute_async(async move {
            LauncherState::get().await?;

            crate::api::java::install(version).await
        })
    )
});

host_fn!(
pub run_command(user_data: PluginContext; command: SerializableCommand) -> HostResult<SerializableOutput> {
    let context = user_data.get()?;
    let id = context.lock().map_err(|_| anyhow::Error::msg("Failed to lock plugin context"))?.id.clone();

    let command_for_log = command.clone();
    log::debug!("Processing command from plugin: {:?}", command_for_log);

    to_extism_res(
        execute_async(async move {
            let id = id.clone();
            let command = command.clone();

            let state = LauncherState::get().await?;

            let host_command = plugin_utils::plugin_command_to_host(&id, &command, &state.location_info)?;
            let mut cmd = host_command.to_tokio_command();

            log::debug!("Running command: {:?}", host_command);
            let output = cmd.output().await;

            match output {
                Ok(output) => {
                    if !output.status.success() {
                        log::error!("Command failed: {:?}, stderr: {:?}", command_for_log, String::from_utf8_lossy(&output.stderr));
                        return Err(crate::ErrorKind::CoreError("Command execution failed".to_string()).as_error());
                    }

                    Ok(SerializableOutput::from_output(&output))
                },
                Err(err) => {
                    log::debug!("Update command run error {:?}", err);
                    Err(crate::ErrorKind::CoreError(format!("Failed to run command: {:?}", cmd)).as_error())
                }
            }
        })
    )
});

host_fn!(
pub list_content(user_data: PluginContext; id: String) -> HostResult<DashMap<String, InstanceFile>> {
    to_extism_res(
        execute_async(async move {
            crate::api::instance::list_content(id).await
        })
    )
});

host_fn!(
pub enable_contents(user_data: PluginContext; instance_id: String, content_paths: Msgpack<Vec<String>>) -> HostResult<()> {
    to_extism_res(
        execute_async(async move {
            crate::api::instance::enable_contents(instance_id, content_paths.0).await
        })
    )
});

host_fn!(
pub disable_contents(user_data: PluginContext; instance_id: String, content_paths: Msgpack<Vec<String>>) -> HostResult<()> {
    to_extism_res(
        execute_async(async move {
            crate::api::instance::disable_contents(instance_id, content_paths.0).await
        })
    )
});
