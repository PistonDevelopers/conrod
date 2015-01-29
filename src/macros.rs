
/// Simplify implementation of BoilerPlate widget module functions.
macro_rules! widget_fns(
    ($widget:ident, $widget_state:ident, $default:expr) => (

        /// Default Widget variant.
        fn default() -> ::widget::Widget { $default }

        /// Get a reference to the widget associated with the given UIID.
        fn get_widget(uic: &mut ::ui_context::UiContext,
                      ui_id: ::ui_context::UIID) -> &mut ::widget::Widget {
            uic.get_widget(ui_id, default())
        }

        /// Get the current State for the widget.
        fn get_state(uic: &mut ::ui_context::UiContext,
                     ui_id: ::ui_context::UIID) -> &$widget_state {
            match *get_widget(uic, ui_id) {
                ::widget::Widget::$widget(ref state) => state,
                _ => panic!("The Widget variant returned by UiContext is different to that which \
                           was requested (Check that there are no UIID conflicts)."),
            }
        }

        /// Set the state for the widget in the UiContext.
        fn set_state(uic: &mut ::ui_context::UiContext,
                     ui_id: ::ui_context::UIID,
                     new_state: $widget_state,
                     pos: ::point::Point,
                     dim: ::dimensions::Dimensions) {
            match *get_widget(uic, ui_id) {
                ::widget::Widget::$widget(ref mut state) => { *state = new_state; },
                _ => panic!("The Widget variant returned by UiContext is different to that which \
                           was requested (Check that there are no UIID conflicts)."),
            }
            uic.set_place(ui_id, pos, dim);
        }

    )
);

/// Simplify implementation of the `Shapeable` trait.
macro_rules! impl_shapeable(
    ($context:ident, $($t:ident),*) => (
        impl<'a $(, $t)*> ::shape::Shapeable for $context<'a $(, $t)*> {
            #[inline]
            fn dimensions(self, width: f64, height: f64) -> $context<'a $(, $t)*> {
                $context { dim: [width, height], ..self }
            }
            #[inline]
            fn dim(self, dim: ::dimensions::Dimensions) -> $context<'a $(, $t)*> {
                $context { dim: dim, ..self }
            }
            #[inline]
            fn width(self, width: f64) -> $context<'a $(, $t)*> {
                $context { dim: [width, self.dim[1]], ..self }
            }
            #[inline]
            fn height(self, height: f64) -> $context<'a $(, $t)*> {
                $context { dim: [self.dim[0], height], ..self }
            }
        }
    )
);
