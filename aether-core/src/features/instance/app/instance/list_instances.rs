use std::sync::Arc;

use crate::features::instance::{Instance, InstanceStorage};

pub struct ListInstancesUseCase<IS> {
    instance_storage: Arc<IS>,
}

impl<IS: InstanceStorage> ListInstancesUseCase<IS> {
    pub fn new(instance_storage: Arc<IS>) -> Self {
        Self { instance_storage }
    }

    pub async fn execute(&self) -> crate::Result<Vec<Instance>> {
        Ok(self.instance_storage.list().await?)
    }
}
