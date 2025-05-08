use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use extism::{FromBytes, ToBytes};
use extism_convert::Json;
use serde::{Deserialize, Serialize};

use crate::{
    features::{
        instance::{Instance, InstanceStorage},
        settings::{MemorySettings, WindowSize},
    },
    shared::domain::AsyncUseCaseWithInputAndError,
};

#[derive(Debug, Serialize, Deserialize, FromBytes, ToBytes)]
#[encoding(Json)]
#[serde(rename_all = "camelCase")]
pub struct EditInstance {
    pub name: Option<String>,
    pub java_path: Option<Option<String>>,
    pub extra_launch_args: Option<Option<Vec<String>>>,
    pub custom_env_vars: Option<Option<Vec<(String, String)>>>,
    pub memory: Option<Option<MemorySettings>>,
    pub game_resolution: Option<Option<WindowSize>>,
}

pub struct EditInstanceUseCase<IS> {
    instance_storage: Arc<IS>,
}

impl<IS> EditInstanceUseCase<IS> {
    pub fn new(instance_storage: Arc<IS>) -> Self {
        Self { instance_storage }
    }
}

#[async_trait]
impl<IS> AsyncUseCaseWithInputAndError for EditInstanceUseCase<IS>
where
    IS: InstanceStorage + Send + Sync,
{
    type Input = (String, EditInstance);
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let (id, edit_instance) = input;
        validate_edit(&edit_instance)?;
        let mut instance = self.instance_storage.get(&id).await?;
        apply_edit_changes(&mut instance, &edit_instance);
        Ok(self.instance_storage.upsert(&instance).await?)
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

fn validate_edit(edit: &EditInstance) -> crate::Result<()> {
    if let Some(name) = &edit.name {
        validate_name(name)?;
    }

    Ok(())
}

fn validate_name(name: &str) -> crate::Result<()> {
    if name.is_empty() {
        return Err(crate::ErrorKind::OtherError("Name cannot be empty".to_string()).into());
    }
    Ok(())
}
