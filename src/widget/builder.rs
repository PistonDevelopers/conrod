//! The `builder_method!` macro module.

/// A macro for simplifying implementation of methods for the `builder pattern`.
#[macro_export]
macro_rules! builder_method {

    // A public builder method that assigns the given `$Type` value to the optional `$assignee`.
    (pub $fn_name:ident { $($assignee:ident).+ = Some($Type:ty) }) => {
        /// Build the type's self.$($assignee).+ with the given $Type.
        pub fn $fn_name(mut self, $fn_name: $Type) -> Self {
            self.$($assignee).+ = Some($fn_name);
            self
        }
    };

    // A builder method that assigns the given `$Type` value to the optional `$assignee`.
    ($fn_name:ident { $($assignee:ident).+ = Some($Type:ty) }) => {
        /// Build the type's self.$($assignee).+ with the given $Type.
        fn $fn_name(mut self, $fn_name: $Type) -> Self {
            self.$($assignee).+ = Some($fn_name);
            self
        }
    };

    // A public builder method that assigns the given `$Type` value to the `$assignee`.
    (pub $fn_name:ident { $($assignee:ident).+ = $Type:ty }) => {
        /// Build the type's self.$($assignee).+ with the given $Type.
        pub fn $fn_name(mut self, $fn_name: $Type) -> Self {
            self.$($assignee).+ = $fn_name;
            self
        }
    };

    // A builder method that assigns the given `$Type` value to the `$assignee`.
    ($fn_name:ident { $($assignee:ident).+ = $Type:ty }) => {
        /// Build the type's self.$($assignee).+ with the given $Type.
        fn $fn_name(mut self, $fn_name: $Type) -> Self {
            self.$($assignee).+ = $fn_name;
            self
        }
    };

}


macro_rules! builder_methods {
    (pub $fn_name:ident { $($assignee:ident).+ = Some($Type:ty) } $($rest:tt)*) => {
        builder_method!(pub $fn_name { $($assignee).+ = Some($Type) });
        builder_methods!($($rest)*);
    };

    ($fn_name:ident { $($assignee:ident).+ = Some($Type:ty) } $($rest:tt)*) => {
        builder_method!($fn_name { $($assignee).+ = Some($Type) });
        builder_methods!($($rest)*);
    };
    
    (pub $fn_name:ident { $($assignee:ident).+ = $Type:ty } $($rest:tt)*) => {
        builder_method!(pub $fn_name { $($assignee).+ = $Type });
        builder_methods!($($rest)*);
    };

    ($fn_name:ident { $($assignee:ident).+ = $Type:ty } $($rest:tt)*) => {
        builder_method!($fn_name { $($assignee).+ = $Type });
        builder_methods!($($rest)*);
    };

    ($($rest:tt)*) => {};
}
