use dashmap::DashMap;
use uuid::Uuid;

use dashmap::mapref::one::Ref as DashMapRef;

use crate::features::events::{
    progress_bar_error::ProgressBarStorageError, ProgressBar, ProgressBarStorage,
};

#[derive(Debug, Default)]
pub struct InMemoryProgressBarStorage {
    progress_bars: DashMap<Uuid, ProgressBar>,
}

impl ProgressBarStorage for InMemoryProgressBarStorage {
    fn insert(
        &self,
        progress_bar_id: Uuid,
        progress_bar: ProgressBar,
    ) -> Result<(), ProgressBarStorageError> {
        if self.progress_bars.contains_key(&progress_bar_id) {
            return Err(ProgressBarStorageError::AlreadyExists { progress_bar_id });
        }
        self.progress_bars.insert(progress_bar_id, progress_bar);

        Ok(())
    }

    fn list(&self) -> DashMap<Uuid, ProgressBar> {
        self.progress_bars.clone()
    }

    fn get(
        &self,
        progress_bar_id: Uuid,
    ) -> Result<DashMapRef<'_, Uuid, ProgressBar>, ProgressBarStorageError> {
        self.progress_bars
            .get(&progress_bar_id)
            .ok_or(ProgressBarStorageError::NotFound { progress_bar_id })
    }

    fn upsert(
        &self,
        progress_bar_id: Uuid,
        progress_bar: ProgressBar,
    ) -> Result<(), ProgressBarStorageError> {
        self.progress_bars.insert(progress_bar_id, progress_bar);

        Ok(())
    }

    fn remove(
        &self,
        progress_bar_id: Uuid,
    ) -> Result<Option<(Uuid, ProgressBar)>, ProgressBarStorageError> {
        Ok(self.progress_bars.remove(&progress_bar_id))
    }
}
