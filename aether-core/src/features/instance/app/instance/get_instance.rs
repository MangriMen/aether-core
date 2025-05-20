use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::instance::{Instance, InstanceStorage},
    shared::domain::AsyncUseCaseWithInputAndError,
};

pub struct GetInstanceUseCase<IS> {
    instance_storage: Arc<IS>,
}

impl<IS> GetInstanceUseCase<IS> {
    pub fn new(instance_storage: Arc<IS>) -> Self {
        Self { instance_storage }
    }
}

#[async_trait]
impl<IS> AsyncUseCaseWithInputAndError for GetInstanceUseCase<IS>
where
    IS: InstanceStorage + Send + Sync,
{
    type Input = String;
    type Output = Instance;
    type Error = crate::Error;

    async fn execute(&self, id: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(self.instance_storage.get(&id).await?)
    }
}
