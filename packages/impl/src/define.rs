use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{DeriveInput, Ident, Result};
use transtype_lib::{Command, TransformOutput};

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    Ok(QuoteDefinition {
        ident: &input.ident,
        data: &input,
    }
    .into_token_stream())
}

pub struct Define;

impl Command for Define {
    type Args = Ident;

    fn execute(
        data: DeriveInput,
        name: Self::Args,
        _: &mut TokenStream,
    ) -> Result<TransformOutput> {
        Ok(TransformOutput::Consumed {
            data: QuoteDefinition {
                ident: &name,
                data: &data,
            }
            .into_token_stream(),
        })
    }
}

pub struct QuoteDefinition<'a> {
    pub ident: &'a Ident,
    pub data: &'a DeriveInput,
}

impl ToTokens for QuoteDefinition<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { ident, data } = self;
        tokens.extend(quote!(
            macro_rules! #ident {
                (
                    data={}
                    args=$args:tt
                    rest=$rest:tt
                ) => {
                    ::transtype::transform! {
                        data={#data}
                        args=$args
                        rest=$rest
                    }
                };
            }
        ))
    }
}
