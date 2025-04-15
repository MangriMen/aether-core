use crate::features::process::{MinecraftProcessMetadata, ProcessManager};

pub fn get_process_by_instance_id<M>(process_manager: &M, id: &str) -> Vec<MinecraftProcessMetadata>
where
    M: ProcessManager + ?Sized,
{
    process_manager
        .list()
        .iter()
        .filter(|x| x.id == id)
        .cloned()
        .collect()
}
