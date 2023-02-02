mod ast;
mod extend;
mod fork;
mod select;
mod wrap;

use crate::{
    ExecuteState, Optional, PipeCommand, TransformInput, TransformRest, TransformState, Transformer,
};
use ast::Nothing;
use extend::Extend;
use proc_macro2::TokenStream;
use quote::ToTokens;
use select::{Select, SelectAttr};
use syn::{DeriveInput, Ident, Result};
use wrap::{Wrap, Wrapped};

pub(crate) struct Executor;

impl crate::Executor for Executor {
    fn execute(
        cmd: PipeCommand,
        data: DeriveInput,
        rest: &mut TransformRest,
    ) -> Result<ExecuteState> {
        Ok(match maybe_builtin(&cmd) {
            Some(builtin) => builtin.execute(cmd, data, rest)?.into(),
            None => ExecuteState::Unsupported { cmd, data },
        })
    }
}

fn expand_builtin<T: Transformer>(input: TokenStream) -> TokenStream {
    if input.is_empty() {
        return input;
    }
    crate::expand(
        |input| syn::parse2::<TransformInput<T>>(input)?.transform(),
        input,
    )
}

macro_rules! builtins {
    (
        $(#[$attr:meta])* enum $name:ident
        { $($key:ident => $variant:ident;)* }
    ) => {
        $(#[$attr])*
        enum $name { $($variant,)* }

        impl $name {
            const ALL: &'static [(&'static str, $name)] =
                &[$((stringify!($key), $name::$variant),)*];

            pub fn execute(
                &self,
                cmd: PipeCommand,
                data: DeriveInput,
                rest: &mut TransformRest,
            ) -> Result<TransformState> {
                rest.track_builtin();
                match self {
                    $(Self::$variant => cmd.execute_as::<$variant>(data, rest),)*
                }
            }
        }

        pub mod commands {
            #[doc(inline)]
            use proc_macro2::TokenStream;
            $(
                pub fn $key(input: TokenStream) -> TokenStream {
                    super::expand_builtin::<super::$variant>(input)
                }
            )*
        }
    };
}

builtins! {
    #[derive(Clone, Copy, Debug)]
    enum Cmd {
        debug       => Debug;
        extend      => Extend;
        finish      => Finish;
        rename      => Rename;
        save        => Save;
        select      => Select;
        select_attr => SelectAttr;
        wrap        => Wrap;
        wrapped     => Wrapped;
    }
}

fn maybe_builtin(cmd: &PipeCommand) -> Option<Cmd> {
    if let Some(ident) = cmd.path().get_ident() {
        if let Ok(i) =
            Cmd::ALL.binary_search_by_key::<&str, _>(&ident.to_string().as_str(), |(s, _)| s)
        {
            return Some(Cmd::ALL[i].1);
        }
    }
    None
}

pub(crate) struct Debug;

impl Transformer for Debug {
    type Args = TokenStream;

    fn transform(
        data: DeriveInput,
        args: Self::Args,
        _: &mut TransformRest,
    ) -> Result<TransformState> {
        Ok(TransformState::Debug { data, args })
    }
}

pub(crate) struct Rename;

impl Transformer for Rename {
    type Args = Ident;

    fn transform(
        mut data: syn::DeriveInput,
        name: Self::Args,
        _: &mut TransformRest,
    ) -> Result<TransformState> {
        data.ident = name;
        Ok(TransformState::pipe(data))
    }
}

pub(crate) struct Save;

impl Transformer for Save {
    type Args = Optional<Ident>;

    fn transform(
        data: DeriveInput,
        name: Self::Args,
        _: &mut TransformRest,
    ) -> Result<TransformState> {
        Ok(TransformState::Save { data, name })
    }
}

pub(crate) struct Finish;

impl Transformer for Finish {
    type Args = Nothing;

    fn transform(
        data: DeriveInput,
        _: Self::Args,
        _: &mut TransformRest,
    ) -> Result<TransformState> {
        Ok(TransformState::consume(data.into_token_stream()))
    }
}
