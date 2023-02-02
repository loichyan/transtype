use crate::kw;
use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    DeriveInput, Result,
};
use transtype_lib::{NamedArg, Optional, TransformRest, TransformState};

pub type PredefinedArgs = transtype_lib::PredefinedInput;

pub fn expand(input: PredefinedInput) -> Result<TokenStream> {
    let PredefinedInput { data, save, args } = input;
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
