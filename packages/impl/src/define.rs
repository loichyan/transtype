use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

pub fn expand(data: DeriveInput) -> Result<TokenStream> {
    let name = &data.ident;
    Ok(quote!(macro_rules! #name {
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
    }))
}
