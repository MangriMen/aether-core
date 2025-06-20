mod expand;
mod parser;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_derive(SerializeError, attributes(serialize_error))]
pub fn serialize_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    match expand::expand_serializable_error(&input) {
        Ok(token_stream) => token_stream,
        Err(e) => e.to_compile_error().into(),
    }
}
