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

pub trait Command: Sized {
    type Args: Parse;

    fn execute(data: DeriveInput, args: Self::Args) -> Result<CommandOutput>;
}

pub struct CommandInput<T: Command> {
    data: NamedArg<kw::data, DeriveInput>,
    args: NamedArg<kw::args, T::Args>,
    rest: NamedArg<kw::rest, TokenStream>,
}

impl<T: Command> CommandInput<T> {
    pub fn expand(self) -> Result<TokenStream> {
        let data = self.data.content;
        let args = self.args.content;
        let rest = self.rest.content;
        match T::execute(data, args)? {
            CommandOutput::Piped(data) => Ok(quote!(::transtype::transform! {
               data={#data}
               args={}
               rest={#rest}
            })),
            CommandOutput::Consumed(tokens) => Ok(tokens),
            CommandOutput::Transformed { path, data, args } => Ok(quote!(
                #path! {
                    data={#data}
                    args={#args}
                    rest={#rest}
                }
            )),
        }
    }
}

pub enum CommandOutput {
    Piped(DeriveInput),
    Consumed(TokenStream),
    Transformed {
        path: Path,
        data: Option<DeriveInput>,
        args: TokenStream,
    },
}

impl<T: Command> Parse for CommandInput<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            data: input.parse()?,
            args: input.parse()?,
            rest: input.parse()?,
        })
    }
}

pub struct NamedArg<K, V> {
    pub name: K,
    pub eq_tk: Token![=],
    pub brace_tk: token::Brace,
    pub content: V,
}

impl<K, V> NamedArg<K, V>
where
    K: Parse,
{
    pub fn swap_content<V2>(self, content: V2) -> (NamedArg<K, V2>, V) {
        let NamedArg {
            name: key,
            eq_tk,
            brace_tk,
            content: old,
        } = self;
        (
            NamedArg {
                name: key,
                eq_tk,
                brace_tk,
                content,
            },
            old,
        )
    }

    pub fn parse_with(
        input: ParseStream,
        f: impl FnOnce(ParseStream) -> Result<V>,
    ) -> Result<Self> {
        let content;
        Ok(Self {
            name: input.parse()?,
            eq_tk: input.parse()?,
            brace_tk: braced!(content in input),
            content: f(&content)?,
        })
    }
}

impl<K, V> Parse for NamedArg<K, V>
where
    K: Parse,
    V: Parse,
{
    fn parse(input: ParseStream) -> Result<Self> {
        Self::parse_with(input, V::parse)
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
