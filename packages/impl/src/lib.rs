#[macro_use]
mod ast;

mod debug;
mod define;
mod extend;
mod finish;
mod pipe;
mod rename;
mod select;
mod transform;
mod wrap;

use proc_macro::TokenStream;
use syn::{parse::Nothing, parse_macro_input, DeriveInput};
use transform::TransformInput;

mod kw {
    use syn::custom_keyword;

    custom_keyword!(defined);
}

#[proc_macro_attribute]
pub fn define(attr: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(attr as Nothing);
    let input = parse_macro_input!(input as DeriveInput);
    define::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn pipe(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as proc_macro2::TokenStream);
    pipe::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn transform(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TransformInput);
    transform::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
