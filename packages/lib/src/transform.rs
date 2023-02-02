#![allow(dead_code)]

use crate::{kw, transformer::TransformRest, ListOf, NamedArg, PipeCommand};
use proc_macro2::{Span, TokenStream};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    DeriveInput, Path, Result, Token,
};

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let input = syn::parse2::<TransformInput>(input)?;
    let state;
    let rest;
    match input.ty {
        TransformType::Consume(ty) => {
            rest = ty.rest.content;
            let TransformConsume { data, .. } = ty;
            state = crate::TransformConsume { data: data.content }.build();
        }
        TransformType::Pipe(ty) => {
            rest = ty.rest.content;
            let TransformPipe {
                data,
                pipe,
                plus,
                mark,
                ..
            } = ty;
            state = crate::TransformPipe {
                data: data.content,
                pipe: pipe.map(content),
                plus: plus.map(content),
                mark: mark.map(content),
            }
            .build();
        }
        TransformType::Start(ty) => {
            rest = ty.rest.content;
            let TransformStart { path, pipe, .. } = ty;
            state = crate::TransformStart {
                path: path.content,
                pipe: pipe.map(content),
            }
            .build();
        }
    }
    state.transform(rest)
}

fn content<K, V>(t: NamedArg<K, V>) -> V {
    t.content
}

struct TransformInput {
    at_token: Token![@],
    ty: TransformType,
}

impl Parse for TransformInput {
    fn parse(input: ParseStream) -> Result<Self> {
        macro_rules! parse_type {
            ($($key:ident => $ty:ident,)*) => {{
                let lookahead = input.lookahead1();
                $(if lookahead.peek(kw::$key) {
                    return input.parse().map(TransformType::$ty);
                })*
                return Err(lookahead.error());
            }};
        }

        Ok(Self {
            at_token: input.parse()?,
            ty: (|| {
                parse_type!(
                    consume => Consume,
                    pipe    => Pipe,
                    start   => Start,
                )
            })()?,
        })
    }
}

enum TransformType {
    Consume(TransformConsume),
    Pipe(TransformPipe),
    Start(TransformStart),
}

fn parse_optional<T: Parse>(
    input: ParseStream,
    arg: &mut Option<T>,
    name: &'static str,
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

fn assert_some<T>(value: Option<T>, span: Span, name: &'static str) -> Result<T> {
    value.ok_or_else(|| syn::Error::new(span, format!("argument '{name}' must be specified")))
}

macro_rules! parse_optional {
    ($input:expr => $($name:ident),* $(,)?) => {
        let input = $input;
        $(let mut $name = None;)*
        loop {
            if input.is_empty() {
                break;
            }
            let lookahead = input.lookahead1();
            $(if lookahead.peek(kw::$name) {
                parse_optional(input, &mut $name, stringify!($name))?;
                continue;
            })*
            return Err(lookahead.error());
        }
    };
}

macro_rules! assert_some {
    ($span:expr => $($name:ident),* $(,)?) => {
        $(let $name = assert_some($name, $span, stringify!($name))?;)*
    };
}

type OptNamedArg<K, V> = Option<NamedArg<K, V>>;

struct TransformConsume {
    name: kw::consume,
    data: NamedArg<kw::data, TokenStream>,
    rest: NamedArg<kw::rest, TransformRest>,
}

impl Parse for TransformConsume {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<kw::consume>()?;
        let span = name.span();
        parse_optional!(input => data, rest);
        assert_some!(span=> data, rest);
        Ok(Self { name, data, rest })
    }
}

struct TransformPipe {
    name: kw::pipe,
    data: NamedArg<kw::data, DeriveInput>,
    pipe: OptNamedArg<kw::pipe, ListOf<PipeCommand>>,
    plus: OptNamedArg<kw::plus, TokenStream>,
    mark: OptNamedArg<kw::mark, TokenStream>,
    rest: NamedArg<kw::rest, TransformRest>,
}

impl Parse for TransformPipe {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<kw::pipe>()?;
        let span = name.span();
        parse_optional!(input => data, pipe, plus, mark, rest);
        assert_some!(span=> data, rest);
        Ok(Self {
            name,
            data,
            pipe,
            plus,
            mark,
            rest,
        })
    }
}

struct TransformStart {
    name: kw::start,
    path: NamedArg<kw::path, Path>,
    pipe: OptNamedArg<kw::pipe, ListOf<PipeCommand>>,
    rest: NamedArg<kw::rest, TransformRest>,
}

impl Parse for TransformStart {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<kw::start>()?;
        let span = name.span();
        parse_optional!(input => path, pipe, rest);
        assert_some!(span=> path, rest);
        Ok(Self {
            name,
            path,
            pipe,
            rest,
        })
    }
}