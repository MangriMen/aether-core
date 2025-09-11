use std::sync::Arc;

use extism::{FromBytes, ToBytes};
use extism_convert::{encoding, Json};
use serde::{Deserialize, Serialize};

use crate::features::settings::{
    DefaultInstanceSettings, DefaultInstanceSettingsStorage, MemorySettings, SettingsError,
    WindowSize,
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
        extra_launch_args,
        custom_env_vars,
        memory,
        game_resolution,
    } = edit_settings;

    if let Some(extra_launch_args) = extra_launch_args {
        settings.extra_launch_args = extra_launch_args.clone();
    }

    if let Some(custom_env_vars) = custom_env_vars {
        settings.custom_env_vars = custom_env_vars.clone();
    }

    if let Some(memory) = memory {
        settings.memory = *memory;
    }

    if let Some(game_resolution) = game_resolution {
        settings.game_resolution = *game_resolution;
    }
}

#[derive(Debug, Serialize, Deserialize, FromBytes, ToBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct EditDefaultInstanceSettings {
    pub extra_launch_args: Option<Vec<String>>,
    pub custom_env_vars: Option<Vec<(String, String)>>,
    pub memory: Option<MemorySettings>,
    pub game_resolution: Option<WindowSize>,
}
