use proc_macro2::TokenStream;
use quote::ToTokens;
use std::ops::{Deref, DerefMut};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    token, Result,
};

#[derive(Clone)]
pub struct ListOf<T>(Vec<T>);

impl<T> ListOf<T> {
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

impl<T> IntoIterator for ListOf<T> {
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T, U> FromIterator<U> for ListOf<T>
where
    U: Into<T>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        Self(iter.into_iter().map(U::into).collect())
    }
}

impl<T> Deref for ListOf<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ListOf<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Default for ListOf<T> {
    fn default() -> Self {
        Self(Vec::default())
    }
}

impl<T: Parse> Parse for ListOf<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut inner = Vec::default();
        loop {
            if input.is_empty() {
                break;
            }
            inner.push(input.parse()?);
        }
        Ok(Self(inner))
    }
}

impl<T: ToTokens> ToTokens for ListOf<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for t in &self.0 {
            t.to_tokens(tokens);
        }
    }
}

#[derive(Clone, Default)]
pub struct NamedArg<K, V> {
    pub name: K,
    pub eq_token: token::Eq,
    pub brace_token: token::Brace,
    pub content: V,
}

impl<K, V> NamedArg<K, V>
where
    K: Default,
{
    pub fn new(content: V) -> Self {
        Self {
            name: Default::default(),
            eq_token: Default::default(),
            brace_token: Default::default(),
            content,
        }
    }
}

impl<K, V> NamedArg<K, V>
where
    K: Clone,
    V: Default,
{
    pub fn take(&mut self) -> Self {
        Self {
            name: self.name.clone(),
            eq_token: self.eq_token,
            brace_token: self.brace_token,
            content: std::mem::take(&mut self.content),
        }
    }
}

impl<K, V> Parse for NamedArg<K, V>
where
    K: Parse,
    V: Parse,
{
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            name: input.parse()?,
            eq_token: input.parse()?,
            brace_token: braced!(content in input),
            content: content.parse()?,
        })
    }
}

impl<K, V> ToTokens for NamedArg<K, V>
where
    K: ToTokens,
    V: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.name.to_tokens(tokens);
        self.eq_token.to_tokens(tokens);
        self.brace_token
            .surround(tokens, |tokens| self.content.to_tokens(tokens));
    }
}

pub struct Optional<T>(Option<T>);

impl<T> Deref for Optional<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Optional<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Optional<T> {
    pub fn into_inner(self) -> Option<T> {
        self.0
    }
}

impl<T: Parse> Parse for Optional<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            Ok(Self(None))
        } else {
            Ok(Self(Some(input.parse()?)))
        }
    }
}

impl<T: ToTokens> ToTokens for Optional<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(t) = &self.0 {
            t.to_tokens(tokens)
        }
    }
}
