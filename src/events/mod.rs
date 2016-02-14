//! This module contains all the logic for handling input events and providing them to widgets.
//! All user input is provided to the `Ui` in the form of `input::Input` events, which are continuously
//! polled from the backend window implementation. These raw input events tent to be fairly low level.
//! The `Ui` passes each of these events off to it's `GlobalInput`, which keeps track of the state of
//! affairs for the entire `Ui`. `GlobalInput` will also aggregate the low level events into higher
//! level ones. For instance, two events indicating that a mouse button was pressed then released
//! would cause a new `UiEvent::MouseClick` to be generated. This saves individual widgets from
//! having to interpret these themselves, thus freeing them from also having to store input state.
//!
//! Whenever there's an update, all of the events that have occured since the last update will be
//! available for widgets to process. This is where the `InputProvider` trait comes in. The
//! `InputProvider` trait provides many methods for conveniently filtering events that a widget would
//! like to handle. There are two things that implement this trait. The first is `GlobalInput`, and
//! the second is `WidgetInput`. `WidgetInput` is used to provide input events to a specific widget.
//! It filters events that don't apply to the widget, and all events provided by `WidgetIput` will
//! have all coordinates in the widget's own local coordinate system. `GlobalInput`, on the other hand,
//! will never filter out any events, and will always provide them with coordinates relative to the
//! window.

pub mod ui_event;
pub mod input_state;
pub mod widget_input;
pub mod global_input;
pub mod input_provider;

pub use self::input_state::{InputState, ButtonMap};
pub use self::global_input::{GlobalInputEventIterator, GlobalInput};
pub use self::widget_input::{WidgetInputEventIterator, WidgetInput};
pub use self::ui_event::{UiEvent, MouseClick, MouseDrag, Scroll};
pub use self::input_provider::InputProvider;
