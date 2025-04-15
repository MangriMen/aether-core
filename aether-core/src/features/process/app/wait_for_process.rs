use uuid::Uuid;

use crate::features::process::ProcessManager;

pub async fn wait_for_process(process_manager: &ProcessManager, id: Uuid) -> crate::Result<()> {
    process_manager.wait_for(id).await
}
