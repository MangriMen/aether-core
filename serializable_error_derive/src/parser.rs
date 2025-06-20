use syn::{
    Expr, Lit, MetaNameValue, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub struct SerializeErrorAttr {
    pub name: String,
}

impl Parse for SerializeErrorAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name_values: Punctuated<MetaNameValue, Token![,]> =
            Punctuated::parse_terminated(input)?;

        let mut name = None;

        for nv in name_values {
            if nv.path.is_ident("name") {
                if let Expr::Lit(expr_lit) = nv.value {
                    if let Lit::Str(lit_str) = expr_lit.lit {
                        name = Some(lit_str.value());
                    }
                }
            }
        }

        Ok(SerializeErrorAttr {
            name: name.ok_or_else(|| input.error("missing `name` argument"))?,
        })
    }
}
