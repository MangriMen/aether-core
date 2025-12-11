extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(RegisterSchema)]
pub fn derive_register_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        inventory::submit! {
            register_schema::SchemaEntry {
                name: stringify!(#name),
                schema: || schemars::schema_for!(#name)
            }
        }
    };

    TokenStream::from(expanded)
}
