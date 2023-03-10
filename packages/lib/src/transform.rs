use crate::{kw, transformer::TransformRest, ForkCommand, ListOf, NamedArg, PipeCommand};
use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    DeriveInput, Path, Result, Token,
};

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let input = syn::parse2::<TransformInput>(input)?;
    let (state, rest) = input.ty.build();
    TransformState(state).transform(rest)
}

fn content<K, V>(t: NamedArg<K, V>) -> V {
    t.content
}

#[allow(dead_code)]
struct TransformInput {
    at_token: Token![@],
    ty: ast::Type,
}

impl Parse for TransformInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            at_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

pub struct TransformState(pub(crate) state::Type);

impl From<state::Type> for TransformState {
    fn from(value: state::Type) -> Self {
        Self(value)
    }
}

type OptNamedArg<K, V> = Option<NamedArg<K, V>>;

macro_rules! define_types {
    (enum $name:ident {
        $(+$h:ident => $h_fn:ident($h_ty:ty),)*
        $($(#[$attr:meta])* $key:ident => $variant:ident $body:tt,)*
    }) => {
        struct Hook {
            $($h: OptNamedArg<kw::$h, $h_ty>,)*
        }

        impl Hook {
            pub fn apply(self, mut rest: TransformRest) -> TransformRest {
                $(if let Some(content) = self.$h.map(content) {
                    rest.$h_fn(content);
                })*
                rest
            }
        }

        define_types! {
            @hook
            enum $name {
                hook={struct Hook { $($h,)* }}
                $(rest={
                    attr={$(#[$attr])*}
                    key={$key}
                    variant={$variant}
                    body=$body
                })*
            }
        }
    };
    (@hook enum $name:ident {
        hook=$hook:tt
        $(rest={$($rest:tt)*})*
    }) => {
        define_types! {
            @final
            enum $name {$(
                hook=$hook
                $($rest)*
            )*}
        }
    };
    (@final enum $name:ident {$(
        hook={struct $hook:ident { $($h_name:ident,)* }}
        attr={$($attr:tt)*}
        key={$key:ident}
        variant={$variant:ident}
        body={
            $(!$f1_name:ident: $f1_ty:ty,)*
            $(?$f2_name:ident: $f2_ty:ty,)*
        }
    )*}) => {
        pub mod state {
            use super::*;

            pub(crate) enum $name {
                $($variant($variant),)*
            }


            $(pub struct $variant {
                $(pub(crate) $f1_name: $f1_ty,)*
                $(pub(crate) $f2_name: Option<$f2_ty>,)*
            }

            impl TransformState {
                $($attr)*
                pub fn $key($($f1_name: $f1_ty,)*) -> $variant {
                    $variant {
                        $($f1_name,)*
                        $($f2_name: None,)*
                    }
                }
            }

            impl $variant {
                $(pub fn $f2_name(self, $f2_name: $f2_ty) -> Self {
                    Self {
                        $f2_name: Some($f2_name),
                        ..self
                    }
                })*

                pub fn build(self) -> TransformState {
                    $name::$variant(self).into()
                }
            })*
        }

        mod ast {
            use super:: *;

            pub(crate) enum $name {
                $($variant($variant),)*
            }

            impl Parse for $name {
                fn parse(input: ParseStream) -> Result<Self> {
                    let lookahead = input.lookahead1();
                    $(if lookahead.peek(kw::$key) {
                        return input.parse().map(Self::$variant);
                    })*
                    Err(lookahead.error())
                }
            }

            impl $name {
                pub fn build(self) -> (state::$name, TransformRest) {
                    match self {$(
                        Self::$variant(t) => (
                            state::$name::$variant(state::$variant {
                                $($f1_name: t.$f1_name.content,)*
                                $($f2_name: t.$f2_name.map(content),)*
                            }),
                            t.hook.apply(t.rest.content),
                        ),
                    )*}
                }
            }

            $(#[allow(dead_code)]
            pub(crate) struct $variant {
                name: kw::$key,
                rest: NamedArg<kw::rest, TransformRest>,
                hook: $hook,
                $($f1_name: NamedArg<kw::$f1_name, $f1_ty>,)*
                $($f2_name: OptNamedArg<kw::$f2_name, $f2_ty>,)*
            }

            impl Parse for $variant {
                fn parse(input: ParseStream) -> Result<Self> {
                    let name = input.parse::<kw::$key>()?;
                    let _span = name.span();
                    parse_named_args!(input, kw => rest, $($f1_name,)* $($f2_name,)* $($h_name,)*);
                    require_named_args!(_span => rest, $($f1_name,)*);
                    Ok(Self {
                        name,
                        rest,
                        hook: $hook { $($h_name,)* },
                        $($f1_name,)*
                        $($f2_name,)*
                    })
                }
            })*
        }
    };
}

define_types! {
    enum Type {
        // Parse hook arguments.
        +pipe   => with_pipe(ListOf<PipeCommand>),
        +extra  => with_extra(TokenStream),
        +marker => with_marker(TokenStream),

        /// ```
        /// transform! {
        ///     @consume
        ///     data={#data}
        ///     ...
        /// }
        /// ```
        consume => Consume {
            !data: TokenStream,
        },

        /// ```
        /// transform! {
        ///     @debug
        ///     data={#data}
        ///     args={#args}
        ///     ...
        /// }
        /// ```
        debug => Debug {
            !data: DeriveInput,
            ?args: TokenStream,
        },
        /// ```
        /// transform! {
        ///     @fork
        ///     data={#data}
        ///     fork={#fork}
        ///     ...
        /// }
        /// ```
        fork => Fork {
            !data: DeriveInput,
            ?fork: ListOf<ForkCommand>,
        },

        /// ```
        /// transform! {
        ///     @pipe
        ///     data={#data}
        ///     ...
        /// }
        /// ```
        pipe => Pipe {
            !data: DeriveInput,
        },

        /// ```
        /// transform! {
        ///     @resume
        ///     path={#path}
        ///     ...
        /// }
        /// ```
        resume => Resume {
            !path: Path,
        },

        /// ```
        /// transform! {
        ///     @save
        ///     ...
        /// }
        /// ```
        save => Save {
            !data: DeriveInput,
        },
    }
}
