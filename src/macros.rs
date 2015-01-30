
/// Simplify implementation of BoilerPlate widget module functions.
macro_rules! widget_fns(
    ($widget:ident, $widget_state:ident, $default:expr) => (

        /// Default Widget variant.
        fn default() -> ::widget::Widget { $default }

        /// Get a reference to the widget associated with the given UIID.
        fn get_widget<C>(
            uic: &mut ::ui_context::UiContext<C>,
            ui_id: ::ui_context::UIID
        ) -> &mut ::widget::Widget {
            uic.get_widget(ui_id, default())
        }

        /// Get the current State for the widget.
        fn get_state<C>(
            uic: &mut ::ui_context::UiContext<C>,
            ui_id: ::ui_context::UIID
        ) -> &$widget_state {
            match *get_widget(uic, ui_id) {
                ::widget::Widget::$widget(ref state) => state,
                _ => panic!("The Widget variant returned by UiContext is different to that which \
                           was requested (Check that there are no UIID conflicts)."),
            }
        }

        /// Set the state for the widget in the UiContext.
        fn set_state<C>(
            uic: &mut ::ui_context::UiContext<C>,
            ui_id: ::ui_context::UIID,
            new_state: ::widget::Widget,
            pos: ::point::Point,
            dim: ::dimensions::Dimensions
        ) {
            match *get_widget(uic, ui_id) {
                ref mut state => {
                    if !state.matches(&new_state) {
                        panic!("The Widget variant returned by UiContext is different to that which \
                                   was requested (Check that there are no UIID conflicts).");
                    }
                    *state = new_state;
                }
            }
            uic.set_place(ui_id, pos, dim);
        }

    )
);
