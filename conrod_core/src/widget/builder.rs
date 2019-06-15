//! The `builder_method!` macro module.

/// A macro for simplifying implementation of methods for the `builder pattern`.
///
/// See the [`builder_methods! docs](./macro.builder_methods!.html) for more background and details.
#[macro_export]
macro_rules! builder_method {

    // A public builder method that assigns the given `$Type` value to the optional `$assignee`.
    (pub $fn_name:ident { $($assignee:ident).+ = Some($Type:ty) }) => {
        /// Build the type's self.$($assignee).+ with the given $Type.
        #[inline]
        pub fn $fn_name(mut self, $fn_name: $Type) -> Self {
            self.$($assignee).+ = Some($fn_name);
            self
        }
    };

    // A builder method that assigns the given `$Type` value to the optional `$assignee`.
    ($fn_name:ident { $($assignee:ident).+ = Some($Type:ty) }) => {
        /// Build the type's self.$($assignee).+ with the given $Type.
        #[inline]
        fn $fn_name(mut self, $fn_name: $Type) -> Self {
            self.$($assignee).+ = Some($fn_name);
            self
        }
    };

    // A public builder method that assigns the given `$Type` value to the `$assignee`.
    (pub $fn_name:ident { $($assignee:ident).+ = $Type:ty }) => {
        /// Build the type's self.$($assignee).+ with the given $Type.
        #[inline]
        pub fn $fn_name(mut self, $fn_name: $Type) -> Self {
            self.$($assignee).+ = $fn_name;
            self
        }
    };

    // A builder method that assigns the given `$Type` value to the `$assignee`.
    ($fn_name:ident { $($assignee:ident).+ = $Type:ty }) => {
        /// Build the type's self.$($assignee).+ with the given $Type.
        #[inline]
        fn $fn_name(mut self, $fn_name: $Type) -> Self {
            self.$($assignee).+ = $fn_name;
            self
        }
    };

}

/// A macro to simplify implementation of
/// ["builder-pattern"](https://en.wikipedia.org/wiki/Builder_pattern) methods.
///
/// The Builder Pattern
/// ===================
///
/// Conrod (and much of the Rust ecosystem) makes extensive use of the builder pattern in order to
/// provide an expressive widget API. After much iteration, we settled upon the builder pattern as
/// the best approach to interacting with highly optional types, or in our case, widgets.
///
/// Almost all widgets implement at least a few methods in order to take advantage of this pattern.
/// We call them "builder methods".
///
/// The builder pattern looks like this:
///
/// ```
/// # extern crate conrod_core;
/// # use conrod_core::color::{Color, BLACK, LIGHT_PURPLE};
///
/// struct Button {
///     color: Option<Color>,
/// }
///
/// impl Button {
///
///     /// Construct a default Button.
///     pub fn new() -> Self {
///         Button { color: None }
///     }
///     
///     /// A Color "builder method".
///     ///
///     /// Builds the Button with the given Color.
///     pub fn color(mut self, color: Color) -> Self {
///         self.color = Some(color);
///         self
///     }
///
/// }
///
/// fn main() {
///     // Here we build a purple button.
///     let purple_button = Button::new().color(LIGHT_PURPLE);
///     assert!(button_color(&purple_button) == LIGHT_PURPLE);
///
///     // Here we build a button with some default colour (which in our case is BLACK).
///     let button = Button::new();
///     assert!(button_color(&button) == BLACK);
/// }
///
/// // A function that returns a button's color or some default if the button's color is `None`.
/// fn button_color(button: &Button) -> Color {
///     button.color.unwrap_or(BLACK)
/// }
/// ```
///
/// This allows us to support large numbers of optionally specified parameters on widgets, rather
/// than forcing a user to give them all as `Option` arguments to some function.
///
/// builder_method!
/// ================
///
/// This macro allows you to easily implement any number of builder methods for either trait or
/// direct implementations.
///
/// Here's what implementing the color method for our `Button` now looks like:
///
/// ```
/// # #[macro_use] extern crate conrod_core;
/// # use conrod_core::color::Color;
/// # struct Button { color: Option<Color> }
/// # fn main() {}
/// impl Button {
///     builder_method!(pub color { color = Some(Color) });
/// }
/// ```
///
/// Breaking it down
/// ----------------
///
/// - The first `color` is an `ident` which specifies the name of the builder function. The
/// preceding `pub` visiblity token is optional.
/// - The second `color` is the field of `self` to which we assign the given value when building.
/// - `Color` is the type which the builder method receives as an argument. The encapsulating
/// `Some(*)` is optional, and can be removed for cases where the field itself is a normal type and
/// not an `Option` type.
///
/// Multiple `builder_methods!`
/// ---------------------------
///
/// We can also use the macro to implement multiple builder methods at once. The following is an
/// example of this directly from conrod's `Tabs` widget implementation. It expands to 9 unique
/// builder methods - one for every line.
///
/// ```txt
/// builder_methods!{
///     pub bar_width { style.maybe_bar_width = Some(Scalar) }
///     pub starting_tab_idx { maybe_starting_tab_idx = Some(usize) }
///     pub label_color { style.maybe_label_color = Some(Color) }
///     pub label_font_size { style.maybe_label_font_size = Some(FontSize) }
///     pub canvas_style { style.canvas = canvas::Style }
///     pub pad_left { style.canvas.pad_left = Some(Scalar) }
///     pub pad_right { style.canvas.pad_right = Some(Scalar) }
///     pub pad_bottom { style.canvas.pad_bottom = Some(Scalar) }
///     pub pad_top { style.canvas.pad_top = Some(Scalar) }
/// }
/// ```
///
/// Note that the `builder_methods!` macro is designed to work harmony with
/// [`widget_style!`][1] - a macro which simplifies implementation of a widget's associated `Style`
/// type. If you are designing your own widget and you haven't looked at it yet, we recommend you
/// [check out the docs][1].
///
/// [1]: ./macro.widget_style!.html
#[macro_export]
macro_rules! builder_methods {
    (pub $fn_name:ident { $($assignee:ident).+ = Some($Type:ty) } $($rest:tt)*) => {
        $crate::builder_method!(pub $fn_name { $($assignee).+ = Some($Type) });
        builder_methods!($($rest)*);
    };

    ($fn_name:ident { $($assignee:ident).+ = Some($Type:ty) } $($rest:tt)*) => {
        $crate::builder_method!($fn_name { $($assignee).+ = Some($Type) });
        builder_methods!($($rest)*);
    };
    
    (pub $fn_name:ident { $($assignee:ident).+ = $Type:ty } $($rest:tt)*) => {
        $crate::builder_method!(pub $fn_name { $($assignee).+ = $Type });
        builder_methods!($($rest)*);
    };

    ($fn_name:ident { $($assignee:ident).+ = $Type:ty } $($rest:tt)*) => {
        $crate::builder_method!($fn_name { $($assignee).+ = $Type });
        builder_methods!($($rest)*);
    };

    ($($rest:tt)*) => {};
}
