
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

/// Simplify implementation of the `Positionable` trait.
macro_rules! impl_positionable(
    ($context:ident, $($t:ident),*) => (
        impl<'a $(,$t)*> ::position::Positionable for $context<'a $(,$t)*> {

            #[inline]
            fn point(self, pos: Point) -> $context<'a $(,$t)*> {
                $context { pos: pos, ..self }
            }

            #[inline]
            fn position(self, x: f64, y: f64) -> $context<'a $(,$t)*> {
                $context { pos: [x, y], ..self }
            }

            #[inline]
            fn down(self, padding: f64, uic: &UiContext) -> $context<'a $(,$t)*> {
                let (x, y) = uic.get_placing(uic.get_prev_uiid()).down(padding);
                $context { pos: [x, y], ..self }
            }
            #[inline]
            fn up(self, padding: f64, uic: &UiContext) -> $context<'a $(,$t)*> {
                let (x, y) = uic.get_placing(uic.get_prev_uiid()).up(padding);
                $context { pos: [x, y], ..self }
            }
            #[inline]
            fn left(self, padding: f64, uic: &UiContext) -> $context<'a $(,$t)*> {
                let (x, y) = uic.get_placing(uic.get_prev_uiid()).left(padding);
                $context { pos: [x, y], ..self }
            }
            #[inline]
            fn right(self, padding: f64, uic: &UiContext) -> $context<'a $(,$t)*> {
                let (x, y) = uic.get_placing(uic.get_prev_uiid()).right(padding);
                $context { pos: [x, y], ..self }
            }

            #[inline]
            fn down_from(self, uiid: u64, padding: f64, uic: &UiContext) -> $context<'a $(,$t)*> {
                let (x, y) = uic.get_placing(uiid).down(padding);
                $context { pos: [x, y], ..self }
            }
            #[inline]
            fn up_from(self, uiid: u64, padding: f64, uic: &UiContext) -> $context<'a $(,$t)*> {
                let (x, y) = uic.get_placing(uiid).up(padding);
                $context { pos: [x, y], ..self }
            }
            #[inline]
            fn left_from(self, uiid: u64, padding: f64, uic: &UiContext) -> $context<'a $(,$t)*> {
                let (x, y) = uic.get_placing(uiid).left(padding);
                $context { pos: [x, y], ..self }
            }
            #[inline]
            fn right_from(self, uiid: u64, padding: f64, uic: &UiContext) -> $context<'a $(,$t)*> {
                let (x, y) = uic.get_placing(uiid).right(padding);
                $context { pos: [x, y], ..self }
            }

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
