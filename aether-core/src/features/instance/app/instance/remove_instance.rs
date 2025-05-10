use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::instance::{InstanceStorage, InstanceWatcherService},
    shared::domain::AsyncUseCaseWithInputAndError,
};

pub struct RemoveInstanceUseCase<IS, IWS> {
    instance_storage: Arc<IS>,
    instance_watcher_service: Arc<IWS>,
}

impl<IM, IWS> RemoveInstanceUseCase<IM, IWS> {
    pub fn new(instance_storage: Arc<IM>, instance_watcher_service: Arc<IWS>) -> Self {
        Self {
            instance_storage,
            instance_watcher_service,
        }
    }
}

#[async_trait]
impl<IS: InstanceStorage, IWS: InstanceWatcherService> AsyncUseCaseWithInputAndError
    for RemoveInstanceUseCase<IS, IWS>
{
    type Input = String;
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, id: Self::Input) -> Result<Self::Output, Self::Error> {
        self.instance_watcher_service.unwatch_instance(&id).await?;
        self.instance_storage.remove(&id).await?;
        Ok(())
    }
}
