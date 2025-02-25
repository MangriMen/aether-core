use crate::state::ImportHandler;

pub async fn get_import_handlers() -> crate::Result<Vec<ImportHandler>> {
    Ok(vec![ImportHandler {
        pack_type: "packwiz".to_string(),
        title: "Packwiz".to_string(),
        field_label: "Packwiz pack URL or file".to_string(),
        file_name: "Packwiz modpack".to_string(),
        file_extensions: vec!["toml".to_string()],
    }])
}
