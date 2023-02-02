mod ast;
mod builtin;
mod define;
mod pipe;
mod predefined;
mod transform;

use builtin::DefaultExecutor;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse::Parse, spanned::Spanned, DeriveInput, Path, Result};

pub use ast::{ListOf, NamedArg, Optional, PipeCommand, TransformInput, TransformRest};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(args);
    custom_keyword!(consume);
    custom_keyword!(data);
    custom_keyword!(path);
    custom_keyword!(pipe);
    custom_keyword!(plus);
    custom_keyword!(rest);
    custom_keyword!(save);
    custom_keyword!(start);
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

pub trait Transformer: Sized {
    type Args: Parse;

    fn transform(
        data: DeriveInput,
        args: Self::Args,
        rest: &mut TransformRest,
    ) -> Result<TransformState>;
}

pub trait Executor: Sized {
    fn execute(
        cmd: PipeCommand,
        data: DeriveInput,
        rest: &mut TransformRest,
    ) -> Result<ExecuteOutput>;
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

    pub(crate) fn transform(self, rest: TransformRest) -> Result<TokenStream> {
        self.transform_with::<DefaultExecutor>(rest)
    }

    pub(crate) fn transform_with<T: Executor>(
        self,
        mut rest: TransformRest,
    ) -> Result<TokenStream> {
        let mut state = self;
        let mut span = Span::call_site();
        Ok(loop {
            let data = match state {
                TransformState::Consume { data } => {
                    if !rest.is_empty() {
                        return Err(syn::Error::new(
                            span,
                            "a consume command should not be followed by other commands",
                        ));
                    }
                    let mut data = data.unwrap_or_default();
                    data.extend(rest.take_plus());
                    break data;
                }
                TransformState::Pipe { data, pipe, plus } => {
                    if let Some(pipe) = pipe {
                        rest.prepend_pipe(pipe);
                    }
                    if let Some(plus) = plus {
                        rest.prepend_plus(plus);
                    }
                    data
                }
                TransformState::Start { path, pipe, plus } => {
                    if let Some(pipe) = pipe {
                        rest.prepend_pipe(pipe);
                    }
                    if let Some(plus) = plus {
                        rest.prepend_plus(plus);
                    }
                    break quote!(#path! {
                        rest={#rest}
                    });
                }
            };
            match rest.next_pipe() {
                Some(cmd) => {
                    span = cmd.path().span();
                    match T::execute(cmd, data, &mut rest)? {
                        ExecuteOutput::Executed { state: s } => {
                            state = s;
                        }
                        ExecuteOutput::Unsupported { cmd, data } => {
                            break cmd.execute(data, rest);
                        }
                    }
                }
                None => {
                    return Err(syn::Error::new(span, "a pipe command should be consumed"));
                }
            }
        })
    }
}

pub enum ExecuteOutput {
    Executed { state: TransformState },
    Unsupported { cmd: PipeCommand, data: DeriveInput },
}

impl From<TransformState> for ExecuteOutput {
    fn from(state: TransformState) -> Self {
        Self::Executed { state }
    }
}
