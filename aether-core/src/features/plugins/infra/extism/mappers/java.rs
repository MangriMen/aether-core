use aether_core_plugin_api::v0::JavaDto;

use crate::features::java::Java;

impl From<Java> for JavaDto {
    fn from(java: Java) -> Self {
        Self {
            major_version: java.major_version,
            version: java.version,
            architecture: java.architecture,
            path: java.path,
        }
    }
}
