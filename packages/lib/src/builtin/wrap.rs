use super::ast::Delimiter;
use crate::{TransformRest, TransformState, Transformer};
use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{parse_quote, Data, DeriveInput, Ident, Member, Result, Type};

pub(crate) struct Wrap;

impl Transformer for Wrap {
    type Args = Ident;

    fn transform(
        mut data: DeriveInput,
        name: Self::Args,
        _: &mut TransformRest,
    ) -> Result<TransformState> {
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

        Ok(TransformState::pipe(data))
    }
}

pub(crate) struct Wrapped;

impl Transformer for Wrapped {
    type Args = Type;

    fn transform(
        data: DeriveInput,
        from: Self::Args,
        rest: &mut TransformRest,
    ) -> Result<TransformState> {
        let span = rest.span();
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
                                    |name| quote_spanned!(span=> #name: ::transtype::Wrapper::unwrap(self.#name),),
                                )
                                .unwrap_or_else(|| {
                                    let i = Member::Unnamed(i.into());
                                    quote_spanned!(span=> ::transtype::Wrapper::unwrap(self.#i),)
                                }),
                        )
                    });
                });
            }
            _ => unreachable!(),
        }
        let name = &data.ident;
        Ok(TransformState::Pipe {
            pipe: None,
            plus: Some(quote_spanned!(span=>
                impl ::transtype::Wrapped for #name {
                    type Original = #from;

                    fn unwrap(self) -> Self::Original {
                        #from #body
                    }
                }
            )),
            data,
        })
    }
}
