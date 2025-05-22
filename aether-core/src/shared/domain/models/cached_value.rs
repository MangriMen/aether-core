use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::shared::Cacheable;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedValue<T> {
    pub value: T,
    pub updated_at: SystemTime,
}

impl<T> CachedValue<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            updated_at: SystemTime::now(),
        }
    }
}

impl<T> Cacheable for CachedValue<T> {
    fn updated_at(&self) -> SystemTime {
        self.updated_at
    }
}
