#![allow(dead_code)]
use serde_json::json;
use serializable_error::ToSerializableError;
use serializable_error_derive::SerializeError;

#[derive(Debug, thiserror::Error, SerializeError)]
enum TestErrorWithFields {
    #[error("Simple variant")]
    VariantWithFields { field1: String, field2: String },
}

fn main() {
    let err = TestErrorWithFields::VariantWithFields {
        field1: "field1".to_owned(),
        field2: "field2".to_owned(),
    };
    let serial = err.to_serializable();

    assert_eq!(serial.code, "variantWithFields");
    assert_eq!(
        serial.fields,
        Some(json!({
            "field1": "field1",
            "field2": "field2"
        }))
    );
}
