use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        events::{EventEmitter, EventEmitterExt, InstanceEventType},
        instance::PackStorage,
        settings::LocationInfo,
    },
    shared::{domain::AsyncUseCaseWithInputAndError, remove_file},
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
}

#[async_trait]
impl<E: EventEmitter, PS> AsyncUseCaseWithInputAndError for RemoveContentUseCase<E, PS>
where
    PS: PackStorage + Send + Sync,
{
    type Input = RemoveContent;
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let RemoveContent {
            instance_id,
            content_paths,
        } = input;

        let instance_dir = self.location_info.instance_dir(&instance_id);

        for content_path in content_paths.iter() {
            remove_file(instance_dir.join(content_path)).await?;
        }

        self.pack_storage
            .remove_pack_file_many(&instance_id, content_paths.as_slice())
            .await?;

        self.event_emitter
            .emit_instance(instance_id.to_string(), InstanceEventType::Edited)
            .await?;

        Ok(())
    }
}
