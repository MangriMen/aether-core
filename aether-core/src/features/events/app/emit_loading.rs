use tauri::Emitter;
use uuid::Uuid;

use crate::features::events::{
    EventError, EventState, LauncherEvent, LoadingBar, LoadingBarId, LoadingBarType, LoadingPayload,
};

pub async fn init_loading_unsafe(
    bar_type: LoadingBarType,
    total: f64,
    title: &str,
) -> crate::Result<LoadingBarId> {
    let event_state = EventState::get()?;
    let key = LoadingBarId(Uuid::new_v4());

    event_state.loading_bars.insert(
        key.0,
        LoadingBar {
            loading_bar_uuid: key.0,
            message: title.to_string(),
            total,
            current: 0.0,
            last_sent: 0.0,
            bar_type,
        },
    );
    // attempt an initial loading_emit event to the frontend
    emit_loading(&key, 0.0, None).await?;

    Ok(key)
}

pub async fn init_loading(
    bar_type: LoadingBarType,
    total: f64,
    title: &str,
) -> crate::Result<LoadingBarId> {
    let key = init_loading_unsafe(bar_type, total, title).await?;
    Ok(key)
}

pub async fn init_or_edit_loading(
    id: Option<LoadingBarId>,
    bar_type: LoadingBarType,
    total: f64,
    title: &str,
) -> crate::Result<LoadingBarId> {
    if let Some(id) = id {
        edit_loading(&id, bar_type, total, title).await?;

        Ok(id)
    } else {
        init_loading(bar_type, total, title).await
    }
}

pub async fn emit_loading(
    key: &LoadingBarId,
    increment_frac: f64,
    message: Option<&str>,
) -> crate::Result<()> {
    let event_state = EventState::get()?;

    let mut loading_bar = match event_state.loading_bars.get_mut(&key.0) {
        Some(f) => f,
        None => {
            return Err(EventError::NoLoadingBar(key.0).into());
        }
    };

    // Tick up loading bar
    loading_bar.current += increment_frac;
    let display_frac = loading_bar.current / loading_bar.total;
    let opt_display_frac = if display_frac >= 1.0 {
        None // by convention, when its done, we submit None
             // any further updates will be ignored (also sending None)
    } else {
        Some(display_frac)
    };

    if f64::abs(display_frac - loading_bar.last_sent) > 0.005 {
        if let Some(app_handle) = &event_state.app {
            app_handle
                .emit(
                    LauncherEvent::Loading.as_str(),
                    LoadingPayload {
                        fraction: opt_display_frac,
                        message: message.unwrap_or(&loading_bar.message).to_string(),
                        event: loading_bar.bar_type.clone(),
                        loader_uuid: loading_bar.loading_bar_uuid,
                    },
                )
                .map_err(|e| EventError::SerializeError(anyhow::Error::from(e)))?;
        }

        loading_bar.last_sent = display_frac;
    }

    Ok(())
}

// Edits a loading bar's type
// This also resets the bar's current progress to 0
pub async fn edit_loading(
    id: &LoadingBarId,
    bar_type: LoadingBarType,
    total: f64,
    title: &str,
) -> crate::Result<()> {
    let event_state = EventState::get()?;

    if let Some(mut bar) = event_state.loading_bars.get_mut(&id.0) {
        bar.bar_type = bar_type;
        bar.total = total;
        bar.message = title.to_string();
        bar.current = 0.0;
        bar.last_sent = 0.0;
    };

    emit_loading(id, 0.0, None).await?;

    Ok(())
}
