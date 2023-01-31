use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    token, DeriveInput, Path, Result, Token,
};

mod kw {
    use syn::custom_keyword;

    custom_keyword!(data);
    custom_keyword!(args);
    custom_keyword!(rest);
}

pub trait Command {
    type Args: Parse;

    fn execute(
        data: DeriveInput,
        args: Self::Args,
        rest: &mut TokenStream,
    ) -> Result<TransformOutput>;
}

impl<T: Command> Transformer for T {
    type Data = DeriveInput;
    type Args = <T as Command>::Args;

    fn transform(
        data: Self::Data,
        args: Self::Args,
        rest: &mut TokenStream,
    ) -> Result<TransformOutput> {
        T::execute(data, args, rest)
    }
}

pub trait Transformer: Sized {
    type Data: Parse;
    type Args: Parse;

    fn transform(
        data: Self::Data,
        args: Self::Args,
        rest: &mut TokenStream,
    ) -> Result<TransformOutput>;
}

pub struct TransformInput<T: Transformer> {
    data: NamedArg<kw::data, T::Data>,
    args: NamedArg<kw::args, T::Args>,
    rest: NamedArg<kw::rest, TokenStream>,
}

impl<T: Transformer> TransformInput<T> {
    pub fn transform(self) -> Result<TokenStream> {
        let data = self.data.content;
        let args = self.args.content;
        let mut rest = self.rest.content;
        Ok(match T::transform(data, args, &mut rest)? {
            TransformOutput::Piped { data } => {
                if rest.is_empty() {
                    return Err(syn::Error::new_spanned(
                        rest,
                        "a pipe command should be consumed",
                    ));
                }
                quote!(::transtype::transform! {
                    data={#data}
                    args={}
                    rest={#rest}
                })
            }
            TransformOutput::Consumed { data } => {
                if !rest.is_empty() {
                    return Err(syn::Error::new_spanned(
                        rest,
                        "a consumer command should not be followed with other commands",
                    ));
                }
                data.into_token_stream()
            }
            TransformOutput::Transferred { path, data, args } => quote!(#path! {
                data={#data}
                args={#args}
                rest={#rest}
            }),
        })
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

pub enum TransformOutput {
    Piped {
        data: DeriveInput,
    },
    Consumed {
        data: TokenStream,
    },
    Transferred {
        path: Path,
        data: Option<DeriveInput>,
        args: TokenStream,
    },
}

struct NamedArg<K, V> {
    name: K,
    eq_tk: Token![=],
    brace_tk: token::Brace,
    content: V,
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
            eq_tk: input.parse()?,
            brace_tk: braced!(content in input),
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
        self.eq_tk.to_tokens(tokens);
        self.brace_tk
            .surround(tokens, |tokens| self.content.to_tokens(tokens));
    }
}
