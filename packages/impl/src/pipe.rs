use crate::{define::QuoteExecution, utils::Delimiter};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    token, DeriveInput, Path, Result, Token,
};
use transtype_lib::{Command, CommandOutput};

pub fn expand(input: PipeInput) -> Result<TokenStream> {
    let PipeInput { path, rest } = input;
    Ok(QuoteExecution {
        path: &path,
        data: None,
        args: None,
        rest: Some(&rest),
    }
    .into_token_stream())
}

pub struct PipeInput {
    path: Path,
    rest: TokenStream,
}

impl Parse for PipeInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            path: input.parse()?,
            rest: input.parse()?,
        })
    }
}

pub struct PipeCommand<T = TokenStream> {
    pub fat_arrow_tk: Token![=>],
    pub path: Path,
    pub delimiter: Delimiter,
    pub args: T,
}

impl PipeCommand<TokenStream> {
    pub fn parse_into<T: Parse>(self) -> Result<PipeCommand<T>> {
        let Self {
            fat_arrow_tk,
            path,
            delimiter,
            args: content,
        } = self;
        Ok(PipeCommand {
            fat_arrow_tk,
            path,
            delimiter,
            args: syn::parse2(content)?,
        })
    }

    pub fn execute_as<T: Command>(self, data: DeriveInput) -> Result<CommandOutput> {
        let cmd = self.parse_into::<T::Args>()?;
        T::execute(data, cmd.args).map_err(|mut e| {
            e.combine(syn::Error::new_spanned(
                &cmd.path,
                "an error occurs in this command",
            ));
            e
        })
    }
}

impl<T: Parse> Parse for PipeCommand<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            fat_arrow_tk: input.parse()?,
            path: input.parse()?,
            delimiter: delimited!(content in input),
            args: content.parse()?,
        })
    }
}
