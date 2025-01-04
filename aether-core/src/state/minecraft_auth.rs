use std::path::PathBuf;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::utils::io::{read_json_async, write_json_async};

use super::LauncherState;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Credentials {
    pub id: Uuid,
    pub username: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires: DateTime<Utc>,
    pub active: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CredentialsState {
    pub default: Option<Uuid>,
    pub credentials: Vec<Credentials>,
}

impl Credentials {
    async fn get_credentials_state_file(state: &LauncherState) -> crate::Result<PathBuf> {
        let credentials_file = state.locations.settings_dir.join("credentials.json");

        if !credentials_file.exists() {
            write_json_async(
                &credentials_file,
                &CredentialsState {
                    default: None,
                    credentials: Vec::new(),
                },
            )
            .await?
        }

        Ok(credentials_file)
    }

    pub async fn get_state(state: &LauncherState) -> crate::Result<CredentialsState> {
        let state_file = &Credentials::get_credentials_state_file(&state).await?;
        read_json_async::<CredentialsState>(&state_file).await
    }

    pub async fn save_state(
        state: &LauncherState,
        credentials_state: &CredentialsState,
    ) -> crate::Result<()> {
        let state_file = &Credentials::get_credentials_state_file(&state).await?;
        write_json_async(&state_file, credentials_state).await
    }

    pub async fn get_default(state: &LauncherState) -> crate::Result<Option<Credentials>> {
        let credentials_state = Credentials::get_state(&state).await?;

        if credentials_state.credentials.is_empty() {
            return Ok(None);
        }

        if let Some(default) = credentials_state.default {
            return Ok(credentials_state
                .credentials
                .iter()
                .find(|x| x.id == default)
                .cloned());
        } else {
            return Ok(None);
        }
    }

    pub async fn create_offline_account(
        state: &LauncherState,
        username: &str,
    ) -> crate::Result<()> {
        let mut credentials_state = Credentials::get_state(&state).await?;

        let new_credentials_id = Uuid::new_v4();

        credentials_state.credentials.push(Credentials {
            id: new_credentials_id,
            username: username.to_string(),
            access_token: String::new(),
            refresh_token: String::new(),
            expires: Utc::now(),
            active: true,
        });

        if credentials_state.default.is_none() {
            credentials_state.default = Some(new_credentials_id);
        }

        Credentials::save_state(state, &credentials_state).await?;

        Ok(())
    }

    pub async fn change_default(state: &LauncherState, id: &Uuid) -> crate::Result<()> {
        let mut credentials_state = Credentials::get_state(&state).await?;

        let credentials = credentials_state.credentials.iter().find(|x| x.id == *id);

        if let Some(credentials) = credentials {
            credentials_state.default = Some(credentials.id);
            Credentials::save_state(state, &credentials_state).await?
        } else {
            return Err(crate::ErrorKind::NoCredentialsError.as_error());
        }

        Ok(())
    }

    pub async fn remove(state: &LauncherState, id: &Uuid) -> crate::Result<()> {
        let mut credentials_state = Credentials::get_state(&state).await?;

        credentials_state.credentials.retain(|x| x.id != *id);

        credentials_state.default = credentials_state
            .credentials
            .last()
            .map_or(None, |x| Some(x.id));

        Credentials::save_state(state, &credentials_state).await?;

        Ok(())
    }
}
