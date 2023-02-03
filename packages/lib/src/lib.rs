mod builtin;
mod define;
mod fork;
mod pipe;
mod predefined;
mod transform;
mod transformer;
mod utils;

use proc_macro2::TokenStream;
use syn::{DeriveInput, Ident, Path, Result};

#[doc(inline)]
pub use self::{
    fork::ForkCommand,
    pipe::PipeCommand,
    transformer::{ExecuteState, Executor, TransformInput, TransformRest, Transformer},
    utils::{ListOf, NamedArg, Optional},
};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(args);
    custom_keyword!(consume);
    custom_keyword!(data);
    custom_keyword!(marker);
    custom_keyword!(path);
    custom_keyword!(pipe);
    custom_keyword!(extra);
    custom_keyword!(rest);
    custom_keyword!(resume);
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
    Consume(TransformConsume),
    /// ```
    /// pipe! {
    ///     -> debug(#args)
    /// }
    /// ```
    Debug(TransformDebug),
    /// ```
    /// pipe! {
    ///     -> fork(#fork)
    /// }
    /// ```
    Fork(TransformFork),
    /// ```
    /// transform! {
    ///     @pipe
    ///     data={#data}
    ///     pipe={#pipe}
    ///     extra={#extra}
    ///     marker={#marker}
    ///     rest={#rest}
    /// }
    /// ```
    Pipe(TransformPipe),
    /// ```
    /// transform! {
    ///     @resume
    ///     path={#path}
    ///     pipe={#pipe}
    ///     rest={#rest}
    /// }
    /// ```
    Resume(TransformResume),
    /// ```
    /// pipe! {
    ///     -> save(#name)
    /// }
    /// ```
    Save(TransformSave),
}

impl TransformState {
    pub fn consume(data: TokenStream) -> TransformConsume {
        TransformConsume { data }
    }

    pub fn debug(data: DeriveInput) -> TransformDebug {
        TransformDebug {
            data,
            args: Default::default(),
        }
    }

    pub fn fork(data: DeriveInput) -> TransformFork {
        TransformFork {
            data,
            fork: Default::default(),
        }
    }

    pub fn pipe(data: DeriveInput) -> TransformPipe {
        TransformPipe {
            data,
            pipe: Default::default(),
            extra: Default::default(),
            marker: Default::default(),
        }
    }

    pub fn resume(path: Path) -> TransformResume {
        TransformResume {
            path,
            pipe: Default::default(),
        }
    }

    pub fn save(data: DeriveInput) -> TransformSave {
        TransformSave {
            data,
            name: Default::default(),
        }
    }
}

pub struct TransformConsume {
    data: TokenStream,
}

impl TransformConsume {
    pub fn build(self) -> TransformState {
        TransformState::Consume(self)
    }
}

pub struct TransformDebug {
    data: DeriveInput,
    args: Option<TokenStream>,
}

impl TransformDebug {
    pub fn args(self, args: TokenStream) -> Self {
        Self {
            args: Some(args),
            ..self
        }
    }

    pub fn build(self) -> TransformState {
        TransformState::Debug(self)
    }
}

pub struct TransformFork {
    data: DeriveInput,
    fork: Option<ListOf<ForkCommand>>,
}

impl TransformFork {
    pub fn fork(self, fork: ListOf<ForkCommand>) -> Self {
        Self {
            fork: Some(fork),
            ..self
        }
    }

    pub fn build(self) -> TransformState {
        TransformState::Fork(self)
    }
}

pub struct TransformPipe {
    data: DeriveInput,
    pipe: Option<ListOf<PipeCommand>>,
    extra: Option<TokenStream>,
    marker: Option<TokenStream>,
}

impl TransformPipe {
    pub fn pipe(self, pipe: ListOf<PipeCommand>) -> Self {
        Self {
            pipe: Some(pipe),
            ..self
        }
    }

    pub fn extra(self, extra: TokenStream) -> Self {
        Self {
            extra: Some(extra),
            ..self
        }
    }

    pub fn marker(self, marker: TokenStream) -> Self {
        Self {
            marker: Some(marker),
            ..self
        }
    }

    pub fn build(self) -> TransformState {
        TransformState::Pipe(self)
    }
}

pub struct TransformResume {
    path: Path,
    pipe: Option<ListOf<PipeCommand>>,
}

impl TransformResume {
    pub fn pipe(self, pipe: ListOf<PipeCommand>) -> Self {
        Self {
            pipe: Some(pipe),
            ..self
        }
    }

    pub fn build(self) -> TransformState {
        TransformState::Resume(self)
    }
}

pub struct TransformSave {
    data: DeriveInput,
    name: Option<Ident>,
}

impl TransformSave {
    pub fn name(self, name: Ident) -> Self {
        Self {
            name: Some(name),
            ..self
        }
    }

    pub fn build(self) -> TransformState {
        TransformState::Save(self)
    }
}
