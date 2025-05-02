use std::collections::HashMap;

use super::ContentProvider;

#[derive(Default)]
pub struct ProviderRegistry<CP> {
    providers: HashMap<String, CP>,
}

impl<CP> ProviderRegistry<CP>
where
    CP: ContentProvider + Send + Sync,
{
    pub fn new(providers: HashMap<String, CP>) -> Self {
        Self { providers }
    }

    pub fn get(&self, provider_id: &str) -> crate::Result<&CP> {
        self.providers.get(provider_id).ok_or_else(|| {
            crate::ErrorKind::ContentProviderNotFound {
                provider: provider_id.to_string(),
            }
            .as_error()
        })
    }

    pub fn list(&self) -> HashMap<String, String> {
        self.providers
            .keys()
            .map(|k| (k.to_uppercase(), k.to_string()))
            .collect()
    }

    pub fn register(&mut self, id: String, provider: CP) {
        self.providers.insert(id, provider);
    }

    pub fn unregister(&mut self, id: &str) {
        self.providers.remove(id);
    }
}
