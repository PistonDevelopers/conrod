//! A backend for converting src events to conrod's `Input` type.

use conrod_core::{Point, Scalar, input, event};
pub use piston_input::{GenericEvent, UpdateEvent};

/// Converts any `GenericEvent` to an `Input` event for conrod.
///
/// The given `width` and `height` must be `Scalar` (DPI agnostic) values.
pub fn convert<E>(event: E, win_w: Scalar, win_h: Scalar) -> Option<event::Input>
    where E: GenericEvent,
{
    // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
    let translate_coords = |xy: Point| (xy[0] - win_w / 2.0, -(xy[1] - win_h / 2.0));

    if let Some(xy) = event.mouse_cursor_args() {
        let (x, y) = translate_coords(xy);
        return Some(event::Input::Motion(input::Motion::MouseCursor { x: x, y: y }));
    }

    if let Some(rel_xy) = event.mouse_relative_args() {
        let (rel_x, rel_y) = translate_coords(rel_xy);
        return Some(event::Input::Motion(input::Motion::MouseRelative { x: rel_x, y: rel_y }));
    }

    if let Some(xy) = event.mouse_scroll_args() {
        // Invert the scrolling of the *y* axis as *y* is up in conrod.
        let (x, y) = (xy[0], -xy[1]);
        return Some(event::Input::Motion(input::Motion::Scroll { x: x, y: y }));
    }

    if let Some(args) = event.controller_axis_args() {
        return Some(event::Input::Motion(input::Motion::ControllerAxis(args)));
    }

    if let Some(args) = event.touch_args() {
        let id = input::touch::Id::new(args.id as u64);
        let xy = [args.x, args.y];
        let phase = match args.touch {
            ::piston_input::Touch::Start => input::touch::Phase::Start,
            ::piston_input::Touch::Move => input::touch::Phase::Move,
            ::piston_input::Touch::Cancel => input::touch::Phase::Cancel,
            ::piston_input::Touch::End => input::touch::Phase::End,
        };
        let touch = input::Touch { id: id, xy: xy, phase: phase };
        return Some(event::Input::Touch(touch));
    }

    if let Some(button) = event.press_args() {
        return Some(event::Input::Press(button));
    }

    if let Some(button) = event.release_args() {
        return Some(event::Input::Release(button));
    }

    if let Some(text) = event.text_args() {
        return Some(event::Input::Text(text));
    }

    if let Some(dim) = event.resize_args() {
        return Some(event::Input::Resize(dim[0], dim[1]));
    }

    if let Some(b) = event.focus_args() {
        return Some(event::Input::Focus(b));
    }

    None
}
