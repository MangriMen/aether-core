use std::fmt;

use async_trait::async_trait;

#[async_trait]
pub trait InstancePlugin: Send + Sync {
    fn get_id(&self) -> String;
    fn get_name(&self) -> String;
    fn get_description(&self) -> String;

    async fn initialize(&self) -> crate::Result<()>;
    async fn call(&self, data: &str) -> crate::Result<()>;
    async fn destroy(&self) -> crate::Result<()>;
}

impl fmt::Debug for dyn InstancePlugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "InstancePlugin(name: {}, description: {})",
            self.get_name(),
            self.get_description()
        )
    }
}
