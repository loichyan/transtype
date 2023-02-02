use crate::{ListOf, NamedArg, PipeCommand};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Ident, Result,
};

#[derive(Clone)]
pub struct ForkCommand(pub(crate) NamedArg<Ident, ListOf<PipeCommand>>);

impl Parse for ForkCommand {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse().map(Self)
    }
}

impl ToTokens for ForkCommand {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}
