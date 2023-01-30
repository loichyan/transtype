use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Nothing as SynNothing, Parse, ParseStream},
    token, Result,
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

#[doc(hidden)]
macro_rules! __delimited_impl {
    ($content:ident in $lookahead:ident => $fallback:expr) => {{
        let input = $lookahead;
        let $lookahead = $lookahead.lookahead1();
        if $lookahead.peek(token::Brace) {
            $crate::utils::Delimiter::Brace(::syn::braced!($content in input))
        } else if $lookahead.peek(token::Bracket) {
            $crate::utils::Delimiter::Bracket(::syn::bracketed!($content in input))
        } else if $lookahead.peek(token::Paren) {
            $crate::utils::Delimiter::Paren(::syn::parenthesized!($content in input))
        } else {
            $fallback
        }
    }};
}

macro_rules! delimited {
    ($content:ident in $cursor:expr => $fallback:expr) => {{
        let lookahead = $cursor;
        __delimited_impl! {
            $content in lookahead
            => $fallback
        }
    }};
    ($content:ident in $cursor:expr) => {{
        let lookahead = $cursor;
        __delimited_impl! {
            $content in lookahead
            => {
                return Err(lookahead.error());
            }
        }
    }};
}

#[derive(Clone, Copy)]
pub enum Delimiter {
    Brace(token::Brace),
    Bracket(token::Bracket),
    Paren(token::Paren),
}

impl Delimiter {
    #[allow(dead_code)]
    pub fn surround(&self, tokens: &mut TokenStream, f: impl FnOnce(&mut TokenStream)) {
        match self {
            Delimiter::Brace(t) => t.surround(tokens, f),
            Delimiter::Bracket(t) => t.surround(tokens, f),
            Delimiter::Paren(t) => t.surround(tokens, f),
        }
    }
}
