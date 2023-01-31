use crate::{
    ast::{ListOf, Nothing, PipeCommand, TokenStreamExt},
    extend::Extend,
    kw,
    rename::Rename,
    select::{Select, SelectAttr},
    wrap::{Wrap, Wrapped},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token, DeriveInput, Ident, Result, Token,
};
use transtype_lib::{Command, NamedArg, TransformOutput, Transformer};

pub type TransformInput = transtype_lib::TransformInput<Transform>;

pub fn expand(input: TransformInput) -> Result<TokenStream> {
    input.transform()
}

pub struct Transform;

impl Transformer for Transform {
    type Data = TransfromData;
    type Args = TokenStream;

    fn transform(
        data: Self::Data,
        args: Self::Args,
        rest_tokens: &mut TokenStream,
    ) -> Result<TransformOutput> {
        // 1) Parse rest commands.
        let mut span = args.span();
        let mut rest = if rest_tokens.is_empty() {
            TransformRest::default()
        } else {
            std::mem::take(rest_tokens).parse2()?
        };
        rest.extend_args(args)?;
        let output = if let Some(mut data) = data.0 {
            loop {
                // 2) Execute pipe commands.
                let output = loop {
                    if let Some(cmd) = rest.pipe.content.0.pop() {
                        span = cmd.path.span();
                        let output = match maybe_builtin(&cmd) {
                            Some(builtin) => {
                                match builtin {
                                    Builtin::Debug | Builtin::Save => {
                                        *rest_tokens =
                                            std::mem::take(&mut rest).into_token_stream();
                                    }
                                    _ => {}
                                }
                                builtin.execute(cmd, data, rest_tokens).map_err(|mut e| {
                                    e.combine(syn::Error::new(
                                        span,
                                        "an error occurs in this command",
                                    ));
                                    e
                                })?
                            }
                            None => {
                                break TransformOutput::Transferr {
                                    path: cmd.path,
                                    data: Some(data),
                                    args: cmd.args,
                                }
                            }
                        };
                        match output {
                            TransformOutput::Pipe { data: d } => data = d,
                            _ => break output,
                        }
                    } else {
                        break TransformOutput::Pipe { data };
                    }
                };
                match output {
                    TransformOutput::Transform { data: d, args: a } => {
                        data = d;
                        rest.extend_args(a)?;
                    }
                    _ => break output,
                }
            }
        } else {
            TransformOutput::Consume {
                data: Default::default(),
            }
        };
        let pipes = &mut rest.pipe.content;
        let withs = &mut rest.with.content;
        Ok(match output {
            TransformOutput::Consume { mut data } => {
                if !pipes.0.is_empty() {
                    return Err(syn::Error::new(
                        span,
                        "a consumer command should not be followed with other commands",
                    ));
                }
                data.extend(std::mem::take(withs));
                TransformOutput::Consume { data }
            }
            TransformOutput::Pipe { .. } => {
                return Err(syn::Error::new(span, "a pipe command should be consumed"))
            }
            _ => {
                if !withs.is_empty() || !pipes.0.is_empty() {
                    *rest_tokens = rest.into_token_stream();
                }
                output
            }
        })
    }
}

pub struct TransfromData(Option<DeriveInput>);

impl Parse for TransfromData {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if input.is_empty() {
            Self(None)
        } else {
            Self(Some(input.parse()?))
        })
    }
}

impl ToTokens for TransfromData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(t) = self.0.as_ref() {
            t.to_tokens(tokens)
        }
    }
}

pub enum TransformCommand {
    Pipe(PipeCommand),
    Add(AddCommand),
}

impl Parse for TransformCommand {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![->]) {
            input.parse().map(Self::Pipe)
        } else if lookahead.peek(Token![+]) {
            input.parse().map(Self::Add)
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct AddCommand {
    pub add_token: Token![+],
    pub brace_token: token::Brace,
    pub args: TokenStream,
}

impl Parse for AddCommand {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            add_token: input.parse()?,
            brace_token: braced!(content in input),
            args: content.parse()?,
        })
    }
}

pub struct TransformRest {
    pipe: NamedArg<kw::pipe, ListOf<PipeCommand>>,
    with: NamedArg<kw::with, TokenStream>,
}

impl Default for TransformRest {
    fn default() -> Self {
        Self {
            pipe: default_named_arg(),
            with: default_named_arg(),
        }
    }
}

fn default_named_arg<K, V>() -> NamedArg<K, V>
where
    K: Default,
    V: Default,
{
    NamedArg {
        name: K::default(),
        eq_token: Default::default(),
        brace_token: Default::default(),
        content: V::default(),
    }
}

impl Parse for TransformRest {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            pipe: input.parse()?,
            with: input.parse()?,
        })
    }
}

impl ToTokens for TransformRest {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.pipe.to_tokens(tokens);
        self.with.to_tokens(tokens);
    }
}

impl TransformRest {
    fn extend_args(&mut self, args: TokenStream) -> Result<()> {
        args.parse2::<ListOf<_>>()?
            .0
            .into_iter()
            .rev()
            .for_each(|cmd| match cmd {
                TransformCommand::Pipe(cmd) => self.pipe.content.0.push(cmd),
                TransformCommand::Add(cmd) => self.with.content.extend(cmd.args),
            });
        Ok(())
    }
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
                rest: &mut TokenStream,
            ) -> Result<TransformOutput> {
                match self {
                    $(Self::$variant => cmd.execute::<$variant>(data, rest),)*
                }
            }
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
    if let Some(ident) = cmd.path.get_ident() {
        if let Ok(i) =
            Builtin::ALL.binary_search_by_key::<&str, _>(&ident.to_string().as_str(), |(s, _)| s)
        {
            return Some(Builtin::ALL[i].1);
        }
    }
    None
}

pub struct Save;

impl Command for Save {
    type Args = Option<Ident>;

    fn execute(
        data: DeriveInput,
        name: Self::Args,
        rest: &mut TokenStream,
    ) -> Result<TransformOutput> {
        let name = name.unwrap_or_else(|| data.ident.clone());
        let rest = std::mem::take(rest);
        Ok(TransformOutput::Consume {
            data: quote!(macro_rules! #name {
                (
                    data={}
                    args=$args:tt
                    rest={}
                ) => {
                    ::transtype::transform! {
                        data={#data}
                        args=$args
                        rest={#rest}
                    }
                };
            }),
        })
    }
}

pub struct Debug;

impl Command for Debug {
    type Args = TokenStream;

    fn execute(
        data: DeriveInput,
        args: Self::Args,
        rest: &mut TokenStream,
    ) -> Result<TransformOutput> {
        let rest = std::mem::take(rest);
        let name = format_ident!("DEBUG_{}", data.ident, span = data.ident.span());
        Ok(TransformOutput::Consume {
            data: quote!(macro_rules! #name {
                () => {
                    stringify! {
                        data={#data}
                        args={#args}
                        rest={#rest}
                    }
                };
            }),
        })
    }
}

pub struct Finish;

impl Command for Finish {
    type Args = Nothing;

    fn execute(data: DeriveInput, _: Self::Args, _: &mut TokenStream) -> Result<TransformOutput> {
        Ok(TransformOutput::Consume {
            data: data.into_token_stream(),
        })
    }
}
