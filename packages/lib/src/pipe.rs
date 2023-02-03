use crate::{ListOf, TransformRest, TransformState, Transformer};
use proc_macro2::TokenStream;
use quote::{quote_spanned, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    token, DeriveInput, Path, Result, Token,
};

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let PipeInput { path, pipe } = syn::parse2(input)?;
    TransformState::resume(path.clone())
        .pipe(pipe)
        .build()
        .transform(TransformRest::empty(path))
}

pub struct PipeInput {
    path: Path,
    pipe: ListOf<PipeCommand>,
}

impl Parse for PipeInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            path: input.parse()?,
            pipe: input.parse()?,
        })
    }
}

#[derive(Clone)]
pub struct PipeCommand {
    r_arrow_token: Token![->],
    path: Path,
    paren_token: token::Paren,
    args: TokenStream,
}

impl PipeCommand {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn execute_as<T: Transformer>(
        self,
        data: DeriveInput,
        rest: &mut TransformRest,
    ) -> Result<TransformState> {
        T::transform(data, syn::parse2(self.args)?, rest)
    }

    pub(crate) fn execute(self, data: DeriveInput, rest: TransformRest) -> TokenStream {
        let span = rest.span();
        let PipeCommand { path, args, .. } = self;
        // TODO: state::Execute
        quote_spanned!(span=>
            #path! {
                data={#data}
                args={#args}
                rest={#rest}
            }
        )
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
