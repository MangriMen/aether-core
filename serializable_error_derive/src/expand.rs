use inflector::Inflector;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Result};

use crate::parser::SerializeErrorAttr;

fn has_serialize_error_attr(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| attr.path().is_ident("serialize_error"))
}

pub fn expand_serializable_error(input: &DeriveInput) -> Result<TokenStream> {
    let enum_ident = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        return Err(syn::Error::new_spanned(
            enum_ident,
            "SerializeError can only be derived for enums",
        ));
    };

    let serialize_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("serialize_error"));

    let prefix = serialize_attr
        .and_then(|attr| attr.parse_args::<SerializeErrorAttr>().ok())
        .map(|attr| attr.name);

    let variants = data_enum.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;

        let variant_name = variant_ident.to_string().to_camel_case();
        let code = if let Some(prefix) = &prefix {
            format!("{prefix}.{variant_name}")
        } else {
            variant_name
        };

        let is_nested = has_serialize_error_attr(&variant.attrs);

        if is_nested {
            match &variant.fields {
                Fields::Unnamed(fields_unnamed) if fields_unnamed.unnamed.len() == 1 => {
                    quote! {
                        Self::#variant_ident(inner) => {
                            let serialized = inner.to_serializable();
                            let full_code = format!("{}.{}", #code, serialized.code);
                            serializable_error::SerializableError {
                                code: std::borrow::Cow::Owned(full_code),
                                fields: serialized.fields,
                                message: self.to_string(),
                            }
                        }
                    }
                }
                _ => syn::Error::new_spanned(
                    variant,
                    "Variants with #[serialize_error] must have exactly one unnamed field",
                )
                .to_compile_error(),
            }
        } else {
            match &variant.fields {
                Fields::Named(fields_named) => {
                    let field_names: Vec<_> = fields_named
                        .named
                        .iter()
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect();

                    quote! {
                        Self::#variant_ident { #(#field_names),* } => {
                            let fields = serde_json::json!({
                                #(stringify!(#field_names): #field_names),*
                            });
                            serializable_error::SerializableError {
                                code: std::borrow::Cow::Borrowed(#code),
                                fields: Some(fields),
                                message: self.to_string(),
                            }
                        }
                    }
                }
                Fields::Unnamed(_) => {
                    quote! {
                        Self::#variant_ident(..) => {
                            serializable_error::SerializableError {
                                code: std::borrow::Cow::Borrowed(#code),
                                fields: None,
                                message: self.to_string(),
                            }
                        }
                    }
                }
                Fields::Unit => {
                    quote! {
                        Self::#variant_ident => {
                            serializable_error::SerializableError {
                                code: std::borrow::Cow::Borrowed(#code),
                                fields: None,
                                message: self.to_string(),
                            }
                        }
                    }
                }
            }
        }
    });

    Ok(quote! {
        impl serializable_error::ToSerializableError for #enum_ident {
            fn to_serializable(&self) -> serializable_error::SerializableError {
                match self {
                    #(#variants),*
                }
            }
        }
    }
    .into())
}
