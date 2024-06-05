use proc_macro2::{Ident, Span};
use syn::{Attribute, parenthesized, Variant};
use syn::__private::TokenStream2;
use syn::parse::Parse;
use syn::spanned::Spanned;

#[derive(Debug, Clone)]
pub(crate) struct ErrorVariant {
    pub name: Ident,
    pub fields: Vec<Ident>,
    pub has_location: bool,
    pub has_source: bool,
    pub has_external_cause: bool,
    pub display: TokenStream2,
    pub span: Span,
    pub cfg_attr: Option<Attribute>,
}

impl ErrorVariant {
    /// Construct [ErrorVariant] from [Variant]
    pub(crate) fn from_enum_variant(variant: Variant) -> Self {
        let span = variant.span();

        let mut has_location = false;
        let mut has_source = false;
        let mut has_external_cause = false;
        for field in &variant.fields {
            if let Some(ident) = &field.ident {
                if ident == "source" {
                    has_source = true;
                } else if ident == "error" {
                    has_external_cause = true;
                } else if ident == "location" {
                    has_location = true;
                }
            }
        }

        let mut display = None;
        let mut cfg_attr = None;
        for attr in variant.attrs {
            if attr.path().is_ident("snafu") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("display") {
                        let content;
                        parenthesized!(content in meta.input);
                        display = Some(content.parse()?);
                        Ok(())
                    } else {
                        Err(meta.error("unrecognized repr"))
                    }
                }).expect("Each error should contains a display attribute");
            }

            if attr.path().is_ident("cfg") {
                cfg_attr = Some(attr);
            }
        }

        let fields_ident = variant.fields.into_iter().map(|field|
            field.clone().ident.unwrap_or_else(|| Ident::new("-", field.span()))
        ).collect();

        Self {
            name: variant.ident,
            fields: fields_ident,
            has_location,
            has_source,
            has_external_cause,
            display: display.unwrap(),
            span,
            cfg_attr,
        }
    }
}
