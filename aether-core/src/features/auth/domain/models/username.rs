use serde::{Deserialize, Deserializer, Serialize};

use crate::features::auth::AuthError;

#[derive(Debug, Clone, Serialize)]
pub struct Username(String);

const MIN_USERNAME_LENGTH: usize = 3;
const MAX_USERNAME_LENGTH: usize = 16;

impl Username {
    pub fn parse(s: &str) -> Result<Self, AuthError> {
        if s.len() < MIN_USERNAME_LENGTH || s.len() > MAX_USERNAME_LENGTH {
            return Err(AuthError::InvalidUsernameLength {
                min: MIN_USERNAME_LENGTH,
                max: MAX_USERNAME_LENGTH,
            });
        }

        if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(AuthError::InvalidUsernameChars);
        }

        Ok(Self(s.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Username {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Username::parse(&s).map_err(serde::de::Error::custom)
    }
}
