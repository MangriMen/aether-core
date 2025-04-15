use uuid::Uuid;

use crate::features::process::ProcessManager;

pub async fn kill_process<M>(process_manager: &M, id: Uuid) -> crate::Result<()>
where
    M: ProcessManager + ?Sized,
{
    process_manager.kill(id).await
}
