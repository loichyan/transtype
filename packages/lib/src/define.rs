use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{spanned::Spanned, DeriveInput, Result};

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let data = syn::parse2::<DeriveInput>(input)?;
    let span = data.span();
    let name = &data.ident;
    Ok(quote_spanned!(span=>
        macro_rules! #name {
            ($($args:tt)*) => {
                ::transtype::__predefined! {
                    args={$($args)*}
                    data={#data}
                    extra={}
                }
            };
        }
    ))
}
