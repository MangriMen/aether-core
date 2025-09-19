use std::sync::Arc;

use extism::{FromBytes, ToBytes};
use extism_convert::{encoding, Json};
use serde::{Deserialize, Serialize};

use crate::features::settings::{
    DefaultInstanceSettings, DefaultInstanceSettingsStorage, EditHooks, MemorySettings,
    SettingsError, WindowSize,
};

pub struct EditDefaultInstanceSettingsUseCase<SS: DefaultInstanceSettingsStorage> {
    instance_settings_storage: Arc<SS>,
}

impl<SS: DefaultInstanceSettingsStorage> EditDefaultInstanceSettingsUseCase<SS> {
    pub fn new(instance_settings_storage: Arc<SS>) -> Self {
        Self {
            instance_settings_storage,
        }
    }

    pub async fn execute(
        &self,
        edit_settings: EditDefaultInstanceSettings,
    ) -> Result<DefaultInstanceSettings, SettingsError> {
        let mut settings = self.instance_settings_storage.get().await?;
        apply_edit_changes(&mut settings, &edit_settings);
        self.instance_settings_storage.upsert(settings).await
    }
}

fn apply_edit_changes(
    settings: &mut DefaultInstanceSettings,
    edit_settings: &EditDefaultInstanceSettings,
) {
    let EditDefaultInstanceSettings {
        launch_args,
        env_vars,
        memory,
        game_resolution,
        hooks,
    } = edit_settings;

    if let Some(launch_args) = launch_args {
        settings.launch_args = launch_args.clone();
    }

    if let Some(env_vars) = env_vars {
        settings.env_vars = env_vars.clone();
    }

    if let Some(memory) = memory {
        settings.memory = *memory;
    }

    if let Some(game_resolution) = game_resolution {
        settings.game_resolution = *game_resolution;
    }

    if let Some(hooks) = hooks {
        if let Some(pre_launch) = &hooks.pre_launch {
            settings.hooks.pre_launch = pre_launch.clone();
        }

        if let Some(wrapper) = &hooks.wrapper {
            settings.hooks.wrapper = wrapper.clone();
        }

        if let Some(post_exit) = &hooks.post_exit {
            settings.hooks.post_exit = post_exit.clone();
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromBytes, ToBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct EditDefaultInstanceSettings {
    pub launch_args: Option<Vec<String>>,
    pub env_vars: Option<Vec<(String, String)>>,
    pub memory: Option<MemorySettings>,
    pub game_resolution: Option<WindowSize>,
    pub hooks: Option<EditHooks>,
}
