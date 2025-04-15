use crate::features::process::{MinecraftProcessMetadata, ProcessManager};

pub fn list_process<M>(process_manager: &M) -> Vec<MinecraftProcessMetadata>
where
    M: ProcessManager + ?Sized,
{
    process_manager.list()
}
