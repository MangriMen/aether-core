use crate::features::process::{MinecraftProcessMetadata, ProcessManager};

pub fn get_process_by_instance_id(
    process_manager: &ProcessManager,
    id: &str,
) -> Vec<MinecraftProcessMetadata> {
    process_manager
        .list()
        .iter()
        .filter(|x| x.id == id)
        .cloned()
        .collect()
}
