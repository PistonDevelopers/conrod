/// Defines a struct called `$Style`.
///
/// Each given `$field_name` `$FieldType` pair will be defined as `Option` fields.
#[doc(hidden)]
#[macro_export]
macro_rules! define_widget_style_struct {

    (
        $(#[$Style_attr:meta])*
        style $Style:ident {
            $( $(#[$field_attr:meta])* - $field_name:ident: $FieldType:ty { $default:expr })*
        }
    ) => {
        $(#[$Style_attr])*
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $Style {
            $(
                $(#[$field_attr])*
                pub $field_name: Option<$FieldType>,
            )*
        }
    };

    (
        $(#[$Style_attr:meta])*
        style $Style:ident {
            $(
                $(#[$field_attr:meta])*
                - $field_name:ident: $FieldType:ty { theme.$($theme_field:ident).+ }
             )*
        }
    ) => {
        define_widget_style_struct!{
            $(#[$Style_attr])*
            style $Style {
                $( $(#[$field_attr])* - $field_name: $FieldType {})*
            }
        }
    };

}


/// Produces a default instance of `$Style` (aka all fields set to `None`).
#[doc(hidden)]
#[macro_export]
macro_rules! default_widget_style_struct {

    (
        $Style:ident {
            $(
                $(#[$field_attr:meta])*
                - $field_name:ident: $FieldType:ty { $default:expr }
             )*
        }
    ) => {
        $Style {
            $(
                $field_name: None,
            )*
        }
    };

    (
        $Style:ident {
            $(
                $(#[$field_attr:meta])*
                - $field_name:ident: $FieldType:ty { theme.$($theme_field:ident).+ }
             )*
        }
    ) => {
        default_widget_style_struct!{
            $Style {
                $(
                    $(#[$field_attr])*
                    - $field_name: $FieldType {}
                )*
            }
        };
    };

}

/// Implements the static method `new` for the `$Style` type.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_widget_style_new {
    ($Style:ident { $($fields:tt)* }) => {
        impl $Style {
            /// Construct the default `Style`, initialising all fields to `None`.
            pub fn new() -> Self {
                default_widget_style_struct!($Style { $($fields)* })
            }
        }
    };
}


#[doc(hidden)]
#[macro_export]
macro_rules! impl_widget_style_retrieval_method {
    ($field_name:ident: $FieldType:ty { theme.$($theme_field:ident).+ }) => {
        /// Retrieves the value from the `Style`.
        ///
        /// If the `Style`'s field is `None`, falls back to default specified within the `Theme`.
        pub fn $field_name(&self, theme: &$crate::Theme) -> $FieldType {
            self.$field_name
                .or_else(|| theme.widget_style::<Self>().and_then(|default| {
                    default.style.$field_name
                }))
                .unwrap_or_else(|| theme.$($theme_field).+)
        }
    };
    ($field_name:ident: $FieldType:ty { $default:expr }) => {
        /// Retrieves the value from the `Style`.
        ///
        /// If the `Style`'s field is `None`, falls back to default specified within the `Theme`.
        pub fn $field_name(&self, theme: &$crate::Theme) -> $FieldType {
            self.$field_name
                .or_else(|| theme.widget_style::<Self>().and_then(|default| {
                    default.style.$field_name
                }))
                .unwrap_or_else(|| $default)
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_widget_style_retrieval_methods {

    // Retrieval methods with a field on the `theme` to use as the default.
    (
        $(#[$field_attr:meta])*
        - $field_name:ident: $FieldType:ty { theme.$($theme_field:ident).+ } $($rest:tt)*
    ) => {
        impl_widget_style_retrieval_method!($field_name: $FieldType { theme.$($theme_field).+});
        impl_widget_style_retrieval_methods!($($rest)*);
    };


    // Retrieval methods with an expression to use as the default.
    (
        $(#[$field_attr:meta])*
        - $field_name:ident: $FieldType:ty { $default:expr } $($rest:tt)*
    ) => {
        impl_widget_style_retrieval_method!($field_name: $FieldType { $default });
        impl_widget_style_retrieval_methods!($($rest)*);
    };

    // All methods have been implemented.
    () => {};
}




/// A macro for vastly simplifying the definition and implementation of a widget's associated
/// `Style` type.
///
/// For more information on the purpose of this `Style` type, see the associated `type Style` docs
/// in the [`Widget` trait documentation](./widget/trait.Widget.html).
///
/// Using the macro looks like this:
///
/// ```
/// # #[macro_use] extern crate conrod;
/// # fn main() {}
/// widget_style!{
///     // Doc comment or attr for the generated `Style` struct can go here.
///     style Style {
///         // Fields and their type T (which get converted to `Option<T>` in the struct definition)
///         // along with their default expression go here.
///         // You can also write doc comments or attr above each field.
///         - color: conrod::Color { theme.shape_color }
///         - label_color: conrod::Color { conrod::color::BLUE }
///         // .. more fields.
///     }
/// }
/// ```
///
/// An invocation of the macro expands into two things:
///
/// 1. A struct definition with the given name following the `style` token.
/// 2. An `impl Style` block with a `new` constructor as well as a style retrieval method for each
///    given field. These retrieval methods do the following:
///
///    1. Attempt to use the value at the field.
///    2. If the field is `None`, attempts to retreive a default from the `widget_styling` map in
///       the `Ui`'s current `Theme`.
///    3. If no defaults were found, evaluates the given default expression (or `theme.field`).
///
///
/// # Examples
///
/// The following is a typical usage example for the `widget_style!` macro.
///
/// ```
/// #[macro_use]
/// extern crate conrod;
/// 
/// struct MyWidget {
///     style: Style,
///     // Other necessary fields...
/// }
///
/// widget_style!{
///     /// Unique, awesome styling for a `MyWidget`.
///     style Style {
///         /// The totally amazing color to use for the `MyWidget`.
///         ///
///         /// If the `color` is unspecified and there is no default given via the `Theme`, the
///         /// `Theme`'s standard `shape_color` field will be used as a fallback.
///         - color: conrod::Color { theme.shape_color }
///         /// The extremely pretty color to use for the `MyWidget`'s label.
///         ///
///         /// If the `label_color` is unspecified and there is no default given via the `Theme`,
///         /// the label will fallback to `conrod::color::PURPLE`.
///         - label_color: conrod::Color { conrod::color::PURPLE }
///     }
/// }
///
/// // We can now retrieve the `color` or `label_color` for a `MyWidget` like so:
/// // let color = my_widget.style.color(&theme);
/// // let label_color = my_widget.style.label_color(&theme);
/// 
/// # fn main() {}
/// ```
///
/// And here is what it expands into:
///
/// ```
/// #[macro_use]
/// extern crate conrod;
///
/// struct MyWidget {
///     style: Style,
///     // Other necessary fields...
/// }
///
/// /// Unique, awesome styling for a `MyWidget`.
/// #[derive(Copy, Clone, Debug, PartialEq)]
/// pub struct Style {
///     /// The totally amazing color to use for the `MyWidget`.
///     ///
///     /// If the `color` is unspecified and there is no default given via the `Theme`, the
///     /// `Theme`'s standard `shape_color` field will be used as a fallback.
///     color: Option<conrod::Color>,
///     /// The extremely pretty color to use for the `MyWidget`'s label.
///     ///
///     /// If the `label_color` is unspecified and there is no default given via the `Theme`,
///     /// the label will fallback to `conrod::color::PURPLE`.
///     label_color: Option<conrod::Color>,
/// }
///
/// impl Style {
///
///     /// Construct the default `Style`, initialising all fields to `None`.
///     pub fn new() -> Self {
///         Style {
///             color: None,
///             label_color: None,
///         }
///     }
/// 
///     /// Retrieves the value from the `Style`.
///     ///
///     /// If the `Style`'s field is `None`, falls back to default specified within the `Theme`.
///     pub fn color(&self, theme: &conrod::Theme) -> conrod::Color {
///         self.color
///             .or_else(|| theme.widget_style::<Self>().and_then(|default| {
///                 default.style.color
///             }))
///             .unwrap_or_else(|| theme.shape_color)
///     }
///
///     /// Retrieves the value from the `Style`.
///     ///
///     /// If the `Style`'s field is `None`, falls back to default specified within the `Theme`.
///     pub fn label_color(&self, theme: &conrod::Theme) -> conrod::Color {
///         self.label_color
///             .or_else(|| theme.widget_style::<Self>().and_then(|default| {
///                 default.style.label_color
///             }))
///             .unwrap_or_else(|| conrod::color::PURPLE)
///     }
///
/// }
///
/// // We can now retrieve the `color` or `label_color` for a `MyWidget` like so:
/// // let color = my_widget.style.color(&theme);
/// // let label_color = my_widget.style.label_color(&theme);
///
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! widget_style {
    (
        $(#[$Style_attr:meta])*
        style $Style:ident {
            $($fields:tt)*
        }
    ) => {

        // The `Style` struct.
        define_widget_style_struct!{
            $(#[$Style_attr])*
            style $Style {
                $($fields)*
            }
        }

        // The `new` method for the `Style` struct.
        impl_widget_style_new!($Style { $($fields)* });

        // The "field, theme or default" retrieval methods.
        impl $Style {
            impl_widget_style_retrieval_methods!($($fields)*);
        }
    };
}
