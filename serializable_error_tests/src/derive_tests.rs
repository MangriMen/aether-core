#[test]
fn test_derive_macro_compiles() {
    let try_build = trybuild::TestCases::new();
    try_build.pass("src/tests/fixtures/simple.rs");
    try_build.pass("src/tests/fixtures/simple_with_name.rs");

    try_build.pass("src/tests/fixtures/with_fields.rs");

    try_build.pass("src/tests/fixtures/nested.rs");
    try_build.pass("src/tests/fixtures/nested_with_fields.rs");
}
