use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::__private::TokenStream2;
use syn::spanned::Spanned;

use crate::parse::ErrorVariant;

/// Generate `debug_fmt` fn.
///
/// The generated fn will bu like:
/// ```rust ignore
/// fn debug_fmt(&self, layer: usize, buf: &mut Vec<String>);
/// ```
pub(crate) fn build_debug_fmt_impl(enum_name: &Ident, variants: &Vec<ErrorVariant>) -> TokenStream2 {
    let match_arms: Vec<TokenStream2> = variants.iter()
        .map(to_debug_match_arm).collect();

    quote! {
        fn debug_fmt(&self, layer: usize, buf: &mut Vec<String>) {
            use #enum_name::*;
            match self {
                #(#match_arms)*
            }
        }
    }
}

/// Convert [ErrorVariant] into an match arm that will be used in `build_debug_impl`
///
/// The generated match arm will be like:
/// ```rust ignore
/// ErrorKindWithSource { source, .. } => {
///     debug_fmt(source, layer + 1, buf);
/// },
/// ErrorKindWithoutSource { .. } => {
///     buf.push(format!("{layer}: {}, at {}", format!(#display), location)));
/// }
/// ```
fn to_debug_match_arm(variant: &ErrorVariant) -> TokenStream2 {
    let name = &variant.name;
    let fields = &variant.fields;
    let display = &variant.display;
    let cfg = match &variant.cfg_attr {
        Some(cfg) => quote_spanned! { cfg.span() => #cfg },
        None => quote! {},
    };

    match (variant.has_location, variant.has_source, variant.has_external_cause) {
        (true, true, _) => quote_spanned! {
            variant.span => #cfg #[allow(unused_variables)] #name { #(#fields),* } => {
                buf.push(format!("{layer}: {}, at {}", format!(#display), location));
                source.debug_fmt(layer + 1, buf);
            },
        },
        (true, false, true) => quote_spanned! {
            variant.span => #cfg #[allow(unused_variables)] #name { #(#fields),* } => {
                buf.push(format!("{layer}: {}, at {}", format!(#display), location));
                buf.push(format!("{}: {:?}", layer + 1, error));
            },
        },
        (true, false, false) => quote_spanned! {
            variant.span => #cfg #[allow(unused_variables)] #name { #(#fields),* } => {
                buf.push(format!("{layer}: {}, at {}", format!(#display), location));
            },
        },
        (false, true, _) => quote_spanned! {
            variant.span => #cfg #[allow(unused_variables)] #name { #(#fields),* } => {
                buf.push(format!("{layer}: {}", format!(#display)));
                source.debug_fmt(layer + 1, buf);
            },
        },
        (false, false, true) => quote_spanned! {
            variant.span => #cfg #[allow(unused_variables)] #name { #(#fields),* } => {
                buf.push(format!("{layer}: {}", format!(#display)));
                buf.push(format!("{}: {:?}", layer + 1, error));
            },
        },
        (false, false, false) => quote_spanned! {
            variant.span => #cfg #[allow(unused_variables)] #name { #(#fields),* } => {
                buf.push(format!("{layer}: {}", format!(#display)));
            },
        },
    }
}

pub(crate) fn build_next_impl(enum_name: &Ident, variants: &Vec<ErrorVariant>) -> TokenStream2 {
    let match_arms: Vec<TokenStream2> = variants.iter().map(to_next_match_arm).collect();
    quote! {
        fn next(&self) -> Option<&dyn ::snafu_stack_error::StackError> {
            use #enum_name::*;
            match self {
                #(#match_arms)*
            }
        }
    }
}

fn to_next_match_arm(variant: &ErrorVariant) -> TokenStream2 {
    let name = &variant.name;
    let fields = &variant.fields;
    let cfg = if let Some(cfg) = &variant.cfg_attr {
        quote_spanned! { cfg.span() => #cfg }
    } else {
        quote! {}
    };

    let source = match variant.has_source {
        true => quote! {
            Some(source)
        },
        false => quote! {
            None
        },
    };

    quote_spanned! {
        variant.span => #cfg #[allow(unused_variables)] #name { #(#fields),* } => {
            #source
        }
    }
}

pub(crate) fn build_debug_impl(enum_name: &Ident) -> TokenStream2 {
    quote! {
        impl std::fmt::Debug for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use ::snafu_stack_error::StackError;
                let mut buf = vec![];
                self.debug_fmt(0, &mut buf);
                write!(f, "{}", buf.join("\n"))
            }
        }
    }
}
