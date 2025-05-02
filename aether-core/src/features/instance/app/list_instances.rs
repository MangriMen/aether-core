use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    features::instance::{Instance, InstanceManager},
    shared::domain::AsyncUseCaseWithError,
};

pub struct ListInstancesUseCase<IM: InstanceManager> {
    instance_manager: Arc<IM>,
}

impl<IM: InstanceManager> ListInstancesUseCase<IM> {
    pub fn new(instance_manager: Arc<IM>) -> Self {
        Self { instance_manager }
    }
}

#[async_trait]
impl<IM> AsyncUseCaseWithError for ListInstancesUseCase<IM>
where
    IM: InstanceManager + Send + Sync,
{
    type Output = Vec<Instance>;
    type Error = crate::Error;

    async fn execute(&self) -> Result<Self::Output, Self::Error> {
        self.instance_manager.list().await
    }
}
