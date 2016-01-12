


/// Defines a struct called `$Style`.
///
/// Each given `$field_name` `$FieldType` pair will be defined as `Option` fields.
macro_rules! define_widget_style_struct {

    (
        $(#[$Style_attr:meta])*
        style $Style:ident {
            $( $(#[$field_attr:meta])* - $field_name:ident: $FieldType:ty { $default:expr }),* $(,)*
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
             ),* $(,)*
        }
    ) => {
        define_widget_style_struct!{
            $(#[$Style_attr])*
            style $Style {
                $( $(#[$field_attr])* - $field_name: $FieldType {},)*
            }
        }
    };

}


/// Produces a default instance of `$Style` (aka all fields set to `None`).
macro_rules! default_widget_style_struct {

    (
        $Style:ident {
            $(
                $(#[$field_attr:meta])*
                - $field_name:ident: $FieldType:ty { $default:expr }
             ),* $(,)*
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
             ),* $(,)*
        }
    ) => {
        default_widget_style_struct!{
            $Style {
                $(
                    $(#[$field_attr])*
                    - $field_name: $FieldType {},
                )*
            }
        };
    };

}

/// Implements the static method `new` for the `$Style` type.
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


macro_rules! impl_widget_style_retrieval_method {
    ($KIND:ident, $field_name:ident: $FieldType:ty { theme.$($theme_field:ident).+ }) => {
        /// Retrieves the value from the `Style`.
        ///
        /// If the `Style`'s field is `None`, falls back to default specified within the `Theme`.
        pub fn $field_name(&self, theme: &$crate::Theme) -> $FieldType {
            self.$field_name
                .or_else(|| theme.widget_style::<Self>($KIND).and_then(|default| {
                    default.style.$field_name
                }))
                .unwrap_or_else(|| theme.$($theme_field).+)
        }
    };
    ($KIND:ident, $field_name:ident: $FieldType:ty { $default:expr }) => {
        /// Retrieves the value from the `Style`.
        ///
        /// If the `Style`'s field is `None`, falls back to default specified within the `Theme`.
        pub fn $field_name(&self, theme: &$crate::Theme) -> $FieldType {
            self.$field_name
                .or_else(|| theme.widget_style::<Self>($KIND).and_then(|default| {
                    default.style.$field_name
                }))
                .unwrap_or_else(|| $default)
        }
    };
}

macro_rules! impl_widget_style_retrieval_methods {

    // Retrieval methods with a field on the `theme` to use as the default.
    (
        $KIND:ident,
        $(#[$field_attr:meta])*
        - $field_name:ident: $FieldType:ty { theme.$($theme_field:ident).+ }, $($rest:tt)*
    ) => {
        impl_widget_style_retrieval_method!($KIND, $field_name: $FieldType { theme.$($theme_field).+});
        impl_widget_style_retrieval_methods!($KIND, $($rest)*);
    };

    // Retrieval methods with an expression to use as the default.
    (
        $KIND:ident,
        $(#[$field_attr:meta])*
        - $field_name:ident: $FieldType:ty { $default:expr }, $($rest:tt)*
    ) => {
        impl_widget_style_retrieval_method!($KIND, $field_name: $FieldType { $default });
        impl_widget_style_retrieval_methods!($KIND, $($rest)*);
    };

    // The last retrieval method with no trailing comma.
    (
        $KIND:ident,
        $(#[$field_attr:meta])*
        - $field_name:ident: $FieldType:ty { theme.$($theme_field:ident).+ }
    ) => {
        impl_widget_style_retrieval_method!($KIND, $field_name: $FieldType { theme.$($theme_field).+});
    };

    // The last retrieval method with no trailing comma.
    (
        $KIND:ident,
        $(#[$field_attr:meta])*
        - $field_name:ident: $FieldType:ty { $default:expr }
    ) => {
        impl_widget_style_retrieval_method!($KIND, $field_name: $FieldType { $default });
    };

    // All methods have been implemented.
    ($KIND:ident,) => {};
}



/// A macro for removing the boilerplate present in widget `Style` type implementations.
#[macro_export]
macro_rules! widget_style {
    (
        $KIND:ident;
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
            impl_widget_style_retrieval_methods!($KIND, $($fields)*);
        }
    };
}
