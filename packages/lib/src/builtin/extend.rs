use crate::{TransformRest, TransformState, Transformer};
use quote::quote_spanned;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote_spanned, token, Data, DeriveInput, Fields, Path, Result, Token,
};

pub struct Extend;

impl Transformer for Extend {
    type Args = ExtendArgs;

    fn transform(
        data: DeriveInput,
        args: Self::Args,
        rest: &mut TransformRest,
    ) -> Result<TransformState> {
        let span = rest.span();
        Ok(match args {
            ExtendArgs::Path(path) => TransformState::Start {
                path,
                pipe: Some(
                    [(
                        parse_quote_spanned!(span=> extend),
                        quote_spanned!(span=> as #data),
                    )]
                    .into_iter()
                    .collect(),
                ),
                plus: None,
            },
            ExtendArgs::As(ExtendAs { data: mut dest, .. }) => {
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
                TransformState::pipe(dest)
            }
            ExtendArgs::Struct(_) => todo!(),
        })
    }
}

pub enum ExtendArgs {
    Path(Path),
    As(ExtendAs),
    Struct(ExtendStruct),
}

impl Parse for ExtendArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![struct]) {
            input.parse().map(Self::Struct)
        } else if input.peek(Token![as]) {
            input.parse().map(Self::As)
        } else {
            input.parse().map(Self::Path)
        }
    }
}

pub struct ExtendAs {
    pub as_token: Token![as],
    pub data: DeriveInput,
}

impl Parse for ExtendAs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            as_token: input.parse()?,
            data: input.parse()?,
        })
    }
}

pub struct ExtendStruct {
    pub struct_token: Token![struct],
    pub fields: Fields,
}

impl Parse for ExtendStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            struct_token: input.parse()?,
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
