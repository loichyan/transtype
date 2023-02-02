use crate::{kw, NamedArg, Optional, TransformRest, TransformState};
use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    DeriveInput, Result,
};

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let PredefinedInput { data, save, args } = syn::parse2(input)?;
    let PredefinedArgs { rest } = args.content;
    let mut rest = rest.content;
    if let Some(save) = save.content.into_inner() {
        rest.prepend(save);
    }
    TransformState::pipe(data.content).transform(rest)
}

pub struct PredefinedInput {
    pub args: NamedArg<kw::args, PredefinedArgs>,
    pub data: NamedArg<kw::data, DeriveInput>,
    pub save: NamedArg<kw::save, Optional<TransformRest>>,
}

impl Parse for PredefinedInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            args: input.parse()?,
            data: input.parse()?,
            save: input.parse()?,
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
