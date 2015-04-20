
/// Simplify implementation of BoilerPlate widget module functions.
macro_rules! widget_fns(
    ($widget:ident, $widget_state:ident, $default:expr) => (

        /// Default Widget state.
        fn default() -> ::widget::Kind { $default }

        /// Get a reference to the widget associated with the given UiId.
        fn get_widget<C>(
            ui: &mut ::ui::Ui<C>,
            ui_id: ::ui::UiId
        ) -> &mut ::widget::Kind {
            ui.get_widget(ui_id, default())
        }

        /// Get the current State for the widget.
        fn get_state<C>(
            ui: &mut ::ui::Ui<C>,
            ui_id: ::ui::UiId
        ) -> &$widget_state {
            match *get_widget(ui, ui_id) {
                ::widget::Kind::$widget(ref state) => state,
                _ => panic!("The Kind variant returned by Ui is different to that which \
                           was requested (Check that there are no UiId conflicts)."),
            }
        }

    )
);
