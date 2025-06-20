#![allow(dead_code)]
use serde_json::json;
use serializable_error::ToSerializableError;
use serializable_error_derive::SerializeError;

#[derive(Debug, thiserror::Error, SerializeError)]
#[serialize_error(name = "test")]
enum TestErrorWithFields {
    #[error("Simple variant")]
    VariantWithFields { field1: String, field2: String },
    #[error(transparent)]
    #[serialize_error]
    NestedErrorVariantWithFields(#[from] NestedErrorWithFields),
}

#[derive(Debug, thiserror::Error, SerializeError)]
enum NestedErrorWithFields {
    #[error("Simple variant")]
    NestedVariantWithFields { field3: String, field4: String },
}

fn main() {
    let err = TestErrorWithFields::NestedErrorVariantWithFields(
        NestedErrorWithFields::NestedVariantWithFields {
            field3: "field3".to_owned(),
            field4: "field4".to_owned(),
        },
    );
    let serial = err.to_serializable();

    assert_eq!(
        serial.code,
        "test.nestedErrorVariantWithFields.nestedVariantWithFields"
    );
    assert_eq!(
        serial.fields,
        Some(json!({
            "field3": "field3",
            "field4": "field4"
        }))
    );
}
