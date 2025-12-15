use aether_core_plugin_api::v0::ContentFileDto;

use crate::features::instance::ContentFile;

impl From<ContentFile> for ContentFileDto {
    fn from(value: ContentFile) -> Self {
        Self {
            content_path: value.content_path,
            content_type: value.content_type.into(),
            disabled: value.disabled,
            filename: value.filename,
            hash: value.hash,
            instance_relative_path: value.instance_relative_path,
            name: value.name,
            size: value.size,
            update: value.update,
        }
    }
}
