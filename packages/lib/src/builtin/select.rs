use super::ast::{DeriveInputExt, PathExt, Selectors};
use crate::{TransformRest, TransformState, Transformer};
use syn::{punctuated::Punctuated, Attribute, DeriveInput, Field, Result, Token};

pub(crate) struct Select;

impl Transformer for Select {
    type Args = Selectors;

    fn transform(
        mut data: DeriveInput,
        args: Self::Args,
        _: &mut TransformRest,
    ) -> Result<TransformState> {
        data.fields_iter()
            .for_each(|fields| args.select_fields(fields));
        Ok(TransformState::pipe(data))
    }
}

pub struct SelectAttr;

impl Transformer for SelectAttr {
    type Args = Selectors;

    fn transform(
        mut data: DeriveInput,
        args: Self::Args,
        _: &mut TransformRest,
    ) -> Result<TransformState> {
        args.select_attrs(&mut data.attrs);
        data.fields_iter()
            .flat_map(|fields| fields.iter_mut())
            .for_each(|field| args.select_attrs(&mut field.attrs));
        Ok(TransformState::pipe(data))
    }
}

impl Selectors {
    fn select_fields(&self, fields: &mut Punctuated<Field, Token![,]>) {
        *fields = std::mem::take(fields)
            .into_iter()
            .filter_map(|mut field| {
                if let Some(name) = &mut field.ident {
                    if let Some(rename) = self.select(name) {
                        *name = rename;
                    } else {
                        return None;
                    }
                }
                Some(field)
            })
            .collect();
    }

    fn select_attrs(&self, attrs: &mut Vec<Attribute>) {
        attrs.retain_mut(|attr| {
            if let Some(name) = attr.path.get_ident_mut() {
                if let Some(rename) = self.select(name) {
                    *name = rename;
                } else {
                    return false;
                }
            }
            true
        });
    }
}
