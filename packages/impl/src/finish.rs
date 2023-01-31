use crate::{define::QuoteDefinition, kw};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    DeriveInput, Result,
};
use transtype_lib::{Command, TransformOutput};

pub struct Finish;

impl Command for Finish {
    type Args = FinishArgs;

    fn execute(
        data: DeriveInput,
        args: Self::Args,
        _: &mut TokenStream,
    ) -> Result<TransformOutput> {
        let defined = if args.definied.is_some() {
            Some(QuoteDefinition {
                ident: &data.ident,
                data: &data,
            })
        } else {
            None
        };
        Ok(TransformOutput::Consumed {
            data: quote!(#data #defined),
        })
    }
}

pub struct FinishArgs {
    pub definied: Option<kw::defined>,
}

impl Parse for FinishArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if input.peek(kw::defined) {
            Self {
                definied: Some(input.parse()?),
            }
        } else {
            Self { definied: None }
        })
    }
}
