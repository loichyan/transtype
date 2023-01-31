use crate::{
    ast::PipeCommand,
    define::Define,
    extend::Extend,
    finish::Finish,
    rename::Rename,
    select::{Select, SelectAttr},
    wrap::{Wrap, Wrapped},
};
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
                let TransformCmd { builtin, cmd } = input.parse::<TransformCmd>()?;
                let output = if let Some(builtin) = builtin {
                    builtin.execute(cmd, data)?
                } else {
                    break TransformOutput::Transferred {
                        path: cmd.path,
                        data: Some(data),
                        args: cmd.args,
                    };
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
    builtin: Option<Builtin>,
    cmd: PipeCommand,
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

impl Parse for TransformCmd {
    fn parse(input: ParseStream) -> Result<Self> {
        let cmd = input.parse::<PipeCommand>()?;
        let mut builtin = None;
        if let Some(ident) = cmd.path.get_ident() {
            if let Ok(i) = Builtin::ALL
                .binary_search_by_key::<&str, _>(&ident.to_string().as_str(), |(s, _)| s)
            {
                builtin = Some(Builtin::ALL[i].1);
            }
        }
        Ok(Self { builtin, cmd })
    }
}
