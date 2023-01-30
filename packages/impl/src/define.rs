use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{DeriveInput, Ident, Path, Result};
use transtype_lib::{Command, CommandOutput};

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

    fn execute(data: DeriveInput, name: Self::Args) -> Result<CommandOutput> {
        Ok(CommandOutput::Consumed(
            QuoteDefinition {
                ident: &name,
                data: &data,
            }
            .into_token_stream(),
        ))
    }
}

pub struct QuoteExecution<'a> {
    pub path: &'a Path,
    pub data: Option<&'a DeriveInput>,
    pub args: Option<&'a TokenStream>,
    pub rest: Option<&'a TokenStream>,
}

impl ToTokens for QuoteExecution<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            path,
            data,
            args,
            rest,
        } = self;
        tokens.extend(quote!(
            #path! {
                data={#data}
                args={#args}
                rest={#rest}
            }

        ))
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
