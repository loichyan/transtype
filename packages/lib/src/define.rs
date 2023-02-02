use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let data = syn::parse2::<DeriveInput>(input)?;
    let name = &data.ident;
    Ok(quote!(macro_rules! #name {
        ($($args:tt)*) => {
            ::transtype::__predefined! {
                args={$($args)*}
                data={#data}
                save={}
            }
        };
    }))
}
