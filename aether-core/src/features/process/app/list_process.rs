use crate::features::process::{MinecraftProcessMetadata, ProcessManager};

pub fn list_process(process_manager: &ProcessManager) -> Vec<MinecraftProcessMetadata> {
    process_manager.list()
}
