use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Data, DeriveInput, Ident, Result, Type};
use transtype_lib::{Command, TransformOutput};

pub struct Wrap;

impl Command for Wrap {
    type Args = Ident;

    fn execute(
        mut data: DeriveInput,
        name: Self::Args,
        _: &mut TokenStream,
    ) -> Result<TransformOutput> {
        match &mut data.data {
            Data::Struct(data) => {
                for field in data.fields.iter_mut() {
                    if let Type::Path(ty) = &field.ty {
                        if ty.path.leading_colon.is_none()
                            && ty.path.segments.first().map(|t| &t.ident) == Some(&name)
                        {
                        } else {
                            let inner = &field.ty;
                            field.ty = parse_quote!(#name::<#inner>);
                        }
                    }
                }
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    &data,
                    "only struct is supported now",
                ))
            }
        }

        // TODO: impl `Wrapper`
        Ok(TransformOutput::Consumed {
            data: quote!(
                const _: () = {
                    ::transtype::private::requires_wrapper::<
                        #name::<::transtype::private::InnerType>,
                    >();
                };
                #data
            ),
        })
    }
}
