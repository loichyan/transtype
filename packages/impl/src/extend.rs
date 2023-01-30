use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    token, Data, DeriveInput, Fields, Path, Result, Token,
};
use transtype_lib::{Command, CommandOutput};

pub struct Extend;

impl Command for Extend {
    type Args = ExtendArgs;

    fn execute(data: DeriveInput, args: Self::Args) -> Result<CommandOutput> {
        Ok(match args {
            ExtendArgs::Path(path) => CommandOutput::Transformed {
                path,
                data: None,
                args: quote!(
                    => extend(@#data)
                ),
            },
            ExtendArgs::Into(ExtendInto { data: mut dest, .. }) => {
                match (&mut dest.data, data.data) {
                    (Data::Struct(dest), Data::Struct(src)) => {
                        match (&mut dest.fields, src.fields) {
                            (Fields::Named(dest), Fields::Named(src)) => {
                                dest.named.extend(src.named);
                            }
                            _ => todo!(),
                        }
                    }
                    _ => todo!(),
                }
                CommandOutput::Piped(dest)
            }
            ExtendArgs::Struct(_) => todo!(),
        })
    }
}

pub enum ExtendArgs {
    Path(Path),
    Into(ExtendInto),
    Struct(ExtendStruct),
}

impl Parse for ExtendArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![struct]) {
            input.parse().map(Self::Struct)
        } else if input.peek(Token![@]) {
            input.parse().map(Self::Into)
        } else {
            input.parse().map(Self::Path)
        }
    }
}

pub struct ExtendInto {
    pub at_tk: Token![@],
    pub data: DeriveInput,
}

impl Parse for ExtendInto {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            at_tk: input.parse()?,
            data: input.parse()?,
        })
    }
}

pub struct ExtendStruct {
    pub struct_tk: Token![struct],
    pub fields: Fields,
    pub semi: Option<Token![;]>,
}

impl Parse for ExtendStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let struct_tk = input.parse()?;
        let fields;
        let semi;
        if input.peek(Token![;]) {
            fields = Fields::Unit;
            semi = Some(input.parse()?);
        } else if input.peek(token::Paren) {
            fields = Fields::Unnamed(input.parse()?);
            semi = None;
        } else {
            fields = Fields::Named(input.parse()?);
            semi = None;
        }
        Ok(Self {
            struct_tk,
            fields,
            semi,
        })
    }
}
