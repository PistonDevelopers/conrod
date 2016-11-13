//! A function for converting a `glutin::Event` to a `conrod::event::Input`.
//!
//! The following is adapted from the piston `glutin_window` crate.

extern crate glutin;

use Scalar;
use event::{Input, Motion};
use input;
use std;

/// A function for converting a `glutin::Event` to a `conrod::event::Input`.
pub fn convert<W>(e: glutin::Event, window: W) -> Option<Input>
    where W: std::ops::Deref<Target=glutin::Window>,
{

    // The window size in points.
    let (win_w, win_h) = match window.get_inner_size() {
        Some((w, h)) => (w as Scalar, h as Scalar),
        None => return None,
    };

    // The "dots per inch" factor. Multiplying this by `win_w` and `win_h` gives the framebuffer
    // width and height.
    let dpi_factor = window.hidpi_factor() as Scalar;

    // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
    //
    // Glutin produces input events in pixels, so these positions need to be divided by the widht
    // and height of the window in order to be DPI agnostic.
    let tx = |x: Scalar| (x / dpi_factor) - win_w / 2.0;
    let ty = |y: Scalar| -((y / dpi_factor) - win_h / 2.0);

    match e {

        glutin::Event::Resized(w, h) =>
            Some(Input::Resize(w, h).into()),

        glutin::Event::ReceivedCharacter(ch) => {
            let string = match ch {
                // Ignore control characters and return ascii for Text event (like sdl2).
                '\u{7f}' | // Delete
                '\u{1b}' | // Escape
                '\u{8}'  | // Backspace
                '\r' | '\n' | '\t' => "".to_string(),
                _ => ch.to_string()
            };
            Some(Input::Text(string).into())
        },

        glutin::Event::Focused(focused) =>
            Some(Input::Focus(focused).into()),

        glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(key)) =>
            Some(Input::Press(input::Button::Keyboard(map_key(key))).into()),

        glutin::Event::KeyboardInput(glutin::ElementState::Released, _, Some(key)) =>
            Some(Input::Release(input::Button::Keyboard(map_key(key))).into()),

        glutin::Event::Touch(glutin::Touch { phase, location, id }) => {
            let phase = match phase {
                glutin::TouchPhase::Started => input::Touch::Start,
                glutin::TouchPhase::Moved => input::Touch::Move,
                glutin::TouchPhase::Ended => input::Touch::End,
                glutin::TouchPhase::Cancelled => input::Touch::Cancel
            };
            let xy = [tx(location.0), ty(location.1)];
            let args = input::TouchArgs::new(0, id as i64, xy, 1.0, phase);
            Some(Input::Move(Motion::Touch(args)).into())
        }

        glutin::Event::MouseMoved(x, y) =>
            Some(Input::Move(Motion::MouseCursor(tx(x as Scalar), ty(y as Scalar))).into()),

        glutin::Event::MouseWheel(glutin::MouseScrollDelta::PixelDelta(x, y), _) => {
            let x = x as Scalar / dpi_factor;
            let y = -y as Scalar / dpi_factor;
            Some(Input::Move(Motion::MouseScroll(x, y)).into())
        },

        glutin::Event::MouseWheel(glutin::MouseScrollDelta::LineDelta(x, y), _) => {
            // This should be configurable (we should provide a LineDelta event to allow for this).
            const ARBITRARY_POINTS_PER_LINE_FACTOR: Scalar = 10.0;
            let x = ARBITRARY_POINTS_PER_LINE_FACTOR * x as Scalar;
            let y = ARBITRARY_POINTS_PER_LINE_FACTOR * -y as Scalar;
            Some(Input::Move(Motion::MouseScroll(x, y)).into())
        },

        glutin::Event::MouseInput(glutin::ElementState::Pressed, button) =>
            Some(Input::Press(input::Button::Mouse(map_mouse(button))).into()),

        glutin::Event::MouseInput(glutin::ElementState::Released, button) =>
            Some(Input::Release(input::Button::Mouse(map_mouse(button))).into()),

        _ => None,
    }
}

