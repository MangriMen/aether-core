use std::sync::Arc;

use crate::features::instance::{Instance, InstanceStorage};

pub struct GetInstanceUseCase<IS> {
    instance_storage: Arc<IS>,
}

impl<IS: InstanceStorage> GetInstanceUseCase<IS> {
    pub fn new(instance_storage: Arc<IS>) -> Self {
        Self { instance_storage }
    }

    pub async fn execute(&self, instance_id: String) -> crate::Result<Instance> {
        Ok(self.instance_storage.get(&instance_id).await?)
    }
}
