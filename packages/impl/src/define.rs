use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

pub fn expand(data: DeriveInput) -> Result<TokenStream> {
    let name = &data.ident;
    Ok(quote!(macro_rules! #name {
        ($($args:tt)*) => {
            ::transtype::predefined! {
                args={$($args)*}
                data={#data}
                save={}
            }
        };
    }))
}
