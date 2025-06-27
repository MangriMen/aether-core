use std::time::{Duration, SystemTime};

pub trait Cacheable {
    fn updated_at(&self) -> SystemTime;

    fn is_expired(&self, ttl: Duration) -> bool {
        SystemTime::now() > (self.updated_at() + ttl)
    }
}
