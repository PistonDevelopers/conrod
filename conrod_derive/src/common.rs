use std;

use proc_macro2;
use syn;

// The implementation for `WidgetCommon`.
pub fn impl_widget_common(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let ident = &ast.ident;
    let common_field = common_builder_field(ast).unwrap();
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let dummy_const = syn::Ident::new(&format!("_IMPL_WIDGET_COMMON_FOR_{}", ident), proc_macro2::Span::call_site());

    let impl_item = quote! {
        impl #impl_generics _conrod::widget::Common for #ident #ty_generics #where_clause {
            fn common(&self) -> &_conrod::widget::CommonBuilder {
                &self.#common_field
            }
            fn common_mut(&mut self) -> &mut _conrod::widget::CommonBuilder {
                &mut self.#common_field
            }
        }
    };

    quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            extern crate conrod_core as _conrod;
            #impl_item
        };
    }
}

// The implementation for `WidgetCommon_`.
//
// The same as `WidgetCommon` but only for use within the conrod crate itself.
pub fn impl_widget_common_(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let ident = &ast.ident;
    let common_field = common_builder_field(ast).unwrap();
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let dummy_const = syn::Ident::new(&format!("_IMPL_WIDGET_COMMON_FOR_{}", ident), proc_macro2::Span::call_site());

    let impl_item = quote! {
        impl #impl_generics ::widget::Common for #ident #ty_generics #where_clause {
            fn common(&self) -> &::widget::CommonBuilder {
                &self.#common_field
            }
            fn common_mut(&mut self) -> &mut ::widget::CommonBuilder {
                &mut self.#common_field
            }
        }
    };

    quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            #impl_item
        };
    }
}


// Find the name of the struct and the field with the `CommonBuilder` attribute.
fn common_builder_field(ast: &syn::DeriveInput) -> Result<&syn::Ident, Error> {

    // Ensure we are deriving for a struct.
    let body = match ast.data {
        syn::Data::Struct(ref body) => body,
        _ => return Err(Error::NotStruct),
    };

    // We can only derive `WidgetCommon` for structs with fields.
     match body.fields {
        syn::Fields::Named(_) => {},
        syn::Fields::Unnamed(_) => return Err(Error::TupleStruct),
        syn::Fields::Unit => return Err(Error::UnitStruct),
    };

    // Find the field on the struct with the `WidgetCommon` attribute.
    //
    // We need this to know what field to use for the `common` and `common_mut` accessor methods.
    let mut common_field = None;
    for field in body.fields.iter() {
        // First, search for the attribute.
        for attr in &field.attrs {
            if let Some(_meta) = attr.interpret_meta() {
                let mut is_conrod=false;
                let mut has_common_builder = false;
                if let syn::Meta::List(_metalist) = _meta {
                    if _metalist.ident == "conrod" {
                        is_conrod = true;
                    }

                    has_common_builder = _metalist.nested.iter().any(|v| match *v {
                        syn::NestedMeta::Meta(syn::Meta::Word(ref w))
                            if w == "common_builder" => true,
                        _ => false,
                    });
                }
                if is_conrod && has_common_builder {
                    // There should only be one `CommonBuilder` attribute.
                    if common_field.is_some() {
                        return Err(Error::MultipleCommonBuilderFields);
                    }
                    common_field = match field.ident.as_ref() {
                        Some(ident) => Some(ident),
                        None => return Err(Error::UnnamedCommonBuilderField),
                    };
                }
            }
        }
    }

    // Panic if we couldn't find a field with the `CommonBuilder` attribute.
    let common_field = match common_field {
        Some(field) => field,
        None => return Err(Error::NoCommonBuilderField),
    };

    Ok(common_field)
}


// Errors that might be returned from `name_and_common_builder_field`.
#[derive(Debug)]
enum Error {
    NotStruct,
    TupleStruct,
    UnitStruct,
    MultipleCommonBuilderFields,
    UnnamedCommonBuilderField,
    NoCommonBuilderField,
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::NotStruct =>
                "#[derive(WidgetCommon)] is only defined for structs",
            Error::TupleStruct =>
                "#[derive(WidgetCommon)] is not defined for tuple structs",
            Error::UnitStruct =>
                "#[derive(WidgetCommon)] is not defined for unit structs",
            Error::MultipleCommonBuilderFields =>
                "Found multiple #[conrod(common_builder)] attributes when only one is required",
            Error::UnnamedCommonBuilderField =>
                "Cannot use #[conrod(common_builder)] attribute on unnamed fields",
            Error::NoCommonBuilderField =>
                "`#[derive(WidgetCommon)]` requires a struct with one field of type \
                 `conrod::widget::CommonBuilder` that has the `#[conrod(common_builder)]` \
                 attribute",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", std::error::Error::description(self))
    }
}
