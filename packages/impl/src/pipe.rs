use crate::ast::Nothing;
use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, Path, Result,
};
use transtype_lib::{TransformInput, TransformOutput, Transformer};

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let input: TransformInput<Pipe> = parse_quote!(
        data={}
        args={#input}
        rest={}
    );
    input.transform()
}

struct Pipe;

impl Transformer for Pipe {
    type Data = Nothing;
    type Args = PipeArgs;

    fn transform(_: Nothing, args: Self::Args, _: &mut TokenStream) -> Result<TransformOutput> {
        let PipeArgs { path, pipes } = args;
        Ok(TransformOutput::Transferred {
            path,
            data: None,
            args: pipes,
        })
    }
}

struct PipeArgs {
    path: Path,
    pipes: TokenStream,
}

impl Parse for PipeArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            path: input.parse()?,
            pipes: input.parse()?,
        })
    }
}
