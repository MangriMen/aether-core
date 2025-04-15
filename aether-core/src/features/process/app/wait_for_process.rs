use uuid::Uuid;

use crate::features::process::ProcessManager;

pub async fn wait_for_process<M>(process_manager: &M, id: Uuid) -> crate::Result<()>
where
    M: ProcessManager + ?Sized,
{
    process_manager.wait_for(id).await
}
