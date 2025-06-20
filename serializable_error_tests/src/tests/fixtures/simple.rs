#![allow(dead_code)]
use serializable_error::ToSerializableError;
use serializable_error_derive::SerializeError;

#[derive(Debug, thiserror::Error, SerializeError)]
enum TestError {
    #[error("Simple variant")]
    SimpleVariant,
}

fn main() {
    let err = TestError::SimpleVariant;
    let serial = err.to_serializable();

    assert_eq!(serial.code, "simpleVariant");
    assert_eq!(serial.fields, None);
}
