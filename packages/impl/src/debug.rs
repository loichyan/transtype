use crate::ast::Nothing;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Result;
use transtype_lib::{Command, TransformOutput};

pub struct Debug;

impl Command for Debug {
    type Args = Nothing;

    fn execute(
        data: syn::DeriveInput,
        _: Self::Args,
        rest: &mut TokenStream,
    ) -> Result<TransformOutput> {
        let rest = std::mem::take(rest);
        let name = &data.ident;
        let name = format_ident!("__debug_{name}", span = name.span());
        Ok(TransformOutput::Consumed {
            data: quote!(
                macro_rules! #name {
                    () => {
                        data={#data}
                        rest={#rest}
                    };
                }
            ),
        })
    }
}
