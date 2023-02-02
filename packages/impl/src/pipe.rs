use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    Path, Result,
};
use transtype_lib::{ListOf, PipeCommand, TransformRest, TransformState};

pub fn expand(input: PipeInput) -> Result<TokenStream> {
    let PipeInput { path, pipe } = input;
    TransformState::Start {
        path,
        pipe: Some(pipe),
        plus: None,
    }
    .transform(TransformRest::empty())
}

pub struct PipeInput {
    path: Path,
    pipe: ListOf<PipeCommand>,
}

impl Parse for PipeInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            path: input.parse()?,
            pipe: input.parse()?,
        })
    }
}
