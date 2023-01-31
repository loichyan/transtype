use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
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
                quote!(::transtype::transform! {
                    data={#data}
                    args={}
                    rest={#rest}
                })
            }
            TransformOutput::Consumed { data } => data,
            TransformOutput::Transferred { path, data, args } => {
                quote!(#path! {
                    data={#data}
                    args={#args}
                    rest={#rest}
                })
            }
            TransformOutput::Transformed { data, args } => {
                quote!(::transtype::transform! {
                    data={#data}
                    args={#args}
                    rest={#rest}
                })
            }
            TransformOutput::Debug { data, args } => {
                let name = format_ident!("DEBUG_{}", data.ident);
                quote!(macro_rules! #name {
                    () => {
                        stringify! {
                            data={#data}
                            args={#args}
                            rest={#rest}
                        }
                    };
                })
            }
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
    Transformed {
        data: DeriveInput,
        args: TokenStream,
    },
    Debug {
        data: DeriveInput,
        args: TokenStream,
    },
}

pub struct NamedArg<K, V> {
    pub name: K,
    pub eq_token: Token![=],
    pub brace_token: token::Brace,
    pub content: V,
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
