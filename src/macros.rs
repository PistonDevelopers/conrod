
#![macro_escape]

/// Simplify implementation of BoilerPlate widget module functions.
macro_rules! widget_fns(
    ($widget:ident, $widget_state:ident, $default:expr) => (

        /// Default Widget variant.
        fn default() -> ::widget::Widget { $default }

        /// Get a reference to the widget associated with the given UIID.
        fn get_widget(uic: &mut ::ui_context::UIContext,
                      ui_id: ::ui_context::UIID) -> &mut ::widget::Widget {
            uic.get_widget(ui_id, default())
        }

        /// Get the current State for the widget.
        fn get_state(uic: &mut ::ui_context::UIContext,
                     ui_id: ::ui_context::UIID) -> &$widget_state {
            match *get_widget(uic, ui_id) {
                ::widget::$widget(ref state) => state,
                _ => fail!("The Widget variant returned by UIContext is different to the requested."),
            }
        }

        /// Set the state for the widget in the UIContext.
        fn set_state(uic: &mut ::ui_context::UIContext,
                     ui_id: ::ui_context::UIID,
                     new_state: $widget_state) {
            match *get_widget(uic, ui_id) {
                ::widget::$widget(ref mut state) => { *state = new_state; },
                _ => fail!("The Widget variant returned by UIContext is different to the requested."),
            }
        }

    )
)

/// Simplify implementation of the `Colorable` trait.
macro_rules! impl_colorable(
    ($context:ident) => (
        impl<'a> ::color::Colorable<'a> for $context<'a> {
            #[inline]
            fn color(self, color: Color) -> $context<'a> {
                $context { maybe_color: Some(color), ..self }
            }
            #[inline]
            fn rgba(self, r: f32, g: f32, b: f32, a: f32) -> $context<'a> {
                $context { maybe_color: Some(Color::new(r, g, b, a)), ..self }
            }
        }
    )
)

/// Simplify implementation of the `Frameable` trait.
macro_rules! impl_frameable(
    ($context:ident) => (
        impl<'a> ::frame::Frameable<'a> for $context<'a> {
            #[inline]
            fn frame(self, width: f64, color: ::color::Color) -> $context<'a> {
                $context { maybe_frame: Some((width, color)), ..self }
            }
        }
    )
)


/// Simplify implementation of the `Labelable` trait.
macro_rules! impl_labelable(
    ($context:ident) => (
        impl<'a> ::label::Labelable<'a> for $context<'a> {
            #[inline]
            fn label(self, text: &'a str, size: u32,
                     color: ::color::Color) -> $context<'a> {
                $context { maybe_label: Some((text, size, color)), ..self }
            }
        }
    )
)

/// Simplify implementation of the `Positionable` trait.
macro_rules! impl_positionable(
    ($context:ident) => (
        impl<'a> ::position::Positionable for $context<'a> {
            #[inline]
            fn position(self, x: f64, y: f64) -> $context<'a> {
                $context { pos: Point::new(x, y, 0.0), ..self }
            }
        }
    )
)
/// Simplify implementation of the `Shapeable` trait.
macro_rules! impl_shapeable(
    ($context:ident) => (
        impl<'a> ::shape::Shapeable for $context<'a> {
            #[inline]
            fn dimensions(self, width: f64, height: f64) -> $context<'a> {
                $context { width: width, height: height, ..self }
            }
        }
    )
)

