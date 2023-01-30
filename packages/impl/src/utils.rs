use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Nothing as SynNothing, Parse, ParseStream},
    Result,
};

pub struct Nothing(SynNothing);

impl Default for Nothing {
    fn default() -> Self {
        Self(SynNothing)
    }
}

impl Parse for Nothing {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse().map(Nothing)
    }
}

impl ToTokens for Nothing {
    fn to_tokens(&self, _: &mut TokenStream) {}
}
