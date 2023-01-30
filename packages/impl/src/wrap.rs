use crate::{ast::Delimiter, kw};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, Data, DeriveInput, Ident, Member, Result, Type,
};
use transtype_lib::{Command, TransformOutput};

pub struct Wrap;

impl Command for Wrap {
    type Args = WrapArgs;

    fn execute(
        mut data: DeriveInput,
        args: Self::Args,
        _: &mut TokenStream,
    ) -> Result<TransformOutput> {
        let WrapArgs {
            ident: name,
            ty: from_ty,
            ..
        } = args;
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

        let definition = quote!(
            const _: () = {
                ::transtype::private::requires_wrapper::<
                    #name::<::transtype::private::InnerType>,
                >();
            };
            #data
        );

        let name = &data.ident;
        let impl_block = from_ty.map(|from_ty| {
            let mut body = TokenStream::default();
            match &data.data {
                Data::Struct(data) => {
                    Delimiter::from_feilds(&data.fields).surround(&mut body, |tokens| {
                        data.fields.iter().enumerate().for_each(|(i, field)| {
                            tokens.extend(
                                field
                                    .ident
                                    .as_ref()
                                    .map(|name| quote!(#name: ::transtype::Wrapper::unwrap(self.#name),))
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
            quote!(
                impl ::transtype::Wrapped for #name {
                    type Original = #from_ty;

                    fn unwrap(self) -> Self::Original {
                        #from_ty #body
                    }
                }
            )
        });

        Ok(TransformOutput::Consumed {
            data: quote!(#definition #impl_block),
        })
    }
}

pub struct WrapArgs {
    pub ident: Ident,
    pub from: Option<kw::from>,
    pub ty: Option<Type>,
}

impl Parse for WrapArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;
        let from;
        let ty;
        if input.peek(kw::from) {
            from = Some(input.parse()?);
            ty = Some(input.parse()?);
        } else {
            from = None;
            ty = None;
        }
        Ok(Self { ident, from, ty })
    }
}
