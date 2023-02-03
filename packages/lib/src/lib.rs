#[macro_use]
mod utils;

mod builtin;
mod define;
mod fork;
mod pipe;
mod predefined;
mod transform;
mod transformer;

use proc_macro2::TokenStream;
use syn::Result;

#[doc(inline)]
pub use self::{
    fork::ForkCommand,
    pipe::PipeCommand,
    transform::TransformState,
    transformer::{ExecuteState, Executor, TransformInput, TransformRest, Transformer},
    utils::{ListOf, NamedArg, Optional},
};

pub mod state {
    #[doc(inline)]
    pub use crate::transform::state::*;
}

#[doc(hidden)]
pub mod private {
    #[doc(inline)]
    pub use crate::builtin::commands::*;
    use proc_macro2::{Span, TokenStream};
    use syn::parse::{Parse, ParseStream, Result};

    macro_rules! expose_expand {
        ($($name:ident),* $(,)?) => {$(
            pub fn $name(input: TokenStream) -> TokenStream {
                $crate::expand($crate::$name::expand, input)
            }
        )*};
    }

    expose_expand! {
       define,
       pipe,
       predefined,
       transform
    }

    pub fn parse_named_arg<T: Parse>(
        name: &'static str,
        arg: &mut Option<T>,
        input: ParseStream,
    ) -> Result<()> {
        if arg.is_some() {
            return Err(syn::Error::new(
                input.span(),
                format!("duplicated argument '{name}'"),
            ));
        }
        *arg = Some(input.parse()?);
        Ok(())
    }

    pub fn require_named_arg<T>(name: &'static str, arg: Option<T>, span: Span) -> Result<T> {
        arg.ok_or_else(|| syn::Error::new(span, format!("argument '{name}' must be specified")))
    }
}

fn expand(f: fn(TokenStream) -> Result<TokenStream>, input: TokenStream) -> TokenStream {
    f(input).unwrap_or_else(syn::Error::into_compile_error)
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(args);
    custom_keyword!(consume);
    custom_keyword!(data);
    custom_keyword!(debug);
    custom_keyword!(extra);
    custom_keyword!(fork);
    custom_keyword!(marker);
    custom_keyword!(path);
    custom_keyword!(pipe);
    custom_keyword!(rest);
    custom_keyword!(resume);
    custom_keyword!(save);
    custom_keyword!(this);
}
