use tokio::process::Command;

use crate::{
    launcher::{InstanceLaunchArgs, InstanceLaunchMetadata, InstanceLaunchSettings},
    state::{Instance, Settings},
    utils::io::IOError,
};

pub fn sanitize_instance_name(name: &str) -> String {
    name.replace(
        ['/', '\\', '?', '*', ':', '\'', '\"', '|', '<', '>', '!'],
        "_",
    )
}

pub fn get_launch_args(instance: &Instance, settings: &Settings) -> InstanceLaunchArgs {
    InstanceLaunchArgs {
        env_args: instance
            .custom_env_vars
            .clone()
            .unwrap_or(settings.custom_env_vars.clone()),
        java_args: instance
            .extra_launch_args
            .clone()
            .unwrap_or(settings.extra_launch_args.clone()),
    }
}

pub fn get_launch_settings(instance: &Instance, settings: &Settings) -> InstanceLaunchSettings {
    InstanceLaunchSettings {
        memory: instance.memory.unwrap_or(settings.memory),
        resolution: instance.game_resolution.unwrap_or(settings.game_resolution),
    }
}

pub fn get_launch_metadata(instance: &Instance, settings: &Settings) -> InstanceLaunchMetadata {
    InstanceLaunchMetadata {
        wrapper: instance
            .hooks
            .wrapper
            .clone()
            .or(settings.hooks.wrapper.clone()),
        post_exit_command: instance
            .hooks
            .post_exit
            .clone()
            .or(settings.hooks.post_exit.clone()),
    }
}

pub async fn run_pre_launch_command(instance: &Instance, settings: &Settings) -> crate::Result<()> {
    let pre_launch_commands = instance
        .hooks
        .pre_launch
        .as_ref()
        .or(settings.hooks.pre_launch.as_ref());

    if let Some(command) = pre_launch_commands {
        // TODO: hook parameters
        let mut cmd = command.split(' ');
        if let Some(command) = cmd.next() {
            let full_path = &instance.path;
            let result = Command::new(command)
                .args(cmd.collect::<Vec<&str>>())
                .current_dir(&full_path)
                .spawn()
                .map_err(|e| IOError::with_path(e, &full_path))?
                .wait()
                .await
                .map_err(IOError::from)?;

            if !result.success() {
                return Err(crate::ErrorKind::LauncherError(format!(
                    "Non-zero exit code for pre-launch hook: {}",
                    result.code().unwrap_or(-1)
                ))
                .as_error());
            }
        }
    }

    Ok(())
}
