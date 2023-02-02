mod ast;
mod extend;
mod select;
mod wrap;

use crate::{
    ExecuteOutput, Executor, PipeCommand, TransformInput, TransformRest, TransformState,
    Transformer,
};
use ast::Nothing;
use proc_macro2::TokenStream;
use quote::{format_ident, quote_spanned, ToTokens};
use syn::{DeriveInput, Ident, Result};

#[doc(inline)]
pub use self::{
    extend::Extend,
    select::{Select, SelectAttr},
    wrap::{Wrap, Wrapped},
};

pub struct DefaultExecutor;

impl Executor for DefaultExecutor {
    fn execute(
        cmd: PipeCommand,
        data: DeriveInput,
        rest: &mut TransformRest,
    ) -> Result<ExecuteOutput> {
        Ok(match maybe_builtin(&cmd) {
            Some(builtin) => builtin.execute(cmd, data, rest)?.into(),
            None => ExecuteOutput::Unsupported { cmd, data },
        })
    }
}

pub fn expand_builtin<T: Transformer>(input: TokenStream) -> TokenStream {
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
    enum Builtin {
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

fn maybe_builtin(cmd: &PipeCommand) -> Option<Builtin> {
    if let Some(ident) = cmd.path().get_ident() {
        if let Ok(i) =
            Builtin::ALL.binary_search_by_key::<&str, _>(&ident.to_string().as_str(), |(s, _)| s)
        {
            return Some(Builtin::ALL[i].1);
        }
    }
    None
}

pub struct Debug;

impl Transformer for Debug {
    type Args = TokenStream;

    fn transform(
        data: DeriveInput,
        args: Self::Args,
        rest: &mut TransformRest,
    ) -> Result<TransformState> {
        let span = rest.span();
        let plus = rest.take_plus();
        let name = format_ident!("DEBUG_{}", data.ident, span = data.ident.span());
        let data = quote_spanned!(span=>
            data={#data}
            args={#args}
            plus={#plus}
        )
        .to_string();
        Ok(TransformState::consume(quote_spanned!(span=>
            macro_rules! #name { () => {{ #data }}; }
        )))
    }
}

pub struct Rename;

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

pub struct Save;

impl Transformer for Save {
    type Args = Option<Ident>;

    fn transform(
        data: DeriveInput,
        name: Self::Args,
        rest: &mut TransformRest,
    ) -> Result<TransformState> {
        let span = rest.span();
        let name = name.unwrap_or_else(|| data.ident.clone());
        let plus = rest.take_plus();
        Ok(TransformState::consume(quote_spanned!(span=>
            macro_rules! #name {
                ($($args:tt)*) => {
                    ::transtype::__predefined! {
                        args={$($args)*}
                        data={#data}
                        plus={#plus}
                    }
                };
            }
        )))
    }
}

pub struct Finish;

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
