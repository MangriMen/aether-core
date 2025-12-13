use crate::features::java::Java;
use crate::features::plugins::infra::plugin_api::v0::JavaDto;

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
