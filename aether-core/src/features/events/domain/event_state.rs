use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::OnceCell;
use uuid::Uuid;

use super::{EventError, ProgressBar};

static EVENT_STATE: OnceCell<Arc<EventState>> = OnceCell::const_new();
pub struct EventState {
    // pub event_emitter: Option<&dyn Fn(&str, S) -> anyhow::Result<()>>,
    pub app: Option<tauri::AppHandle>,
    pub progress_bars: DashMap<Uuid, ProgressBar>,
}

impl EventState {
    pub async fn init() -> crate::Result<Arc<Self>> {
        EVENT_STATE
            .get_or_try_init(|| async {
                Ok(Arc::new(Self {
                    app: None,
                    progress_bars: DashMap::new(),
                }))
            })
            .await
            .cloned()
    }

    // TODO: migrate to event emitter
    // pub async fn init_with_emitter<S: serde::Serialize + Clone>(
    //     emit_fn: &dyn Fn(&str, S) -> anyhow::Result<()>,
    // ) -> anyhow::Result<Arc<Self>> {
    //     EVENT_STATE
    //         .get_or_try_init(|| async {
    //             Ok(Arc::new(Self {
    //                 event_emitter: Some(emit_fn),
    //                 loading_bars: DashMap::new(),
    //             }))
    //         })
    //         .await
    //         .cloned()
    // }

    pub async fn init_with_app(app: tauri::AppHandle) -> crate::Result<Arc<Self>> {
        EVENT_STATE
            .get_or_try_init(|| async {
                Ok(Arc::new(Self {
                    app: Some(app),
                    progress_bars: DashMap::new(),
                }))
            })
            .await
            .cloned()
    }

    pub fn get() -> crate::Result<Arc<Self>> {
        Ok(EVENT_STATE.get().ok_or(EventError::NotInitialized)?.clone())
    }

    // Values provided should not be used directly, as they are clones and are not guaranteed to be up-to-date
    pub async fn list_progress_bars() -> crate::Result<DashMap<Uuid, ProgressBar>> {
        let value = Self::get()?;
        Ok(value.progress_bars.clone())
    }
}
