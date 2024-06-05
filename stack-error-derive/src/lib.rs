use proc_macro::TokenStream;

use quote::quote;
use syn::__private::TokenStream2;
use syn::ItemEnum;

use crate::gen::{build_debug_fmt_impl, build_debug_impl, build_next_impl};
use crate::parse::ErrorVariant;

mod gen;
mod parse;

#[proc_macro_attribute]
pub fn stack_trace_debug(args: TokenStream, input: TokenStream) -> TokenStream {
    match stack_trace_debug2(args, input.into()) {
        Ok(token) => token.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn stack_trace_debug2(args: TokenStream, input: TokenStream2) -> syn::Result<TokenStream2> {
    let input_clone = input.clone();
    let error_enum: ItemEnum = syn::parse2(input_clone)?;
    let enum_name = error_enum.ident;
    let variants: Vec<ErrorVariant> = error_enum.variants.into_iter()
        .map(ErrorVariant::from_enum_variant).collect();

    let debug_fmt_fn = build_debug_fmt_impl(&enum_name, &variants);
    let next_fn = build_next_impl(&enum_name, &variants);
    let debug_impl = build_debug_impl(&enum_name);
    let quote_args: TokenStream2 = args.into();

    Ok(quote! {
        #quote_args
        #input

        impl ::snafu_stack_error::StackError for #enum_name {
            #debug_fmt_fn
            #next_fn
        }

        #debug_impl
    })
}
