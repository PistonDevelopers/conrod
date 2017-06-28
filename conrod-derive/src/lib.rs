extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;

use proc_macro::TokenStream;

// The implementation for the `WidgetCommon` trait derivation (aka `conrod::widget::Common`).
#[proc_macro_derive(WidgetCommon, attributes(conrod, common_builder))]
pub fn widget_common(input: TokenStream) -> TokenStream {

    // A string representatin of the type definition.
    let input_string = input.to_string();

    // Parse the string representation.
    let ast = syn::parse_derive_input(&input_string).unwrap();

    // Build the implementation.
    let gen = impl_widget_common(&ast);

    // Return the generated impl.
    gen.parse().unwrap()
}

// The implementation for the `WidgetCommon_` trait derivation (aka `conrod::widget::Common`).
//
// Note that this is identical t the `WidgetCommon` trait, but only for use within the conrod
// crate itself.
#[proc_macro_derive(WidgetCommon_, attributes(conrod, common_builder))]
pub fn widget_common_(input: TokenStream) -> TokenStream {

    // A string representatin of the type definition.
    let input_string = input.to_string();

    // Parse the string representation.
    let ast = syn::parse_derive_input(&input_string).unwrap();

    // Build the implementation.
    let gen = impl_widget_common_(&ast);

    // Return the generated impl.
    gen.parse().unwrap()
}

// The implementation for `WidgetCommon`.
fn impl_widget_common(ast: &syn::DeriveInput) -> quote::Tokens {
    let ident = &ast.ident;
    let common_field = common_builder_field(ast).unwrap();
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let dummy_const = syn::Ident::new(format!("_IMPL_WIDGET_COMMON_FOR_{}", ident));

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
        extern crate conrod as _conrod;
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            #impl_item
        };
    }
}

// The implementation for `WidgetCommon_`. The same as `WidgetCommon` but only for use within the
// conrod crate itself.
fn impl_widget_common_(ast: &syn::DeriveInput) -> quote::Tokens {
    let ident = &ast.ident;
    let common_field = common_builder_field(ast).unwrap();
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let dummy_const = syn::Ident::new(format!("_IMPL_WIDGET_COMMON_FOR_{}", ident));

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
    let body = match ast.body {
        syn::Body::Struct(ref body) => body,
        _ => return Err(Error::NotStruct),
    };

    // We can only derive `WidgetCommon` for structs with fields.
    let fields = match *body {
        syn::VariantData::Struct(ref fields) => fields,
        syn::VariantData::Tuple(_) => return Err(Error::TupleStruct),
        syn::VariantData::Unit => return Err(Error::UnitStruct),
    };

    // Find the field on the struct with the `WidgetCommon` attribute.
    //
    // We need this to know what field to use for the `common` and `common_mut` accessor methods.
    let mut common_field = None;
    for field in fields {
        // First, search for the attribute.
        for attr in &field.attrs {
            if let syn::MetaItem::List(ref ident, ref values) = attr.value {
                let has_common_builder = values.iter().any(|v| match *v {
                    syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref w))
                        if w == "common_builder" => true,
                    _ => false,
                });
                if ident == "conrod" && has_common_builder {
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
                "#[derive(WidgetCommon)]` is only defined for structs",
            Error::TupleStruct =>
                "#[derive(WidgetCommon)]` is not defined for tuple structs",
            Error::UnitStruct =>
                "#[derive(WidgetCommon)]` is not defined for unit structs",
            Error::MultipleCommonBuilderFields =>
                "Found multiple `#[CommonBuilder]` attributes when only one is required",
            Error::UnnamedCommonBuilderField =>
                "Cannot use #[CommonBuilder] attribute on unnamed fields",
            Error::NoCommonBuilderField =>
                "`#[derive(WidgetCommon)]` requires a struct with one field of type \
                 `conrod::widget::CommonBuilder` that has the `#[CommonBuilder]` attribute",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", std::error::Error::description(self))
    }
}
