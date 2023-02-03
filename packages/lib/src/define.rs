use crate::{state, TransformRest};
use proc_macro2::TokenStream;
use syn::{parse_quote_spanned, spanned::Spanned, DeriveInput, Result};

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let data = syn::parse2::<DeriveInput>(input)?;
    let span = data.span();
    let name = &data.ident;
    let path = parse_quote_spanned!(span=> #name);
    state::Save { data }
        .build()
        .transform(TransformRest::empty(path))
}
