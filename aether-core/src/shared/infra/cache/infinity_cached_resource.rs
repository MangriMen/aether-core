use std::time::Duration;

use tracing::debug;

use crate::shared::{Cache, CacheKey};

pub struct InfinityCachedResource<C: Cache> {
    pub cache: C,
}

impl<C: Cache> InfinityCachedResource<C> {
    pub fn new(cache: C) -> Self {
        Self { cache }
    }

    pub async fn get_cached<T, Fut, KeyFn, ContextFn, E>(
        &self,
        key_fn: KeyFn,
        fetch_fn: Fut,
        context_fn: ContextFn,
        force: bool,
    ) -> Result<T, E>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<T, E>>,
        KeyFn: Fn() -> CacheKey<T>,
        ContextFn: Fn() -> String,
    {
        let key = key_fn();

        if !force {
            debug!("Get {} from cache", context_fn());
            if let Some(value) = self.cache.get::<T>(&key).await {
                return Ok(value);
            }
        }

        debug!("Fetching {}", context_fn());
        let value = fetch_fn.await?;

        self.cache.set(&key, &value, Duration::ZERO).await;

        Ok(value)
    }

    pub async fn pick_cached<T, KeyFn>(&self, key_fn: KeyFn) -> bool
    where
        T: Send + Sync + 'static,
        KeyFn: Fn() -> CacheKey<T>,
    {
        let key = key_fn();

        self.cache.exists(&key).await
    }
}
