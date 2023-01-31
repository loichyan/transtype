use crate::{
    ast::PipeCommand,
    debug::Debug,
    define::Define,
    extend::Extend,
    finish::Finish,
    rename::Rename,
    select::{Select, SelectAttr},
    wrap::{Wrap, Wrapped},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream, Parser},
    token, DeriveInput, Result, Token,
};
use transtype_lib::{Command, TransformOutput};

pub type TransformInput = transtype_lib::TransformInput<Transform>;

pub fn expand(input: TransformInput) -> Result<TokenStream> {
    input.transform()
}

pub struct Transform;

impl Command for Transform {
    type Args = TokenStream;

    fn execute(
        mut data: DeriveInput,
        mut args: Self::Args,
        rest: &mut TokenStream,
    ) -> Result<TransformOutput> {
        let mut addition = TokenStream::default();
        Ok(loop {
            args.extend(std::mem::take(rest));
            let output = (|input: ParseStream| {
                let output = loop {
                    if input.is_empty() {
                        break TransformOutput::Piped { data };
                    }
                    let output = match input.parse::<TransformCommand>()? {
                        TransformCommand::Pipe(cmd) => match maybe_builtin(&cmd) {
                            Some(builtin) => builtin.execute(cmd, data)?,
                            None => {
                                break TransformOutput::Transferred {
                                    path: cmd.path,
                                    data: Some(data),
                                    args: cmd.args,
                                }
                            }
                        },
                        TransformCommand::Add(cmd) => {
                            addition.extend(cmd.content);
                            continue;
                        }
                    };
                    match output {
                        TransformOutput::Piped { data: d } => data = d,
                        _ => break output,
                    }
                };
                *rest = input.parse()?;
                Ok(output)
            })
            .parse2(args)?;
            match output {
                TransformOutput::Transformed { data: d, args: a } => {
                    data = d;
                    args = a;
                }
                TransformOutput::Consumed { mut data } => {
                    data.extend(addition);
                    break TransformOutput::Consumed { data };
                }
                _ => {
                    if !addition.is_empty() {
                        rest.extend(quote!(+{ #addition }));
                    }
                    break output;
                }
            }
        })
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
    pub content: TokenStream,
}

impl Parse for AddCommand {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            add_token: input.parse()?,
            brace_token: braced!(content in input),
            content: content.parse()?,
        })
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
