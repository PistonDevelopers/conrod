//! The backend for conrod's handling of events.
//!
//! The [**ToEvent** trait](./trait.ToEvent.html) should be implemented for event types that are to
//! be passed to the `Ui::handle_event` method.

use input;
use {Point, Scalar};

#[doc(inline)]
pub use input::Input;

/// Types that may be converted to the `Event` type used by conrod to track events happening in the
/// world.
pub trait ToEvent {
    /// Convert self into an `Event`.
    ///
    /// Events that contain co-ordinates must be oriented with (0, 0) at the middle of the window
    /// with the `y` axis pointing upwards (Cartesian co-ordinates). The window dimensions are
    /// provided in case the co-ordinates require translation.
    fn to_event(self, win_w: Scalar, win_h: Scalar) -> Option<Event>;
}

/// The event type that is used by conrod to track inputs from the world.
///
/// Specificly, the `Ui::handle_event` method immediately converts given events to this event type,
/// reacts to them and stores them for later processing by `Widget`s.
pub type Event = input::Event<input::Input>;


impl<E> ToEvent for E
    where E: input::GenericEvent,
{
    fn to_event(self, win_w: Scalar, win_h: Scalar) -> Option<Event> {
        use input::{
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
            println!("render: {:?}", args);
            return Some(input::Event::Render(args));
        }

        if let Some(xy) = self.mouse_cursor_args() {
            let (x, y) = translate_coords(xy);
            println!("mouse_cursor: {:?}", (x, y));
            return Some(input::Input::Move(input::Motion::MouseCursor(x, y)).into());
        }

        if let Some(rel_xy) = self.mouse_relative_args() {
            let (rel_x, rel_y) = translate_coords(rel_xy);
            println!("mouse_relative: {:?}", (rel_x, rel_y));
            return Some(input::Input::Move(input::Motion::MouseRelative(rel_x, rel_y)).into());
        }

        if let Some(xy) = self.mouse_scroll_args() {
            println!("mouse_scroll: {:?}", xy);
            return Some(input::Input::Move(input::Motion::MouseScroll(xy[0], xy[1])).into());
        }

        if let Some(button) = self.press_args() {
            println!("press: {:?}", button);
            return Some(input::Input::Press(button).into());
        }

        if let Some(button) = self.release_args() {
            println!("release: {:?}", button);
            return Some(input::Input::Release(button).into());
        }

        if let Some(text) = self.text_args() {
            println!("text: {:?}", &text);
            return Some(input::Input::Text(text).into());
        }

        if let Some(dim) = self.resize_args() {
            println!("resize: {:?}", dim);
            return Some(input::Input::Resize(dim[0], dim[1]).into());
        }

        if let Some(b) = self.focus_args() {
            println!("focus: {:?}", b);
            return Some(input::Input::Focus(b).into());
        }

        if let Some(b) = self.cursor_args() {
            println!("cursor: {:?}", b);
            return Some(input::Input::Cursor(b).into());
        }

        None
    }
}
