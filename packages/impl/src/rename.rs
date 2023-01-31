use proc_macro2::TokenStream;
use syn::{Ident, Result};
use transtype_lib::{Command, TransformOutput};

pub struct Rename;

impl Command for Rename {
    type Args = Ident;

    fn execute(
        mut data: syn::DeriveInput,
        name: Self::Args,
        _: &mut TokenStream,
    ) -> Result<TransformOutput> {
        data.ident = name;
        Ok(TransformOutput::Pipe { data })
    }
}
