use std::sync::Arc;

use crate::{
    features::{
        events::{EventEmitter, EventEmitterExt, InstanceEventType},
        settings::LocationInfo,
    },
    shared::rename,
};

pub enum ContentStateAction {
    Enable,
    Disable,
}

pub struct ChangeContentState {
    pub instance_id: String,
    pub content_paths: Vec<String>,
    pub action: ContentStateAction,
}

impl ChangeContentState {
    pub fn single(instance_id: String, content_path: String, action: ContentStateAction) -> Self {
        Self {
            instance_id,
            content_paths: vec![content_path],
            action,
        }
    }

    pub fn multiple(
        instance_id: String,
        content_paths: Vec<String>,
        action: ContentStateAction,
    ) -> Self {
        Self {
            instance_id,
            content_paths,
            action,
        }
    }
}

pub struct ChangeContentStateUseCase<E: EventEmitter> {
    event_emitter: Arc<E>,
    location_info: Arc<LocationInfo>,
}

impl<E: EventEmitter> ChangeContentStateUseCase<E> {
    pub fn new(event_emitter: Arc<E>, location_info: Arc<LocationInfo>) -> Self {
        Self {
            event_emitter,
            location_info,
        }
    }

    pub async fn enable_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> crate::Result<()> {
        for content_path in content_paths {
            self.enable(instance_id, content_path).await?;
        }

        self.event_emitter
            .emit_instance(instance_id.to_string(), InstanceEventType::Edited)
            .await?;

        Ok(())
    }

    pub async fn disable_many(
        &self,
        instance_id: &str,
        content_paths: &[String],
    ) -> crate::Result<()> {
        for content_path in content_paths {
            self.disable(instance_id, content_path).await?;
        }

        self.event_emitter
            .emit_instance(instance_id.to_string(), InstanceEventType::Edited)
            .await?;

        Ok(())
    }

    async fn enable(&self, instance_id: &str, content_path: &str) -> crate::Result<Option<String>> {
        if !content_path.ends_with(".disabled") {
            return Ok(None);
        }

        let new_path = content_path.trim_end_matches(".disabled").to_string();
        self.rename_content_file(instance_id, content_path, &new_path)
            .await?;

        Ok(Some(new_path))
    }

    async fn disable(
        &self,
        instance_id: &str,
        content_path: &str,
    ) -> crate::Result<Option<String>> {
        if content_path.ends_with(".disabled") {
            return Ok(None);
        }

        let new_path = format!("{content_path}.disabled");
        self.rename_content_file(instance_id, content_path, &new_path)
            .await?;

        Ok(Some(new_path))
    }

    async fn rename_content_file(
        &self,
        instance_id: &str,
        from: &str,
        to: &str,
    ) -> crate::Result<()> {
        let instance_dir = self.location_info.instance_dir(instance_id);
        Ok(rename(&instance_dir.join(from), &instance_dir.join(to)).await?)
    }

    pub async fn execute(&self, input: ChangeContentState) -> crate::Result<()> {
        let ChangeContentState {
            instance_id,
            content_paths,
            action,
        } = input;

        match action {
            ContentStateAction::Enable => {
                self.enable_many(&instance_id, content_paths.as_slice())
                    .await
            }
            ContentStateAction::Disable => {
                self.disable_many(&instance_id, content_paths.as_slice())
                    .await
            }
        }?;

        Ok(())
    }
}
