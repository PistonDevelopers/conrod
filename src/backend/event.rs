//! The backend for conrod's event system.
//!
//! Conrod's event system looks like this:
//!
//! *RawEvent -> Ui -> UiEvent -> Widget*
//!
//! The **Ui** receives **RawEvent**s such as `Press` and `Release` via the `Ui::handle_event`
//! method. It interprets these **RawEvent**s to create higher-level **UiEvent**s such as
//! `DoubleClick`, `WidgetCapturesKeyboard`, etc. These **UiEvent**s are stored and then fed to
//! each **Widget** when `Ui::set_widgets` is called. At the end of `Ui::set_widgets` the stored
//! **UiEvent**s are flushed ready for the next incoming **RawEvent**s.
//!
//! Conrod uses the `pistoncore-input` crate's `Event` type as the [**RawEvent**
//! type](./type.RawEvent.html). There are a few reasons for this:
//!
//! 1. This `Event` type already provides a number of useful variants of events that we wish to
//!    provide and handle within conrod, and we do not yet see any great need to re-write it and
//!    duplicate code.
//! 2. The `Event` type is already compatible with all `pistoncore-window` backends including
//!    `glfw_window`, `sdl2_window` and `glutin_window`.
//! 3. The `pistoncore-input` crate also provides a `GenericEvent` trait which allows us to easily
//!    provide a blanket implementation of `ToRawEvent` for all event types that already implement
//!    this trait.
//!
//! Because we use the `pistoncore-input` `Event` type, we also re-export its associated data
//! types (`Button`, `ControllerAxisArgs`, `Key`, etc).
//!
//! The [**ToRawEvent** trait](./trait.ToRawEvent.html) should be implemented for event types that
//! are to be passed to the `Ui::handle_event` method. As mentioned above, a blanket implementation
//! is already provided for all types that implement `pistoncore-input::GenericEvent`, so users of
//! `glfw_window`, `sdl2_window` and `glutin_window` need not concern themselves with this trait.

use {Point, Scalar};

#[doc(inline)]
pub use piston_input::{
    self,
    Button,
    Key,
    keyboard,
    Motion,
    MouseButton,
    Input,
};
#[doc(inline)]
pub use piston_input::keyboard::ModifierKey;

/// The event type that is used by conrod to track inputs from the world. This can be thought of as
/// the event type which is supplied by the window backend to drive the state of the `Ui` forward.
///
/// This type is primarily used within the `Ui::handle_event` method which immediately converts
/// given user events to `RawEvent`s via the [**ToRawEvent** trait](./trait.ToRawEvent.html) bound.
/// The `RawEvent`s are interpreted to create higher level `UiEvent`s (such as DoubleClick,
/// WidgetCapturesKeyboard, etc) which are stored for later processing by `Widget`s, which will
/// occur during the call to `Ui::set_widgets`.
pub type RawEvent = piston_input::Event<piston_input::Input>;

/// Unfortunately we're unable to access variants of the `Event` enum via the `RawEvent` type
/// alias above, thus we must also re-export `Event` itself so that a user may use it if they
/// require accessing its variants.
pub use piston_input::Event;

/// Types that may be converted to the `RawEvent` type used by conrod to track events happening in
/// the world.
///
/// This is only used by conrod as a bound for the `Ui::handle_event` method.
pub trait ToRawEvent {
    /// Convert self into an `RawEvent`.
    ///
    /// **Note:** `RawEvent`s that contain co-ordinates must be oriented with (0, 0) at the middle
    /// of the window with the `y` axis pointing upwards (Cartesian co-ordinates). The window
    /// dimensions are provided in case co-ordinates require translation.
    fn to_raw_event(self, win_w: Scalar, win_h: Scalar) -> Option<RawEvent>;
}

impl<E> ToRawEvent for E
    where E: piston_input::GenericEvent,
{
    fn to_raw_event(self, win_w: Scalar, win_h: Scalar) -> Option<RawEvent> {
        use piston_input::{
            RenderEvent,
            MouseCursorEvent,
            MouseRelativeEvent,
            MouseScrollEvent,
            PressEvent,
            ReleaseEvent,
            ResizeEvent,
            FocusEvent,
            TextEvent,
            CursorEvent,
        };

        // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
        let translate_coords = |xy: Point| (xy[0] - win_w / 2.0, -(xy[1] - win_h / 2.0));

        if let Some(args) = self.render_args() {
            return Some(Event::Render(args));
        }

        if let Some(xy) = self.mouse_cursor_args() {
            let (x, y) = translate_coords(xy);
            return Some(Input::Move(Motion::MouseCursor(x, y)).into());
        }

        if let Some(rel_xy) = self.mouse_relative_args() {
            let (rel_x, rel_y) = translate_coords(rel_xy);
            return Some(Input::Move(Motion::MouseRelative(rel_x, rel_y)).into());
        }

        if let Some(xy) = self.mouse_scroll_args() {
            return Some(Input::Move(Motion::MouseScroll(xy[0], xy[1])).into());
        }

        if let Some(button) = self.press_args() {
            return Some(Input::Press(button).into());
        }

        if let Some(button) = self.release_args() {
            return Some(Input::Release(button).into());
        }

        if let Some(text) = self.text_args() {
            return Some(Input::Text(text).into());
        }

        if let Some(dim) = self.resize_args() {
            return Some(Input::Resize(dim[0], dim[1]).into());
        }

        if let Some(b) = self.focus_args() {
            return Some(Input::Focus(b).into());
        }

        if let Some(b) = self.cursor_args() {
            return Some(Input::Cursor(b).into());
        }

        None
    }
}
