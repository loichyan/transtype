mod define;
mod extend;
mod pipe;
mod rename;
mod transform;
mod utils;
mod wrap;

use pipe::PipeInput;
use proc_macro::TokenStream;
use syn::{parse::Nothing, parse_macro_input, DeriveInput};
use transform::TransformInput;

mod kw {
    use syn::custom_keyword;

    custom_keyword!(data);
    custom_keyword!(args);
    custom_keyword!(rest);
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
    let input = parse_macro_input!(input as PipeInput);
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
