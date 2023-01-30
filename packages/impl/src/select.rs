use proc_macro2::TokenStream;
use syn::{punctuated::Punctuated, Attribute, DeriveInput, Field, Result, Token};
use transtype_lib::{Command, TransformOutput};

use crate::ast::{DeriveInputExt, PathExt, Selectors};

pub struct Select;

impl Command for Select {
    type Args = Selectors;

    fn execute(
        mut data: DeriveInput,
        args: Self::Args,
        _: &mut TokenStream,
    ) -> Result<TransformOutput> {
        data.fields_iter()
            .for_each(|fields| args.select_fields(fields));
        Ok(TransformOutput::Piped { data })
    }
}

pub struct SelectAttr;

impl Command for SelectAttr {
    type Args = Selectors;

    fn execute(
        mut data: DeriveInput,
        args: Self::Args,
        _: &mut TokenStream,
    ) -> Result<TransformOutput> {
        args.select_attrs(&mut data.attrs);
        data.fields_iter()
            .flat_map(|fields| fields.iter_mut())
            .for_each(|field| args.select_attrs(&mut field.attrs));
        Ok(TransformOutput::Piped { data })
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
