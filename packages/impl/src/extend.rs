use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    token, Data, DeriveInput, Fields, Path, Result, Token,
};
use transtype_lib::{Command, TransformOutput};

pub struct Extend;

impl Command for Extend {
    type Args = ExtendArgs;

    fn execute(
        data: DeriveInput,
        args: Self::Args,
        _: &mut TokenStream,
    ) -> Result<TransformOutput> {
        Ok(match args {
            ExtendArgs::Path(path) => TransformOutput::Transferred {
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
                TransformOutput::Piped { data: dest }
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
}

impl Parse for ExtendStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            struct_tk: input.parse()?,
            fields: {
                let lookahead = input.lookahead1();
                if lookahead.peek(token::Paren) {
                    Fields::Unnamed(input.parse()?)
                } else if input.peek(token::Brace) {
                    Fields::Named(input.parse()?)
                } else {
                    return Err(lookahead.error());
                }
            },
        })
    }
}
