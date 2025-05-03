use dashmap::DashMap;
use uuid::Uuid;

use dashmap::mapref::one::Ref as DashMapRef;

use crate::features::events::{progress_bar_error::ProgressBarStorageError, ProgressBar};

pub trait ProgressBarStorage: Send + Sync {
    fn insert(
        &self,
        progress_bar_id: Uuid,
        progress_bar: ProgressBar,
    ) -> Result<(), ProgressBarStorageError>;
    fn list(&self) -> DashMap<Uuid, ProgressBar>;
    fn get(
        &self,
        progress_bar_id: Uuid,
    ) -> Result<DashMapRef<'_, Uuid, ProgressBar>, ProgressBarStorageError>;
    fn upsert(
        &self,
        progress_bar_id: Uuid,
        progress_bar: ProgressBar,
    ) -> Result<(), ProgressBarStorageError>;
    fn remove(&self, progress_bar_id: Uuid) -> Result<(), ProgressBarStorageError>;
}
