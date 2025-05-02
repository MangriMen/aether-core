use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::{
        events::{emit_instance, InstancePayloadType},
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

pub struct RemoveContentUseCase<PS: PackStorage> {
    pack_storage: Arc<PS>,
    location_info: Arc<LocationInfo>,
}

impl<PS: PackStorage> RemoveContentUseCase<PS> {
    pub fn new(pack_storage: Arc<PS>, location_info: Arc<LocationInfo>) -> Self {
        Self {
            pack_storage,
            location_info,
        }
    }
}

#[async_trait]
impl<PS> AsyncUseCaseWithInputAndError for RemoveContentUseCase<PS>
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

        emit_instance(&instance_id, InstancePayloadType::Edited).await?;

        Ok(())
    }
}
