use serde::Serialize;

use crate::features::events::EventError;

pub trait EventEmitter: Send + Sync {
    fn emit<P: Serialize + Clone>(&self, event: &str, payload: P) -> Result<(), EventError>;
}
