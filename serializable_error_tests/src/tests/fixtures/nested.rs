#![allow(dead_code)]
use serializable_error::ToSerializableError;
use serializable_error_derive::SerializeError;

#[derive(Debug, thiserror::Error, SerializeError)]
enum TestErrorWithNested {
    #[error("Simple variant")]
    SimpleVariant,
    #[error(transparent)]
    #[serialize_error]
    NestedError(#[from] NestedError),
}

#[derive(Debug, thiserror::Error, SerializeError)]
enum NestedError {
    #[error("Simple variant")]
    SimpleVariant,
}

fn main() {
    let err = TestErrorWithNested::NestedError(NestedError::SimpleVariant);
    let serial = err.to_serializable();

    assert_eq!(serial.code, "nestedError.simpleVariant");
    assert_eq!(serial.fields, None);
}
