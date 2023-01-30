use crate::{ast::PipeCommand, define::Define, extend::Extend, rename::Rename, wrap::Wrap};
use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream, Parser},
    DeriveInput, Result,
};
use transtype_lib::{Command, TransformOutput};

pub type TransformInput = transtype_lib::TransformInput<Transform>;

pub struct Transform;

impl Command for Transform {
    type Args = TokenStream;

    fn execute(
        mut data: DeriveInput,
        mut args: Self::Args,
        rest: &mut TokenStream,
    ) -> Result<TransformOutput> {
        args.extend(std::mem::take(rest));
        (|input: ParseStream| {
            let output = loop {
                if input.is_empty() {
                    break TransformOutput::Piped { data };
                }
                let TransformCmd { which, cmd } = input.parse::<TransformCmd>()?;
                let output = match which {
                    Which::Define => cmd.execute::<Define>(data)?,
                    Which::Extend => cmd.execute::<Extend>(data)?,
                    Which::Rename => cmd.execute::<Rename>(data)?,
                    Which::Wrap => cmd.execute::<Wrap>(data)?,
                    Which::Undefined => {
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
            };
            *rest = input.parse()?;
            Ok(output)
        })
        .parse2(args)
    }
}

pub fn expand(input: TransformInput) -> Result<TokenStream> {
    input.transform()
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
