
#![macro_escape]

/// Simplify implementation of ToPrimitive.
#[macro_export]
macro_rules! widget_state(
    ($obj:ty, $obji:ident {
        $($var:ident -> $val:expr),+
    }) => (

        /// Widget state.
        #[deriving(FromPrimitive, PartialEq)]
        pub enum $obji {
            $($var, )+
        }

        impl ToPrimitive for $obj {
            fn to_i64(&self) -> Option<i64> {
                match self {
                    $(&$var => Some($val as i64),)+
                    //_ => None,
                }
            }
            fn to_u64(&self) -> Option<u64> {
                match self {
                    $(&$var => Some($val as u64),)+
                    //_ => None,
                }
            }
        }
    )
)

/// Simplify implementation of BoilerPlate widget module functions.
macro_rules! widget_boilerplate_fns(
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
                     ui_id: ::ui_context::UIID) -> $widget_state {
            match *get_widget(uic, ui_id) {
                ::widget::$widget(state) => state,
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



