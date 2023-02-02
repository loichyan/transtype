use crate::{kw, DefaultExecutor, Executor, TransformState, Transformer};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::ops::{Deref, DerefMut};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token, DeriveInput, Path, Result, Token,
};

pub struct TransformInput<T: Transformer> {
    data: NamedArg<kw::data, DeriveInput>,
    args: NamedArg<kw::args, T::Args>,
    rest: NamedArg<kw::rest, TransformRest>,
}

impl<T: Transformer> TransformInput<T> {
    pub fn transform(self) -> Result<TokenStream> {
        self.transform_with::<DefaultExecutor>()
    }

    pub fn transform_with<Exe: Executor>(self) -> Result<TokenStream> {
        let mut rest = self.rest.content;
        T::transform(self.data.content, self.args.content, &mut rest)?.transform_with::<Exe>(rest)
    }
}

impl<T: Transformer> Parse for TransformInput<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            data: input.parse()?,
            args: input.parse()?,
            rest: input.parse()?,
        })
    }
}

pub struct TransformRest {
    this: NamedArg<kw::this, Path>,
    pipe: NamedArg<kw::pipe, ListOf<PipeCommand>>,
    plus: NamedArg<kw::plus, TokenStream>,
}

impl TransformRest {
    pub(crate) fn empty(path: Path) -> Self {
        Self {
            this: NamedArg::new(path),
            pipe: Default::default(),
            plus: Default::default(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.pipe.content.is_empty()
    }

    pub(crate) fn next_pipe(&mut self) -> Option<PipeCommand> {
        self.pipe.content.pop()
    }

    pub(crate) fn prepend_pipe(&mut self, pipe: ListOf<PipeCommand>) {
        self.pipe
            .content
            .extend(pipe.into_inner().into_iter().rev());
    }

    pub(crate) fn prepend_plus(&mut self, plus: TokenStream) {
        self.plus.content.extend(plus);
    }

    pub(crate) fn take_plus(&mut self) -> TokenStream {
        std::mem::take(&mut self.plus.content)
    }
}

impl Parse for TransformRest {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            this: input.parse()?,
            pipe: input.parse()?,
            plus: input.parse()?,
        })
    }
}

impl ToTokens for TransformRest {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.this.to_tokens(tokens);
        self.pipe.to_tokens(tokens);
        self.plus.to_tokens(tokens);
    }
}

pub struct PipeCommand {
    r_arrow_token: Token![->],
    path: Path,
    paren_token: token::Paren,
    args: TokenStream,
}

impl PipeCommand {
    pub fn new(path: Path, args: TokenStream) -> Self {
        Self {
            r_arrow_token: Default::default(),
            path,
            paren_token: Default::default(),
            args,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn execute_as<T: Transformer>(
        self,
        data: DeriveInput,
        rest: &mut TransformRest,
    ) -> Result<TransformState> {
        let span = self.path.span();
        (|| T::transform(data, syn::parse2(self.args)?, rest))().map_err(|mut e| {
            e.combine(syn::Error::new(span, "an error occurs in this command"));
            e
        })
    }

    pub(crate) fn execute(self, data: DeriveInput, rest: TransformRest) -> TokenStream {
        let PipeCommand { path, args, .. } = self;
        quote!(#path! {
            data={#data}
            args={#args}
            rest={#rest}
        })
    }
}

impl From<(Path, TokenStream)> for PipeCommand {
    fn from((path, args): (Path, TokenStream)) -> Self {
        Self::new(path, args)
    }
}

impl Parse for PipeCommand {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            r_arrow_token: input.parse()?,
            path: input.parse()?,
            paren_token: parenthesized!(content in input),
            args: content.parse()?,
        })
    }
}

impl ToTokens for PipeCommand {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.r_arrow_token.to_tokens(tokens);
        self.path.to_tokens(tokens);
        self.paren_token
            .surround(tokens, |tokens| self.args.to_tokens(tokens));
    }
}

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
