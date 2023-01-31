use crate::{
    ast::{ListOf, PipeCommand},
    debug::Debug,
    define::Define,
    extend::Extend,
    finish::Finish,
    kw,
    rename::Rename,
    select::{Select, SelectAttr},
    wrap::{Wrap, Wrapped},
};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    token, DeriveInput, Result, Token,
};
use transtype_lib::{NamedArg, TransformOutput, Transformer};

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
        let mut rest = if rest_tokens.is_empty() {
            TransformRest::default()
        } else {
            syn::parse2::<TransformRest>(std::mem::take(rest_tokens))?
        };
        rest.extend_args(args)?;
        let output = if let Some(mut data) = data.0 {
            loop {
                // 2) Execute pipe commands.
                let pipes = &mut rest.pipe.content.0;
                let output = loop {
                    if let Some(cmd) = pipes.pop() {
                        let output = match maybe_builtin(&cmd) {
                            Some(builtin) => builtin.execute(cmd, data)?,
                            None => {
                                break TransformOutput::Transferred {
                                    path: cmd.path,
                                    data: Some(data),
                                    args: cmd.args,
                                }
                            }
                        };
                        match output {
                            TransformOutput::Piped { data: d } => data = d,
                            _ => break output,
                        }
                    } else {
                        break TransformOutput::Piped { data };
                    }
                };
                match output {
                    TransformOutput::Transformed { data: d, args: a } => {
                        data = d;
                        rest.extend_args(a)?;
                    }
                    _ => break output,
                }
            }
        } else {
            TransformOutput::Consumed {
                data: Default::default(),
            }
        };
        let pipes = &mut rest.pipe.content;
        let withs = &mut rest.with.content;
        Ok(match output {
            TransformOutput::Consumed { mut data } => {
                if !pipes.0.is_empty() {
                    return Err(syn::Error::new_spanned(
                        pipes,
                        "a consumer command should not be followed with other commands",
                    ));
                }
                data.extend(std::mem::take(withs));
                TransformOutput::Consumed { data }
            }
            TransformOutput::Piped { .. } => {
                return Err(syn::Error::new_spanned(
                    pipes,
                    "a pipe command should be consumed",
                ))
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
        syn::parse2::<ListOf<TransformCommand>>(args)?
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
            ) -> Result<TransformOutput> {
                match self {
                    $(Self::$variant => cmd.execute::<$variant>(data),)*
                }
            }
        }

    };
}

builtins! {
    #[derive(Clone, Copy, Debug)]
    enum Builtin {
        debug       => Debug;
        define      => Define;
        extend      => Extend;
        finish      => Finish;
        rename      => Rename;
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
