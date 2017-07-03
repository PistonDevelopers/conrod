extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;

mod common;
mod style;
mod utils;

use proc_macro::TokenStream;

// The implementation for the `WidgetCommon` trait derivation (aka `conrod::widget::Common`).
#[proc_macro_derive(WidgetCommon, attributes(conrod, common_builder))]
pub fn widget_common(input: TokenStream) -> TokenStream {
    impl_derive(input, common::impl_widget_common)
}

// The implementation for the `WidgetCommon_` trait derivation (aka `conrod::widget::Common`).
//
// Note that this is identical to the `WidgetCommon` trait, but only for use within the conrod
// crate itself.
#[proc_macro_derive(WidgetCommon_, attributes(conrod, common_builder))]
pub fn widget_common_(input: TokenStream) -> TokenStream {
    impl_derive(input, common::impl_widget_common_)
}

// The implementation for the `WidgetStyle` trait derivation (aka `conrod::widget::Style`).
#[proc_macro_derive(WidgetStyle, attributes(conrod, default))]
pub fn widget_style(input: TokenStream) -> TokenStream {
    impl_derive(input, style::impl_widget_style)
}

// The implementation for the `WidgetStyle_` trait derivation (aka `conrod::widget::Style`).
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
    generate_derive: fn(&syn::DeriveInput) -> quote::Tokens,
) -> TokenStream
{
    // A string representatin of the type definition.
    let input_string = input.to_string();

    // Parse the string representation.
    let ast = syn::parse_derive_input(&input_string).unwrap();

    // Build the implementation.
    let gen = generate_derive(&ast);

    // Return the generated impl.
    gen.parse().unwrap()
}
