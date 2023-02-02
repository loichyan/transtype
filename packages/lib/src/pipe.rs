use crate::{ListOf, PipeCommand, TransformRest, TransformState};
use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    Path, Result,
};

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let PipeInput { path, pipe } = syn::parse2(input)?;
    TransformState::Start {
        path: path.clone(),
        pipe: Some(pipe),
        plus: None,
    }
    .transform(TransformRest::empty(path))
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
