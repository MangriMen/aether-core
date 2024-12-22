use async_trait::async_trait;

use super::InstancePlugin;

pub struct PackwizPlugin {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PackwizPluginData {
    kind: String,
    url: String,
}

#[async_trait]
impl InstancePlugin for PackwizPlugin {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_description(&self) -> String {
        self.description.clone()
    }

    async fn initialize(&self) -> crate::Result<()> {
        Ok(())
    }

    async fn call(&self, data: &str) -> crate::Result<()> {
        let data = serde_json::from_str::<PackwizPluginData>(data)?;

        println!("Downloading {:?}", data.url);

        Ok(())
    }

    async fn destroy(&self) -> crate::Result<()> {
        Ok(())
    }
}
