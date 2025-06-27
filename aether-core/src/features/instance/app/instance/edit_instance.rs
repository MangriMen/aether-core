use std::sync::Arc;

use chrono::Utc;
use extism::{FromBytes, ToBytes};
use extism_convert::Json;
use serde::{Deserialize, Serialize};

use crate::features::{
    instance::{Instance, InstanceError, InstanceStorage},
    settings::{MemorySettings, WindowSize},
};

#[derive(Debug, Serialize, Deserialize, FromBytes, ToBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct EditInstance {
    pub name: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub java_path: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub extra_launch_args: Option<Option<Vec<String>>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub custom_env_vars: Option<Option<Vec<(String, String)>>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub memory: Option<Option<MemorySettings>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "::serde_with::rust::double_option"
    )]
    pub game_resolution: Option<Option<WindowSize>>,
}

pub struct EditInstanceUseCase<IS> {
    instance_storage: Arc<IS>,
}

impl<IS: InstanceStorage> EditInstanceUseCase<IS> {
    pub fn new(instance_storage: Arc<IS>) -> Self {
        Self { instance_storage }
    }

    pub async fn execute(
        &self,
        instance_id: String,
        edit_instance: EditInstance,
    ) -> Result<(), InstanceError> {
        validate_edit(&edit_instance)?;
        let mut instance = self.instance_storage.get(&instance_id).await?;
        apply_edit_changes(&mut instance, &edit_instance);
        self.instance_storage.upsert(&instance).await
    }
}

fn apply_edit_changes(instance: &mut Instance, edit_instance: &EditInstance) {
    let EditInstance {
        name,
        java_path,
        extra_launch_args,
        custom_env_vars,
        memory,
        game_resolution,
    } = edit_instance;

    if let Some(name) = name {
        instance.name = name.clone();
    }

    if let Some(java_path) = java_path {
        instance.java_path = java_path.clone();
    }

    if let Some(args) = extra_launch_args {
        instance.extra_launch_args = args.clone();
    }

    if let Some(vars) = custom_env_vars {
        instance.custom_env_vars = vars.clone();
    }

    if let Some(mem) = memory {
        instance.memory = *mem;
    }

    if let Some(res) = game_resolution {
        instance.game_resolution = *res;
    }

    instance.modified = Utc::now();
}

fn validate_edit(edit: &EditInstance) -> Result<(), InstanceError> {
    if let Some(name) = &edit.name {
        validate_name(name)?;
    }

    Ok(())
}

fn validate_name(name: &str) -> Result<(), InstanceError> {
    if name.is_empty() {
        return Err(InstanceError::ValidationError {
            field: "name".to_owned(),
            reason: "name cannot be empty".to_owned(),
        });
    }
    Ok(())
}
