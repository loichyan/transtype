use proc_macro2::TokenStream;
use quote::ToTokens;
use std::borrow::BorrowMut;
use syn::{
    parse::{Nothing as SynNothing, Parse, ParseStream},
    punctuated::Punctuated,
    token, Data, DeriveInput, Field, Fields, Ident, Path, Result, Token,
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

#[derive(Clone, Copy)]
#[allow(dead_code)]
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

pub struct Selectors(pub Punctuated<Selector, Token![,]>);

impl Parse for Selectors {
    fn parse(input: ParseStream) -> Result<Self> {
        Punctuated::parse_terminated(input).map(Self)
    }
}

impl Selectors {
    pub fn select(&self, name: &Ident) -> Option<Ident> {
        for arg in self.0.iter() {
            match &arg.name {
                WildName::Wild(_) => return Some(name.clone()),
                WildName::Name(pat) if name == pat => match &arg.rename {
                    Some(WildName::Name(rename)) => return Some(rename.clone()),
                    None => return Some(name.clone()),
                    _ => return None,
                },
                _ => {}
            }
        }
        None
    }
}

pub struct Selector {
    pub name: WildName,
    pub as_token: Option<Token![as]>,
    pub rename: Option<WildName>,
}

impl Parse for Selector {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        if input.peek(Token![as]) {
            Ok(Self {
                name,
                as_token: Some(input.parse()?),
                rename: Some(input.parse()?),
            })
        } else {
            Ok(Self {
                name,
                as_token: None,
                rename: None,
            })
        }
    }
}

pub enum WildName {
    Wild(Token![_]),
    Name(Ident),
}

impl Parse for WildName {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![_]) {
            input.parse().map(Self::Wild)
        } else if lookahead.peek(Ident) {
            input.parse().map(Self::Name)
        } else {
            Err(lookahead.error())
        }
    }
}

trait BorrowMut2 {
    fn borrow_mut2<T>(&mut self) -> &mut T
    where
        T: ?Sized,
        Self: std::borrow::BorrowMut<T>,
    {
        std::borrow::BorrowMut::borrow_mut(self)
    }
}

impl<T: ?Sized> BorrowMut2 for T {}

pub trait DeriveInputExt: BorrowMut<DeriveInput> {
    fn fields_iter<'a>(
        &'a mut self,
    ) -> Box<dyn 'a + Iterator<Item = &'a mut Punctuated<Field, Token![,]>>> {
        match &mut self.borrow_mut2::<DeriveInput>().data {
            Data::Struct(data) => Box::new(data.fields.get_fields().into_iter()),
            Data::Enum(data) => Box::new(
                data.variants
                    .iter_mut()
                    .filter_map(|variant| variant.fields.get_fields()),
            ),
            Data::Union(data) => Box::new(std::iter::once(&mut data.fields.named)),
        }
    }
}

impl<T: BorrowMut<DeriveInput>> DeriveInputExt for T {}

pub trait FieldsExt: BorrowMut<Fields> {
    fn get_fields(&mut self) -> Option<&mut Punctuated<Field, Token![,]>> {
        match self.borrow_mut2::<Fields>() {
            Fields::Named(fields) => Some(&mut fields.named),
            Fields::Unnamed(fields) => Some(&mut fields.unnamed),
            Fields::Unit => None,
        }
    }
}

impl<T: BorrowMut<Fields>> FieldsExt for T {}

pub trait PathExt: BorrowMut<Path> {
    fn get_ident_mut(&mut self) -> Option<&mut Ident> {
        let path = self.borrow_mut2::<Path>();
        if path.leading_colon.is_none()
            && path.segments.len() == 1
            && path.segments[0].arguments.is_none()
        {
            Some(&mut path.segments[0].ident)
        } else {
            None
        }
    }
}

impl<T: BorrowMut<Path>> PathExt for T {}

pub trait TokenStreamExt: Into<TokenStream> {
    fn parse2<T: Parse>(self) -> Result<T> {
        syn::parse2(self.into())
    }
}

impl<T: Into<TokenStream>> TokenStreamExt for T {}
