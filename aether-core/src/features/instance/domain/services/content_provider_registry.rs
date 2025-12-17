use std::collections::HashMap;

use crate::features::instance::{ContentProvider, InstanceError};

#[derive(Default)]
pub struct ContentProviderRegistry<CP> {
    providers: HashMap<String, CP>,
}

impl<CP: ContentProvider> ContentProviderRegistry<CP> {
    pub fn new(providers: HashMap<String, CP>) -> Self {
        Self { providers }
    }

    pub fn get(&self, provider_id: &str) -> Result<&CP, InstanceError> {
        self.providers
            .get(provider_id)
            .ok_or(InstanceError::ContentProviderNotFound {
                provider_id: provider_id.to_string(),
            })
    }

    pub fn list(&self) -> HashMap<String, String> {
        self.providers
            .iter()
            .map(|(k, v)| (k.to_string(), v.get_name()))
            .collect()
    }

    pub fn register(&mut self, id: String, provider: CP) {
        self.providers.insert(id, provider);
    }

    pub fn unregister(&mut self, id: &str) {
        self.providers.remove(id);
    }
}
