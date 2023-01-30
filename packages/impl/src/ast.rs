use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Nothing as SynNothing, Parse, ParseStream},
    token, DeriveInput, Fields, Path, Result, Token,
};
use transtype_lib::{Command, TransformOutput};

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

macro_rules! delimited {
    ($content:ident in $cursor:expr) => {{
        let input = $cursor;
        let lookahead = input.lookahead1();
        if lookahead.peek(token::Brace) {
            $crate::ast::Delimiter::Brace(::syn::braced!($content in input))
        } else if lookahead.peek(token::Bracket) {
            $crate::ast::Delimiter::Bracket(::syn::bracketed!($content in input))
        } else if lookahead.peek(token::Paren) {
            $crate::ast::Delimiter::Paren(::syn::parenthesized!($content in input))
        } else {
            return Err(lookahead.error());
        }
    }};
}

pub struct PipeCommand {
    pub fat_arrow: Token![=>],
    pub path: Path,
    pub delimiter: Delimiter,
    pub args: TokenStream,
}

impl PipeCommand {
    pub fn execute<T: Command>(self, data: DeriveInput) -> Result<TransformOutput> {
        let Self { path, args, .. } = self;
        T::execute(data, syn::parse2(args)?, &mut TokenStream::default()).map_err(|mut e| {
            e.combine(syn::Error::new_spanned(
                &path,
                "an error occurs in this command",
            ));
            e
        })
    }
}

impl Parse for PipeCommand {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            fat_arrow: input.parse()?,
            path: input.parse()?,
            delimiter: delimited!(content in input),
            args: content.parse()?,
        })
    }
}

#[derive(Clone, Copy)]
pub enum Delimiter {
    Brace(token::Brace),
    Bracket(token::Bracket),
    Paren(token::Paren),
    None,
}

impl Delimiter {
    pub fn from_feilds(fields: &Fields) -> Self {
        match fields {
            Fields::Named(fields) => Delimiter::Brace(fields.brace_token),
            Fields::Unnamed(fields) => Delimiter::Paren(fields.paren_token),
            Fields::Unit => Delimiter::None,
        }
    }

    pub fn surround(&self, tokens: &mut TokenStream, f: impl FnOnce(&mut TokenStream)) {
        match self {
            Delimiter::Brace(t) => t.surround(tokens, f),
            Delimiter::Bracket(t) => t.surround(tokens, f),
            Delimiter::Paren(t) => t.surround(tokens, f),
            Delimiter::None => f(tokens),
        }
    }
}
