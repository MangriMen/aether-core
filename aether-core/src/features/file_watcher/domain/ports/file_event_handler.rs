use async_trait::async_trait;

use crate::features::file_watcher::{FileEvent, FileWatcherError};

#[async_trait]
pub trait FileEventHandler: Send + Sync {
    async fn handle_events(
        &self,
        events: crate::Result<Vec<FileEvent>>,
    ) -> Result<(), FileWatcherError>;
}
