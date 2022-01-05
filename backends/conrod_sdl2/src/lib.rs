/// A function for converting a `sdl2::event::Event` to a `conrod::event::Input`.
///
/// This can be useful for single-window applications. Note that win_w and win_h should be the dpi
/// agnostic values (i.e. pass window.size() values and NOT window.drawable_size())
///
/// NOTE: The sdl2 MouseMotion event is a combination of a MouseCursor and MouseRelative conrod
/// events. Thus we may sometimes return two events in place of one, hence the tuple return type
pub fn convert_event(e: event::Event, win_w: Scalar, win_h: Scalar) -> (Option<Input>, Option<Input>)
{
    use sdl2::event::{Event, WindowEvent};

    // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
    //
    // winit produces input events in pixels, so these positions need to be divided by the width
    // and height of the window in order to be DPI agnostic.
    let tx = |x: f32| ((x as Scalar) - win_w / 2.0) as Scalar;
    let ty = |y: f32| -((y as Scalar) - win_h / 2.0) as Scalar;

    match e {
        Event::Window { win_event, .. } =>
            (match win_event {
                WindowEvent::Resized(w, h) => {
                    Some(Input::Resize(w as u32, h as u32))
                },

                WindowEvent::FocusGained =>
                    Some(Input::Focus(true)),

                WindowEvent::FocusLost =>
                    Some(Input::Focus(false)),

                _ => None,
            }, None),
        Event::TextInput { text, .. } => {
            (Some(Input::Text(text)), None)
        },

        Event::KeyDown { keycode: Some(key), .. } => {
            (Some(Input::Press(input::Button::Keyboard(map_key(key)))), None)
        },

        Event::KeyUp { keycode: Some(key), .. } => {
            (Some(Input::Release(input::Button::Keyboard(map_key(key)))), None)
        },

        Event::FingerDown { touch_id, x, y, .. } => {
            let xy = [x as f64, y as f64];
            let id = input::touch::Id::new(touch_id as u64);
            let phase = input::touch::Phase::Start;
            let touch = input::Touch { phase, id, xy };
            (Some(Input::Touch(touch)), None)
        },

        Event::FingerMotion { touch_id, x, y, .. } => {
            let xy = [x as f64, y as f64];
            let id = input::touch::Id::new(touch_id as u64);
            let phase = input::touch::Phase::Move;
            let touch = input::Touch { phase, id, xy };
            (Some(Input::Touch(touch)), None)
        },

        Event::FingerUp { touch_id, x, y, .. } => {
            let xy = [x as f64, y as f64];
            let id = input::touch::Id::new(touch_id as u64);
            let phase = input::touch::Phase::End;
            let touch = input::Touch { phase, id, xy };
            (Some(Input::Touch(touch)), None)
        },

        Event::MouseMotion { x, y, xrel, yrel, .. } => {
            let cursor = input::Motion::MouseCursor { x: tx(x as f32), y: ty(y as f32) };
            let relative = input::Motion::MouseRelative { x: tx(xrel as f32), y: ty(yrel as f32) };
            (Some(Input::Motion(cursor)), Some(Input::Motion(relative)))
        },

        Event::MouseWheel { x, y, .. } => {
            // Invert the scrolling of the *y* axis as *y* is up in conrod.
            const ARBITRARY_POINTS_PER_LINE_FACTOR: Scalar = 10.0;
            let x = ARBITRARY_POINTS_PER_LINE_FACTOR * x as Scalar;
            let y = ARBITRARY_POINTS_PER_LINE_FACTOR * (-y) as Scalar;
            let motion = input::Motion::Scroll { x, y };
            (Some(Input::Motion(motion)), None)
        },

        Event::MouseButtonDown { mouse_btn, .. } =>
            (Some(Input::Press(input::Button::Mouse(map_mouse(mouse_btn)))), None),

        Event::MouseButtonUp { mouse_btn, .. } =>
            (Some(Input::Release(input::Button::Mouse(map_mouse(mouse_btn)))), None),

        Event::JoyAxisMotion{ which, axis_idx, value, .. } => {
            // Axis motion is an absolute value in the range
            // [-32768, 32767]. Normalize it down to a float.
            use std::i16::MAX;
            let normalized_value = value as f64 / MAX as f64;
            (Some(Input::Motion(input::Motion::ControllerAxis(ControllerAxisArgs::new(
                            which, axis_idx, normalized_value)))), None)
        }

        _ => (None, None),
    }
}

/// Maps sdl2's key to a conrod `Key`.
pub fn map_key(keycode: sdl2::keyboard::Keycode) -> input::keyboard::Key {
    (keycode as u32).into()
}

/// Maps sdl2's mouse button to conrod's mouse button.
pub fn map_mouse(mouse_button: sdl2::mouse::MouseButton) -> input::MouseButton {
    use input::MouseButton;
    match mouse_button {
        sdl2::mouse::MouseButton::Left => MouseButton::Left,
        sdl2::mouse::MouseButton::Right => MouseButton::Right,
        sdl2::mouse::MouseButton::Middle => MouseButton::Middle,
        sdl2::mouse::MouseButton::X1 => MouseButton::X1,
        sdl2::mouse::MouseButton::X2 => MouseButton::X2,
        _ => MouseButton::Unknown
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
