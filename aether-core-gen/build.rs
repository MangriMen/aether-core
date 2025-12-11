use std::path::{Path, PathBuf};
use std::env;

use register_schema::{SchemaEntry, get_all_schemas};

#[allow(unused_imports)]
use aether_core::*;

fn main() {
    let schemas_dir = get_schemas_dir();
    generate_schemas(&schemas_dir);
}

fn get_schemas_dir() -> PathBuf {
    let workspace_dir = PathBuf::from(env::var("CARGO_WORKSPACE_DIR").unwrap());
    let current_dir = workspace_dir.as_path();

    current_dir.join("spec").join("json-schema")
}

fn generate_schemas(schemas_dir: &Path) {
    let all_schemas = get_all_schemas();

    println!("cargo:warning=all_schemas: {:?}", all_schemas);

    if all_schemas.is_empty() {
        return;
    }

    std::fs::create_dir_all(schemas_dir).unwrap();

    for entry in all_schemas {
        generate_schema(schemas_dir, entry);
    }
}

fn generate_schema(schemas_dir: &Path, registered_schema: &SchemaEntry) {
    let schema = (registered_schema.schema)();
    let json = serde_json::to_string_pretty(&schema).unwrap();

    let schema_file_name = get_schema_filename(registered_schema.name);
    let schema_file_path = schemas_dir.join(schema_file_name);

    std::fs::write(schema_file_path, json).unwrap();
}

fn get_schema_filename(name: &str) -> String {
    format!("{}.schema.json", name)
}
