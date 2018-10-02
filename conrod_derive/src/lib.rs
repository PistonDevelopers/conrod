extern crate proc_macro;

extern crate proc_macro2;
#[macro_use] extern crate quote;
extern crate syn;

mod common;
mod style;
mod utils;

use proc_macro::TokenStream;

// The implementation for the `WidgetCommon` trait derivation (aka `conrod_core::widget::Common`).
#[proc_macro_derive(WidgetCommon, attributes(conrod, common_builder))]
pub fn widget_common(input: TokenStream) -> TokenStream {
    impl_derive(input, common::impl_widget_common)
}

// The implementation for the `WidgetCommon_` trait derivation (aka `conrod_core::widget::Common`).
//
// Note that this is identical to the `WidgetCommon` trait, but only for use within the conrod
// crate itself.
#[proc_macro_derive(WidgetCommon_, attributes(conrod, common_builder))]
pub fn widget_common_(input: TokenStream) -> TokenStream {
    impl_derive(input, common::impl_widget_common_)
}

// The implementation for the `WidgetStyle` trait derivation (aka `conrod_core::widget::Style`).
#[proc_macro_derive(WidgetStyle, attributes(conrod, default))]
pub fn widget_style(input: TokenStream) -> TokenStream {
    impl_derive(input, style::impl_widget_style)
}

// The implementation for the `WidgetStyle_` trait derivation (aka `conrod_core::widget::Style`).
//
// Note that this is identical to the `WidgetStyle_` trait, but only for use within the conrod
// crate itself.
#[proc_macro_derive(WidgetStyle_, attributes(conrod, default))]
pub fn widget_style_(input: TokenStream) -> TokenStream {
    impl_derive(input, style::impl_widget_style_)
}

// Use the given function to generate a TokenStream for the derive implementation.
fn impl_derive(
    input: TokenStream,
    generate_derive: fn(&syn::DeriveInput) -> proc_macro2::TokenStream,
) -> TokenStream
{
    // Parse the input TokenStream representation.
    let ast = syn::parse(input).unwrap();
    
    // Build the implementation.
    let gen = generate_derive(&ast);
    // Return the generated impl.
    gen.into()
}
