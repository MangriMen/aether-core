use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::OnceCell;
use uuid::Uuid;

use super::LoadingBar;

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Event state was not properly initialized")]
    NotInitialized,

    #[error("Non-existent loading bar of key: {0}")]
    NoLoadingBar(Uuid),

    #[error("Failed to sent event")]
    SerializeError(anyhow::Error),
}

static EVENT_STATE: OnceCell<Arc<EventState>> = OnceCell::const_new();
pub struct EventState {
    // pub event_emitter: Option<&dyn Fn(&str, S) -> anyhow::Result<()>>,
    pub app: Option<tauri::AppHandle>,
    pub loading_bars: DashMap<Uuid, LoadingBar>,
}

impl EventState {
    pub async fn init() -> anyhow::Result<Arc<Self>> {
        EVENT_STATE
            .get_or_try_init(|| async {
                Ok(Arc::new(Self {
                    app: None,
                    loading_bars: DashMap::new(),
                }))
            })
            .await
            .cloned()
    }

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

    pub async fn init_with_app(app: tauri::AppHandle) -> anyhow::Result<Arc<Self>> {
        EVENT_STATE
            .get_or_try_init(|| async {
                Ok(Arc::new(Self {
                    app: Some(app),
                    loading_bars: DashMap::new(),
                }))
            })
            .await
            .cloned()
    }

    pub fn get() -> anyhow::Result<Arc<Self>> {
        Ok(EVENT_STATE.get().ok_or(EventError::NotInitialized)?.clone())
    }

    // Values provided should not be used directly, as they are clones and are not guaranteed to be up-to-date
    pub async fn list_progress_bars() -> anyhow::Result<DashMap<Uuid, LoadingBar>> {
        let value = Self::get()?;
        Ok(value.loading_bars.clone())
    }
}
