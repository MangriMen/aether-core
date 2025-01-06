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

pub type CredentialsData = Vec<Credentials>;

impl Credentials {
    async fn get_credentials_file(state: &LauncherState) -> crate::Result<PathBuf> {
        let credentials_file = state.locations.settings_dir.join("credentials.json");

        if !credentials_file.exists() {
            write_json_async(&credentials_file, CredentialsData::new()).await?
        }

        Ok(credentials_file)
    }

    pub async fn get(state: &LauncherState) -> crate::Result<CredentialsData> {
        let state_file = &Credentials::get_credentials_file(&state).await?;
        read_json_async::<CredentialsData>(&state_file).await
    }

    pub async fn save(
        state: &LauncherState,
        credentials_state: &CredentialsData,
    ) -> crate::Result<()> {
        let state_file = &Credentials::get_credentials_file(&state).await?;
        write_json_async(&state_file, credentials_state).await
    }

    pub async fn get_active(state: &LauncherState) -> crate::Result<Option<Credentials>> {
        let credentials = Credentials::get(&state).await?;

        if credentials.is_empty() {
            return Ok(None);
        }

        Ok(credentials.iter().find(|x| x.active).cloned())
    }

    #[inline]
    async fn set_active_internal(
        credentials: &CredentialsData,
        id: &Uuid,
    ) -> crate::Result<CredentialsData> {
        let mut cloned_credentials = credentials.clone();

        if cloned_credentials.is_empty() {
            return Ok(cloned_credentials);
        }

        let mut prev_active = None;
        let mut new_active = None;

        for credential in &mut cloned_credentials {
            if credential.active {
                prev_active = Some(credential);
            } else if credential.id == *id {
                new_active = Some(credential);
            }
        }

        if let Some(new_active) = new_active {
            if let Some(prev_active) = prev_active {
                prev_active.active = false;
            }
            new_active.active = true;
        } else {
            return Err(crate::ErrorKind::NoCredentialsError.as_error());
        }

        Ok(cloned_credentials)
    }

    pub async fn set_active(state: &LauncherState, id: &Uuid) -> crate::Result<()> {
        let credentials = Credentials::get(&state).await?;
        let updated_credentials = Self::set_active_internal(&credentials, id).await?;
        Self::save(state, &updated_credentials).await
    }

    pub async fn remove(state: &LauncherState, id: &Uuid) -> crate::Result<()> {
        let mut credentials = Credentials::get(&state).await?;

        let mut need_to_set_active = false;
        credentials.retain(|x| {
            let need_retain = x.id != *id;

            if !need_retain && x.active {
                need_to_set_active = true
            }

            return need_retain;
        });

        if need_to_set_active {
            if let Some(first) = credentials.first_mut() {
                first.active = true;
            };
        }

        Self::save(state, &credentials).await?;

        Ok(())
    }

    pub async fn create_offline_account(
        state: &LauncherState,
        username: &str,
    ) -> crate::Result<()> {
        let mut credentials = Credentials::get(&state).await?;

        let new_credentials_id = Uuid::new_v4();

        credentials.push(Credentials {
            id: new_credentials_id,
            username: username.to_string(),
            access_token: String::new(),
            refresh_token: String::new(),
            expires: Utc::now(),
            active: false,
        });

        let updated_credentials =
            Self::set_active_internal(&credentials, &new_credentials_id).await?;
        Self::save(state, &updated_credentials).await?;

        Ok(())
    }
}
