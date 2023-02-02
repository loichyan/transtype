mod builtin;
mod define;
mod pipe;
mod predefined;
mod transform;
mod transformer;
mod utils;

use proc_macro2::TokenStream;
use syn::{DeriveInput, Ident, Path, Result};

#[doc(inline)]
pub use self::{
    pipe::PipeCommand,
    transformer::{ExecuteState, Executor, TransformInput, TransformRest, Transformer},
    utils::{ListOf, NamedArg, Optional},
};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(args);
    custom_keyword!(consume);
    custom_keyword!(data);
    custom_keyword!(path);
    custom_keyword!(pipe);
    custom_keyword!(plus);
    custom_keyword!(rest);
    custom_keyword!(start);
    custom_keyword!(this);
}

#[doc(hidden)]
pub mod private {
    #[doc(inline)]
    pub use crate::builtin::commands::*;
    use proc_macro2::TokenStream;

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
}

fn expand(f: fn(TokenStream) -> Result<TokenStream>, input: TokenStream) -> TokenStream {
    f(input).unwrap_or_else(syn::Error::into_compile_error)
}

pub enum TransformState {
    /// ```
    /// transform! {
    ///     @consume
    ///     data={#data}
    ///     rest={#rest}
    /// }
    /// ```
    Consume { data: Option<TokenStream> },
    /// ```
    /// pipe! {
    ///     -> debug(#args)
    /// }
    /// ```
    Debug {
        data: DeriveInput,
        args: TokenStream,
    },
    /// ```
    /// pipe! {
    ///     -> fork(#fork)
    /// }
    /// ```
    Fork { data: DeriveInput, fork: ListOf<()> },
    /// ```
    /// transform! {
    ///     @pipe
    ///     data={#data}
    ///     pipe={#pipe}
    ///     plus={#plus}
    ///     rest={#rest}
    /// }
    /// ```
    Pipe {
        data: DeriveInput,
        pipe: Option<ListOf<PipeCommand>>,
        plus: Option<TokenStream>,
    },
    /// ```
    /// pipe! {
    ///     -> save(#name)
    /// }
    /// ```
    Save {
        data: DeriveInput,
        name: Optional<Ident>,
    },
    /// ```
    /// transform! {
    ///     @start
    ///     path={#path}
    ///     pipe={#pipe}
    ///     plus={#plus}
    ///     rest={#rest}
    /// }
    /// ```
    Start {
        path: Path,
        pipe: Option<ListOf<PipeCommand>>,
        plus: Option<TokenStream>,
    },
}

impl TransformState {
    pub fn consume(data: TokenStream) -> Self {
        Self::Consume { data: Some(data) }
    }

    pub fn pipe(data: DeriveInput) -> Self {
        Self::Pipe {
            data,
            pipe: None,
            plus: None,
        }
    }

    pub fn start(path: Path) -> Self {
        Self::Start {
            path,
            pipe: None,
            plus: None,
        }
    }
}
