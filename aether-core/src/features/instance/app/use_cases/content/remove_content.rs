use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    features::{
        events::{EventEmitter, EventEmitterExt, InstanceEventType},
        instance::{InstanceError, PackStorage},
        settings::LocationInfo,
    },
    shared::remove_file,
};

pub struct RemoveContent {
    instance_id: String,
    content_paths: Vec<String>,
}

impl RemoveContent {
    pub fn single(instance_id: String, content_path: String) -> Self {
        Self {
            instance_id,
            content_paths: vec![content_path],
        }
    }

    pub fn multiple(instance_id: String, content_paths: Vec<String>) -> Self {
        Self {
            instance_id,
            content_paths,
        }
    }
}

pub struct RemoveContentUseCase<E, PS: PackStorage> {
    event_emitter: Arc<E>,
    pack_storage: Arc<PS>,
    location_info: Arc<LocationInfo>,
}

impl<E: EventEmitter, PS: PackStorage> RemoveContentUseCase<E, PS> {
    pub fn new(
        event_emitter: Arc<E>,
        pack_storage: Arc<PS>,
        location_info: Arc<LocationInfo>,
    ) -> Self {
        Self {
            event_emitter,
            pack_storage,
            location_info,
        }
    }

    pub fn get_real_content_path(&self, instance_dir: &Path, content_path: &str) -> PathBuf {
        let mut absolute_path = instance_dir.join(content_path);

        if !absolute_path.exists() {
            absolute_path.add_extension("disabled");
        }

        absolute_path
    }

    pub async fn execute(&self, input: RemoveContent) -> Result<(), InstanceError> {
        let RemoveContent {
            instance_id,
            content_paths,
        } = input;

        let instance_dir = self.location_info.instance_dir(&instance_id);

        for content_path in content_paths.iter() {
            let real_content_path = self.get_real_content_path(&instance_dir, content_path);

            remove_file(real_content_path).await?;
        }

        self.pack_storage
            .remove_pack_file_many(&instance_id, content_paths.as_slice())
            .await?;

        self.event_emitter
            .emit_instance_safe(instance_id.to_string(), InstanceEventType::Edited)
            .await;

        Ok(())
    }
}