/// Maps Glutin's key to a conrod `Key`.
pub fn map_key(keycode: glutin::VirtualKeyCode) -> input::keyboard::Key {
    use input::keyboard::Key;

    match keycode {
        glutin::VirtualKeyCode::Key0 => Key::D0,
        glutin::VirtualKeyCode::Key1 => Key::D1,
        glutin::VirtualKeyCode::Key2 => Key::D2,
        glutin::VirtualKeyCode::Key3 => Key::D3,
        glutin::VirtualKeyCode::Key4 => Key::D4,
        glutin::VirtualKeyCode::Key5 => Key::D5,
        glutin::VirtualKeyCode::Key6 => Key::D6,
        glutin::VirtualKeyCode::Key7 => Key::D7,
        glutin::VirtualKeyCode::Key8 => Key::D8,
        glutin::VirtualKeyCode::Key9 => Key::D9,
        glutin::VirtualKeyCode::A => Key::A,
        glutin::VirtualKeyCode::B => Key::B,
        glutin::VirtualKeyCode::C => Key::C,
        glutin::VirtualKeyCode::D => Key::D,
        glutin::VirtualKeyCode::E => Key::E,
        glutin::VirtualKeyCode::F => Key::F,
        glutin::VirtualKeyCode::G => Key::G,
        glutin::VirtualKeyCode::H => Key::H,
        glutin::VirtualKeyCode::I => Key::I,
        glutin::VirtualKeyCode::J => Key::J,
        glutin::VirtualKeyCode::K => Key::K,
        glutin::VirtualKeyCode::L => Key::L,
        glutin::VirtualKeyCode::M => Key::M,
        glutin::VirtualKeyCode::N => Key::N,
        glutin::VirtualKeyCode::O => Key::O,
        glutin::VirtualKeyCode::P => Key::P,
        glutin::VirtualKeyCode::Q => Key::Q,
        glutin::VirtualKeyCode::R => Key::R,
        glutin::VirtualKeyCode::S => Key::S,
        glutin::VirtualKeyCode::T => Key::T,
        glutin::VirtualKeyCode::U => Key::U,
        glutin::VirtualKeyCode::V => Key::V,
        glutin::VirtualKeyCode::W => Key::W,
        glutin::VirtualKeyCode::X => Key::X,
        glutin::VirtualKeyCode::Y => Key::Y,
        glutin::VirtualKeyCode::Z => Key::Z,
        glutin::VirtualKeyCode::Apostrophe => Key::Unknown,
        glutin::VirtualKeyCode::Backslash => Key::Backslash,
        glutin::VirtualKeyCode::Back => Key::Backspace,
        // K::CapsLock => Key::CapsLock,
        glutin::VirtualKeyCode::Delete => Key::Delete,
        glutin::VirtualKeyCode::Comma => Key::Comma,
        glutin::VirtualKeyCode::Down => Key::Down,
        glutin::VirtualKeyCode::End => Key::End,
        glutin::VirtualKeyCode::Return => Key::Return,
        glutin::VirtualKeyCode::Equals => Key::Equals,
        glutin::VirtualKeyCode::Escape => Key::Escape,
        glutin::VirtualKeyCode::F1 => Key::F1,
        glutin::VirtualKeyCode::F2 => Key::F2,
        glutin::VirtualKeyCode::F3 => Key::F3,
        glutin::VirtualKeyCode::F4 => Key::F4,
        glutin::VirtualKeyCode::F5 => Key::F5,
        glutin::VirtualKeyCode::F6 => Key::F6,
        glutin::VirtualKeyCode::F7 => Key::F7,
        glutin::VirtualKeyCode::F8 => Key::F8,
        glutin::VirtualKeyCode::F9 => Key::F9,
        glutin::VirtualKeyCode::F10 => Key::F10,
        glutin::VirtualKeyCode::F11 => Key::F11,
        glutin::VirtualKeyCode::F12 => Key::F12,
        glutin::VirtualKeyCode::F13 => Key::F13,
        glutin::VirtualKeyCode::F14 => Key::F14,
        glutin::VirtualKeyCode::F15 => Key::F15,
        // K::F16 => Key::F16,
        // K::F17 => Key::F17,
        // K::F18 => Key::F18,
        // K::F19 => Key::F19,
        // K::F20 => Key::F20,
        // K::F21 => Key::F21,
        // K::F22 => Key::F22,
        // K::F23 => Key::F23,
        // K::F24 => Key::F24,
        // Possibly next code.
        // K::F25 => Key::Unknown,
        glutin::VirtualKeyCode::Numpad0 => Key::NumPad0,
        glutin::VirtualKeyCode::Numpad1 => Key::NumPad1,
        glutin::VirtualKeyCode::Numpad2 => Key::NumPad2,
        glutin::VirtualKeyCode::Numpad3 => Key::NumPad3,
        glutin::VirtualKeyCode::Numpad4 => Key::NumPad4,
        glutin::VirtualKeyCode::Numpad5 => Key::NumPad5,
        glutin::VirtualKeyCode::Numpad6 => Key::NumPad6,
        glutin::VirtualKeyCode::Numpad7 => Key::NumPad7,
        glutin::VirtualKeyCode::Numpad8 => Key::NumPad8,
        glutin::VirtualKeyCode::Numpad9 => Key::NumPad9,
        glutin::VirtualKeyCode::NumpadComma => Key::NumPadDecimal,
        glutin::VirtualKeyCode::Divide => Key::NumPadDivide,
        glutin::VirtualKeyCode::Multiply => Key::NumPadMultiply,
        glutin::VirtualKeyCode::Subtract => Key::NumPadMinus,
        glutin::VirtualKeyCode::Add => Key::NumPadPlus,
        glutin::VirtualKeyCode::NumpadEnter => Key::NumPadEnter,
        glutin::VirtualKeyCode::NumpadEquals => Key::NumPadEquals,
        glutin::VirtualKeyCode::LShift => Key::LShift,
        glutin::VirtualKeyCode::LControl => Key::LCtrl,
        glutin::VirtualKeyCode::LAlt => Key::LAlt,
        glutin::VirtualKeyCode::LMenu => Key::LGui,
        glutin::VirtualKeyCode::RShift => Key::RShift,
        glutin::VirtualKeyCode::RControl => Key::RCtrl,
        glutin::VirtualKeyCode::RAlt => Key::RAlt,
        glutin::VirtualKeyCode::RMenu => Key::RGui,
        // Map to backslash?
        // K::GraveAccent => Key::Unknown,
        glutin::VirtualKeyCode::Home => Key::Home,
        glutin::VirtualKeyCode::Insert => Key::Insert,
        glutin::VirtualKeyCode::Left => Key::Left,
        glutin::VirtualKeyCode::LBracket => Key::LeftBracket,
        // K::Menu => Key::Menu,
        glutin::VirtualKeyCode::Minus => Key::Minus,
        glutin::VirtualKeyCode::Numlock => Key::NumLockClear,
        glutin::VirtualKeyCode::PageDown => Key::PageDown,
        glutin::VirtualKeyCode::PageUp => Key::PageUp,
        glutin::VirtualKeyCode::Pause => Key::Pause,
        glutin::VirtualKeyCode::Period => Key::Period,
        // K::PrintScreen => Key::PrintScreen,
        glutin::VirtualKeyCode::Right => Key::Right,
        glutin::VirtualKeyCode::RBracket => Key::RightBracket,
        // K::ScrollLock => Key::ScrollLock,
        glutin::VirtualKeyCode::Semicolon => Key::Semicolon,
        glutin::VirtualKeyCode::Slash => Key::Slash,
        glutin::VirtualKeyCode::Space => Key::Space,
        glutin::VirtualKeyCode::Tab => Key::Tab,
        glutin::VirtualKeyCode::Up => Key::Up,
        // K::World1 => Key::Unknown,
        // K::World2 => Key::Unknown,
        _ => Key::Unknown,
    }
}

/// Maps Glutin's mouse button to Piston's mouse button.
pub fn map_mouse(mouse_button: glutin::MouseButton) -> input::MouseButton {
    use input::MouseButton;
    match mouse_button {
        glutin::MouseButton::Left => MouseButton::Left,
        glutin::MouseButton::Right => MouseButton::Right,
        glutin::MouseButton::Middle => MouseButton::Middle,
        glutin::MouseButton::Other(0) => MouseButton::X1,
        glutin::MouseButton::Other(1) => MouseButton::X2,
        glutin::MouseButton::Other(2) => MouseButton::Button6,
        glutin::MouseButton::Other(3) => MouseButton::Button7,
        glutin::MouseButton::Other(4) => MouseButton::Button8,
        _ => MouseButton::Unknown
    }
}
