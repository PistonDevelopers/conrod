use super::sdl2::event::Event;
use super::sdl2::mouse::MouseButton;

use input::state::mouse;
use {Point, Scalar};
use event::Input;
use input;

pub fn convert(event: Event, win_w: Scalar, win_h: Scalar) -> Option<Input> {
    // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
    let translate_coords = |xy: Point| (xy[0] - win_w / 2.0, -(xy[1] - win_h / 2.0));

    match event {
        Event::MouseWheel { x, y, .. } => Some(Input::Motion(input::Motion::Scroll {
            x: x as Scalar * 20.0,
            y: -(y as Scalar) * 20.0
        })),
        Event::MouseMotion {x, y, .. } => {
            let (x, y) = translate_coords([x as Scalar, y as Scalar]);
            Some(Input::Motion(input::Motion::MouseCursor { x, y }))
        }
        Event::MouseButtonDown { mouse_btn, .. } => {
            Some(Input::Press(input::Button::Mouse(convert_mouse_btn(mouse_btn))))
        }
        Event::MouseButtonUp { mouse_btn, .. } => {
            Some(Input::Release(input::Button::Mouse(convert_mouse_btn(mouse_btn))))
        }
        _ => None,
    }

    /*
    if let Some(xy) = event.mouse_cursor_args() {
        let (x, y) = translate_coords(xy);
        return Some(Input::Motion(input::Motion::MouseCursor { x: x, y: y }));
    }

    if let Some(rel_xy) = event.mouse_relative_args() {
        let (rel_x, rel_y) = translate_coords(rel_xy);
        return Some(Input::Motion(input::Motion::MouseRelative { x: rel_x, y: rel_y }));
    }

    if let Some(xy) = event.mouse_scroll_args() {
        // Invert the scrolling of the *y* axis as *y* is up in conrod.
        let (x, y) = (xy[0], -xy[1]);
        return Some(Input::Motion(input::Motion::Scroll { x: x, y: y }));
    }

    if let Some(args) = event.controller_axis_args() {
        return Some(Input::Motion(input::Motion::ControllerAxis(args)));
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
        return Some(Input::Touch(touch));
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

    None
    */
}

fn convert_mouse_btn(btn: MouseButton) -> mouse::Button {
    match btn {
        MouseButton::Left => mouse::Button::Left,
        MouseButton::Right => mouse::Button::Right,
        MouseButton::Middle => mouse::Button::Middle,
        MouseButton::X1 => mouse::Button::X1,
        MouseButton::X2 => mouse::Button::X2,
        MouseButton::Unknown => mouse::Button::Unknown,
    }
}
