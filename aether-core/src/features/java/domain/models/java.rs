use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub struct Java {
    pub major_version: u32,
    pub version: String,
    pub architecture: String,
    pub path: String,
}
