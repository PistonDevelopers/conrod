//! A backend for converting piston events to conrod's `Input` type.

use {Point, Scalar};
use event::{Input, Motion};
pub use piston_input::{GenericEvent, UpdateEvent};

/// Converts any `GenericEvent` to an `Input` event for conrod.
///
/// The given `width` and `height` must be `Scalar` (DPI agnostic) values.
pub fn convert<E>(event: E, win_w: Scalar, win_h: Scalar) -> Option<Input>
    where E: GenericEvent,
{
    // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
    let translate_coords = |xy: Point| (xy[0] - win_w / 2.0, -(xy[1] - win_h / 2.0));

    if let Some(xy) = event.mouse_cursor_args() {
        let (x, y) = translate_coords(xy);
        return Some(Input::Move(Motion::MouseCursor(x, y)));
    }

    if let Some(rel_xy) = event.mouse_relative_args() {
        let (rel_x, rel_y) = translate_coords(rel_xy);
        return Some(Input::Move(Motion::MouseRelative(rel_x, rel_y)));
    }

    if let Some(xy) = event.mouse_scroll_args() {
        // Invert the scrolling of the *y* axis as *y* is up in conrod.
        let (x, y) = (xy[0], -xy[1]);
        return Some(Input::Move(Motion::MouseScroll(x, y)));
    }

    if let Some(args) = event.controller_axis_args() {
        return Some(Input::Move(Motion::ControllerAxis(args)));
    }

    if let Some(button) = event.press_args() {
        return Some(Input::Press(button));
    }

    if let Some(button) = event.release_args() {
        return Some(Input::Release(button));
    }

    if let Some(text) = event.text_args() {
        return Some(Input::Text(text));
    }

    if let Some(dim) = event.resize_args() {
        return Some(Input::Resize(dim[0], dim[1]));
    }

    if let Some(b) = event.focus_args() {
        return Some(Input::Focus(b));
    }

    if let Some(b) = event.cursor_args() {
        return Some(Input::Cursor(b));
    }

    None
}
