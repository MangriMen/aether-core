use crate::features::{
    instance::Instance,
    minecraft::LaunchSettings,
    settings::{Hooks, Settings},
};

pub fn get_merged_settings(instance: &Instance, settings: &Settings) -> LaunchSettings {
    LaunchSettings {
        extra_launch_args: instance
            .extra_launch_args
            .clone()
            .unwrap_or_else(|| settings.extra_launch_args.clone()),

        custom_env_vars: instance
            .custom_env_vars
            .clone()
            .unwrap_or_else(|| settings.custom_env_vars.clone()),

        memory: instance.memory.unwrap_or(settings.memory),

        game_resolution: instance.game_resolution.unwrap_or(settings.game_resolution),

        hooks: Hooks {
            pre_launch: instance
                .hooks
                .pre_launch
                .clone()
                .or_else(|| settings.hooks.pre_launch.clone()),

            wrapper: instance
                .hooks
                .wrapper
                .clone()
                .or_else(|| settings.hooks.wrapper.clone()),

            post_exit: instance
                .hooks
                .post_exit
                .clone()
                .or_else(|| settings.hooks.post_exit.clone()),
        },
    }
}
