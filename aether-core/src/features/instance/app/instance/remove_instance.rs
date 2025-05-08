use std::sync::Arc;

use async_trait::async_trait;

use crate::{features::instance::InstanceStorage, shared::domain::AsyncUseCaseWithInputAndError};

pub struct RemoveInstanceUseCase<IS> {
    instance_storage: Arc<IS>,
}

impl<IM> RemoveInstanceUseCase<IM> {
    pub fn new(instance_storage: Arc<IM>) -> Self {
        Self { instance_storage }
    }
}

#[async_trait]
impl<IS> AsyncUseCaseWithInputAndError for RemoveInstanceUseCase<IS>
where
    IS: InstanceStorage + Send + Sync,
{
    type Input = String;
    type Output = ();
    type Error = crate::Error;

    async fn execute(&self, id: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(self.instance_storage.remove(&id).await?)
    }
}
