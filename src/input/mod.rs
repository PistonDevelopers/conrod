//! This module contains all the logic for handling input events and providing them to widgets.
//!
//! All user input is provided to the `Ui` in the form of `input::Input` events, which are received
//! via the `Ui::handle_event` method. These raw input events tend to be fairly low level. The `Ui`
//! stores each of these `Input` events in it's `GlobalInput`, which keeps track of the state of
//! input for the entire `Ui`. `GlobalInput` will also aggregate the low level events into higher
//! level ones. For instance, two events indicating that a mouse button was pressed then released
//! would cause a new `UiEvent::MouseClick` to be generated. This saves individual widgets from
//! having to interpret these themselves, thus freeing them from also having to store input state.
//!
//! Whenever there's an update, all of the events that have occured since the last update will be
//! available for widgets to process. `WidgetInput` is used to provide input events to a specific
//! widget. It filters events that do not apply to the widget. All events provided by `WidgetIput`
//! will have all coordinates in the widget's own local coordinate system, where `(0, 0)` is the
//! middle of the widget's bounding `Rect`. `GlobalInput`, on the other hand, will never filter out
//! any events, and will always provide them with coordinates relative to the window.

pub mod state;
pub mod widget;
pub mod global;

pub use self::state::State;
pub use self::global::Global;
pub use self::widget::Widget;

#[doc(inline)]
pub use piston_input::keyboard::ModifierKey;
#[doc(inline)]
pub use piston_input::{
    Button,
    ControllerButton,
    ControllerAxisArgs,
    keyboard,
    Key,
    MouseButton,
    RenderArgs,
    Touch,
    TouchArgs,
};
