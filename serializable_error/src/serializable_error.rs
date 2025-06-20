use std::borrow::Cow;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SerializableError {
    pub code: Cow<'static, str>,
    pub fields: Option<serde_json::Value>,
    pub message: String,
}

pub trait ToSerializableError {
    fn to_serializable(&self) -> SerializableError;
}
