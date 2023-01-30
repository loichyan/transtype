use crate::{
    define::{Define, QuoteExecution},
    extend::Extend,
    kw,
    pipe::PipeCommand,
    rename::Rename,
    wrap::Wrap,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Parser},
    DeriveInput, Result,
};
use transtype_lib::{CommandOutput, NamedArg};

pub fn expand(input: TransformInput) -> Result<TokenStream> {
    let mut data = input.data.content;
    let mut pipe = input.args.content;
    let rest = input.rest.content;
    pipe.extend(rest);
    (|input: ParseStream| {
        Ok(loop {
            if input.is_empty() {
                break data.into_token_stream();
            }
            let TransformCmd { which, cmd } = input.parse::<TransformCmd>()?;
            let output = match which {
                Which::Define => cmd.execute_as::<Define>(data)?,
                Which::Extend => cmd.execute_as::<Extend>(data)?,
                Which::Rename => cmd.execute_as::<Rename>(data)?,
                Which::Wrap => cmd.execute_as::<Wrap>(data)?,
                Which::Undefined => {
                    break QuoteExecution {
                        path: &cmd.path,
                        args: Some(&cmd.args),
                        data: Some(&data),
                        rest: Some(&input.parse()?),
                    }
                    .into_token_stream()
                }
            };
            if let CommandOutput::Piped(d) = output {
                data = d;
                continue;
            }
            let rest = input.parse::<TokenStream>()?;
            match output {
                CommandOutput::Consumed(tokens) => {
                    if !rest.is_empty() {
                        return Err(syn::Error::new_spanned(
                            rest,
                            "a consumer command should not be followed with other commands",
                        ));
                    }
                    break tokens;
                }
                CommandOutput::Transformed { path, data, args } => {
                    break quote! {
                        #path! {
                            data={#data}
                            args={#args}
                            rest={#rest}
                        }
                    };
                }
                _ => unreachable!(),
            }
        })
    })
    .parse2(pipe)
}

struct TransformCmd {
    which: Which,
    cmd: PipeCommand,
}

#[derive(Clone, Copy)]
enum Which {
    Define,
    Extend,
    Rename,
    Wrap,
    Undefined,
}

impl Parse for TransformCmd {
    fn parse(input: ParseStream) -> Result<Self> {
        const CMDS: &[(&str, Which)] = &[
            ("define", Which::Define),
            ("extend", Which::Extend),
            ("rename", Which::Rename),
            ("wrap", Which::Wrap),
        ];

        let cmd = input.parse::<PipeCommand>()?;
        let mut which = Which::Undefined;
        if let Some(ident) = cmd.path.get_ident() {
            if let Ok(i) =
                CMDS.binary_search_by_key::<&str, _>(&ident.to_string().as_str(), |(s, _)| s)
            {
                which = CMDS[i].1;
            }
        }
        Ok(Self { which, cmd })
    }
}

pub struct TransformInput {
    data: NamedArg<kw::data, DeriveInput>,
    args: NamedArg<kw::args, TokenStream>,
    rest: NamedArg<kw::rest, TokenStream>,
}

impl Parse for TransformInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            data: input.parse()?,
            args: input.parse()?,
            rest: input.parse()?,
        })
    }
}
