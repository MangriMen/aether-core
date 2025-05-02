use std::sync::Arc;

use async_trait::async_trait;

use crate::{features::instance::InstanceManager, shared::domain::AsyncUseCaseWithInputAndError};

pub struct RemoveInstanceUseCase<IM: InstanceManager> {
    instance_manager: Arc<IM>,
}

impl<IM: InstanceManager> RemoveInstanceUseCase<IM> {
    pub fn new(instance_manager: Arc<IM>) -> Self {
        Self { instance_manager }
    }
}

#[async_trait]
impl<IM> AsyncUseCaseWithInputAndError for RemoveInstanceUseCase<IM>
where
    IM: InstanceManager + Send + Sync,
{
    type Input = String;
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, id: Self::Input) -> Result<Self::Output, Self::Error> {
        self.instance_manager.remove(&id).await
    }
}
