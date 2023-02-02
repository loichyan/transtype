mod ast;
mod extend;
mod select;
mod wrap;

use crate::{ExecuteOutput, Executor, PipeCommand, TransformRest, TransformState, Transformer};
use ast::Nothing;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
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

macro_rules! builtins {
    (
        $(#[$attr:meta])* enum $name:ident
        { $($(#[$cmd_attr:meta])* $key:ident => $variant:ident;)* }
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

        #[doc(hidden)]
        #[macro_export]
        macro_rules! define_builtins {
            () => {
                $(
                    $(#[$cmd_attr])*
                    #[proc_macro]
                    pub fn $key(input: ::proc_macro::TokenStream) -> ::proc_macro::TokenStream {
                        $crate::private::expand_builtin::<
                            $crate::private::commands::$variant,
                            ::proc_macro::TokenStream,
                        >(input)
                    }
                )*

            };
        }

        pub mod commands {
            #[doc(inline)]
            pub use super::{ $($variant,)* };
        }
    };
}

builtins! {
    #[derive(Clone, Copy, Debug)]
    enum Builtin {
        /// Consumes all rest tokens, generates a macro prefixes with `DEBUG_` which
        /// returns the stringified tokens tree.
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
        let rest = rest.take();
        let name = format_ident!("DEBUG_{}", data.ident, span = data.ident.span());
        Ok(TransformState::consume(quote!(macro_rules! #name {
            () => {
                stringify! {
                    data={#data}
                    args={#args}
                    rest={#rest}
                }
            };
        })))
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
        let name = name.unwrap_or_else(|| data.ident.clone());
        let rest = rest.take();
        Ok(TransformState::consume(quote!(macro_rules! #name {
            ($($args:tt)*) => {
                ::transtype::predefined! {
                    args={$($args)*}
                    data={#data}
                    save={#rest}
                }
            };
        })))
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
