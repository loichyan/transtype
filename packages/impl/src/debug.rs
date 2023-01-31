use proc_macro2::TokenStream;
use syn::{DeriveInput, Result};
use transtype_lib::{Command, TransformOutput};

pub struct Debug;

impl Command for Debug {
    type Args = TokenStream;

    fn execute(
        data: DeriveInput,
        args: Self::Args,
        _: &mut TokenStream,
    ) -> Result<TransformOutput> {
        Ok(TransformOutput::Debug { data, args })
    }
}
