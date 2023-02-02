use crate::{kw, NamedArg, TransformRest, TransformState};
use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    DeriveInput, Result,
};

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let PredefinedInput { args, data, plus } = syn::parse2(input)?;
    let PredefinedArgs { rest } = args.content;
    TransformState::Pipe {
        data: data.content,
        pipe: None,
        plus: Some(plus.content),
    }
    .transform(rest.content)
}

pub struct PredefinedInput {
    pub args: NamedArg<kw::args, PredefinedArgs>,
    pub data: NamedArg<kw::data, DeriveInput>,
    pub plus: NamedArg<kw::plus, TokenStream>,
}

impl Parse for PredefinedInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            args: input.parse()?,
            data: input.parse()?,
            plus: input.parse()?,
        })
    }
}

pub struct PredefinedArgs {
    pub rest: NamedArg<kw::rest, TransformRest>,
}

impl Parse for PredefinedArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            rest: input.parse()?,
        })
    }
}
