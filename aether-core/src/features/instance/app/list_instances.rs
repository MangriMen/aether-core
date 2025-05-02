use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::instance::{Instance, InstanceStorage},
    shared::domain::AsyncUseCaseWithError,
};

pub struct ListInstancesUseCase<IS> {
    instance_storage: Arc<IS>,
}

impl<IS> ListInstancesUseCase<IS> {
    pub fn new(instance_storage: Arc<IS>) -> Self {
        Self { instance_storage }
    }
}

#[async_trait]
impl<IS> AsyncUseCaseWithError for ListInstancesUseCase<IS>
where
    IS: InstanceStorage + Send + Sync,
{
    type Output = Vec<Instance>;
    type Error = crate::Error;

    async fn execute(&self) -> Result<Self::Output, Self::Error> {
        Ok(self.instance_storage.list().await?)
    }
}
