
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
                _ => fail!("The Widget variant returned by UIContext is different to that which \
                           was requested (Check that there are no UIID conflicts)."),
            }
        }

        /// Set the state for the widget in the UIContext.
        fn set_state(uic: &mut ::ui_context::UIContext,
                     ui_id: ::ui_context::UIID,
                     new_state: $widget_state,
                     x: f64, y: f64, w: f64, h: f64) {
            match *get_widget(uic, ui_id) {
                ::widget::$widget(ref mut state) => { *state = new_state; },
                _ => fail!("The Widget variant returned by UIContext is different to that which \
                           was requested (Check that there are no UIID conflicts)."),
            }
            uic.set_place(ui_id, x, y, w, h);
        }

    )
)

/// Simplify implementation of the `Colorable` trait.
macro_rules! impl_callable(
    ($context:ident, $cb:ty $(, $t:ident)*) => (
        impl<'a $(, $t)*> ::callback::Callable<$cb> for $context<'a $(, $t)*> {
            #[inline]
            fn callback(self, callback: $cb) -> $context<'a $(, $t)*> {
                $context { maybe_callback: Some(callback), ..self }
            }
        }
    )
)

/// Simplify implementation of the `Colorable` trait.
macro_rules! impl_colorable(
    ($context:ident $(, $t:ident)*) => (
        impl<'a $(, $t)*> ::color::Colorable for $context<'a $(, $t)*> {
            #[inline]
            fn color(self, color: Color) -> $context<'a $(, $t)*> {
                $context { maybe_color: Some(color), ..self }
            }
            #[inline]
            fn rgba(self, r: f32, g: f32, b: f32, a: f32) -> $context<'a $(, $t)*> {
                $context { maybe_color: Some(Color::new(r, g, b, a)), ..self }
            }
        }
    )
)

/// Simplify implementation of the `Frameable` trait.
macro_rules! impl_frameable(
    ($context:ident $(, $t:ident)*) => (
        impl<'a $(, $t)*> ::frame::Frameable for $context<'a $(, $t)*> {
            #[inline]
            fn frame(self, width: f64, color: ::color::Color) -> $context<'a $(, $t)*> {
                $context { maybe_frame: Some((width, color)), ..self }
            }
        }
    )
)


/// Simplify implementation of the `Labelable` trait.
macro_rules! impl_labelable(
    ($context:ident $(, $t:ident)*) => (
        impl<'a $(, $t)*> ::label::Labelable<'a> for $context<'a $(, $t)*> {
            #[inline]
            fn label(self, text: &'a str, size: u32,
                     color: ::color::Color) -> $context<'a $(, $t)*> {
                $context { maybe_label: Some((text, size, color)), ..self }
            }
        }
    )
)

/// Simplify implementation of the `Positionable` trait.
macro_rules! impl_positionable(
    ($context:ident $(, $t:ident)*) => (
        impl<'a $(,$t)*> ::position::Positionable for $context<'a $(,$t)*> {

            #[inline]
            fn position(self, x: f64, y: f64) -> $context<'a $(,$t)*> {
                $context { pos: Point::new(x, y, 0.0), ..self }
            }

            #[inline]
            fn down(self, padding: f64) -> $context<'a $(,$t)*> {
                let (x, y) = self.uic.get_placing(self.uic.get_prev_uiid()).down(padding);
                $context { pos: Point::new(x, y, 0.0), ..self }
            }
            #[inline]
            fn up(self, padding: f64) -> $context<'a $(,$t)*> {
                let (x, y) = self.uic.get_placing(self.uic.get_prev_uiid()).up(padding);
                $context { pos: Point::new(x, y, 0.0), ..self }
            }
            #[inline]
            fn left(self, padding: f64) -> $context<'a $(,$t)*> {
                let (x, y) = self.uic.get_placing(self.uic.get_prev_uiid()).left(padding);
                $context { pos: Point::new(x, y, 0.0), ..self }
            }
            #[inline]
            fn right(self, padding: f64) -> $context<'a $(,$t)*> {
                let (x, y) = self.uic.get_placing(self.uic.get_prev_uiid()).right(padding);
                $context { pos: Point::new(x, y, 0.0), ..self }
            }

            #[inline]
            fn down_from(self, uiid: u64, padding: f64) -> $context<'a $(,$t)*> {
                let (x, y) = self.uic.get_placing(uiid).down(padding);
                $context { pos: Point::new(x, y, 0.0), ..self }
            }
            #[inline]
            fn up_from(self, uiid: u64, padding: f64) -> $context<'a $(,$t)*> {
                let (x, y) = self.uic.get_placing(uiid).up(padding);
                $context { pos: Point::new(x, y, 0.0), ..self }
            }
            #[inline]
            fn left_from(self, uiid: u64, padding: f64) -> $context<'a $(,$t)*> {
                let (x, y) = self.uic.get_placing(uiid).left(padding);
                $context { pos: Point::new(x, y, 0.0), ..self }
            }
            #[inline]
            fn right_from(self, uiid: u64, padding: f64) -> $context<'a $(,$t)*> {
                let (x, y) = self.uic.get_placing(uiid).right(padding);
                $context { pos: Point::new(x, y, 0.0), ..self }
            }

        }
    )
)

/// Simplify implementation of the `Shapeable` trait.
macro_rules! impl_shapeable(
    ($context:ident $(, $t:ident)*) => (
        impl<'a $(, $t)*> ::shape::Shapeable for $context<'a $(, $t)*> {
            #[inline]
            fn dimensions(self, width: f64, height: f64) -> $context<'a $(, $t)*> {
                $context { width: width, height: height, ..self }
            }
        }
    )
)

