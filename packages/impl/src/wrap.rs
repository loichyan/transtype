use crate::ast::Delimiter;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Data, DeriveInput, Ident, Member, Result, Type};
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

        Ok(TransformOutput::Piped { data })
    }
}

pub struct Wrapped;

impl Command for Wrapped {
    type Args = Type;

    fn execute(
        data: DeriveInput,
        from: Self::Args,
        _: &mut TokenStream,
    ) -> Result<TransformOutput> {
        let mut body = TokenStream::default();
        match &data.data {
            Data::Struct(data) => {
                Delimiter::from_feilds(&data.fields).surround(&mut body, |tokens| {
                    data.fields.iter().enumerate().for_each(|(i, field)| {
                        tokens.extend(
                            field
                                .ident
                                .as_ref()
                                .map(
                                    |name| quote!(#name: ::transtype::Wrapper::unwrap(self.#name),),
                                )
                                .unwrap_or_else(|| {
                                    let i = Member::Unnamed(i.into());
                                    quote!(::transtype::Wrapper::unwrap(self.#i),)
                                }),
                        )
                    });
                });
            }
            _ => unreachable!(),
        }
        let name = &data.ident;
        Ok(TransformOutput::Consumed {
            data: quote!(
                impl ::transtype::Wrapped for #name {
                    type Original = #from;

                    fn unwrap(self) -> Self::Original {
                        #from #body
                    }
                }
            ),
        })
    }
}
