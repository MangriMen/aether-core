use uuid::Uuid;

use crate::features::process::ProcessManager;

pub async fn kill_process(process_manager: &ProcessManager, id: Uuid) -> crate::Result<()> {
    process_manager.kill(id).await
}
