//! A function for converting a `winit::Event` to a `conrod::event::Input`.

extern crate winit;

use Scalar;
use event::Input;
use input;
use cursor;
#[cfg(feature = "glium")] use glium;


/// Types that have access to a `winit::Window` and can provide the necessary dimensions and hidpi
/// factor for converting `winit::Event`s to `conrod::event::Input`, as well as set the mouse
/// cursor.
///
/// This allows users to pass either `glium::Display`, `glium::glutin::Window` or `winit::Window`
/// to the `conrod::backend::winit::convert` function defined below.
pub trait WinitWindow {
    /// Return the inner size of the window in pixels.
    fn get_inner_size(&self) -> Option<(u32, u32)>;
    /// Return the window's DPI factor so that we can convert from pixel values to scalar values.
    fn hidpi_factor(&self) -> f32;
}

impl WinitWindow for winit::Window {
    fn get_inner_size(&self) -> Option<(u32, u32)> {
        winit::Window::get_inner_size(self)
    }
    fn hidpi_factor(&self) -> f32 {
        winit::Window::hidpi_factor(self)
    }
}

#[cfg(feature = "glium")]
impl WinitWindow for glium::Display {
    fn get_inner_size(&self) -> Option<(u32, u32)> {
        self.gl_window().get_inner_size()
    }
    fn hidpi_factor(&self) -> f32 {
        self.gl_window().hidpi_factor()
    }
}


/// A function for converting a `winit::Event` to a `conrod::event::Input`.
///
/// This can be useful for single-window applications.
pub fn convert_event<W>(e: winit::Event, window: &W) -> Option<Input>
    where W: WinitWindow,
{
    match e {
        winit::Event::WindowEvent { event, .. } => convert_window_event(event, window),
        _ => None,
    }
}

/// A function for converting a `winit::WindowEvent` to a `conrod::event::Input`.
///
/// This is useful for multi-window applications.
pub fn convert_window_event<W>(e: winit::WindowEvent, window: &W) -> Option<Input>
    where W: WinitWindow,
{
    // The "dots per inch" factor. Multiplying this by `win_w` and `win_h` gives the framebuffer
    // width and height.
    let dpi_factor = window.hidpi_factor() as Scalar;

    // The window size in points.
    let (win_w, win_h) = match window.get_inner_size() {
        Some((w, h)) => (w as Scalar / dpi_factor, h as Scalar / dpi_factor),
        None => return None,
    };

    // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
    //
    // winit produces input events in pixels, so these positions need to be divided by the widht
    // and height of the window in order to be DPI agnostic.
    let tx = |x: Scalar| (x / dpi_factor) - win_w / 2.0;
    let ty = |y: Scalar| -((y / dpi_factor) - win_h / 2.0);

    match e {

        winit::WindowEvent::Resized(w, h) => {
            let w = (w as Scalar / dpi_factor) as u32;
            let h = (h as Scalar / dpi_factor) as u32;
            Some(Input::Resize(w, h).into())
        },

        winit::WindowEvent::ReceivedCharacter(ch) => {
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

        winit::WindowEvent::Focused(focused) =>
            Some(Input::Focus(focused).into()),

        winit::WindowEvent::KeyboardInput { input, .. } => {
            input.virtual_keycode.map(|key| {
                match input.state {
                    winit::ElementState::Pressed =>
                        Input::Press(input::Button::Keyboard(map_key(key))).into(),
                    winit::ElementState::Released =>
                        Input::Release(input::Button::Keyboard(map_key(key))).into(),
                }
            })
        },

        winit::WindowEvent::Touch(winit::Touch { phase, location: (x, y), id, .. }) => {
            let phase = match phase {
                winit::TouchPhase::Started => input::touch::Phase::Start,
                winit::TouchPhase::Moved => input::touch::Phase::Move,
                winit::TouchPhase::Cancelled => input::touch::Phase::Cancel,
                winit::TouchPhase::Ended => input::touch::Phase::End,
            };
            let xy = [tx(x), ty(y)];
            let id = input::touch::Id::new(id);
            let touch = input::Touch { phase: phase, id: id, xy: xy };
            Some(Input::Touch(touch).into())
        }

        winit::WindowEvent::CursorMoved { position: (x, y), .. } => {
            let x = tx(x as Scalar);
            let y = ty(y as Scalar);
            let motion = input::Motion::MouseCursor { x: x, y: y };
            Some(Input::Motion(motion).into())
        },

        winit::WindowEvent::MouseWheel { delta, .. } => match delta {

            winit::MouseScrollDelta::PixelDelta(x, y) => {
                let x = x as Scalar / dpi_factor;
                let y = -y as Scalar / dpi_factor;
                let motion = input::Motion::Scroll { x: x, y: y };
                Some(Input::Motion(motion).into())
            },

            winit::MouseScrollDelta::LineDelta(x, y) => {
                // This should be configurable (we should provide a LineDelta event to allow for this).
                const ARBITRARY_POINTS_PER_LINE_FACTOR: Scalar = 10.0;
                let x = ARBITRARY_POINTS_PER_LINE_FACTOR * x as Scalar;
                let y = ARBITRARY_POINTS_PER_LINE_FACTOR * -y as Scalar;
                Some(Input::Motion(input::Motion::Scroll { x: x, y: y }).into())
            },
        },

        winit::WindowEvent::MouseInput { state, button, .. } => match state {
            winit::ElementState::Pressed =>
                Some(Input::Press(input::Button::Mouse(map_mouse(button))).into()),
            winit::ElementState::Released =>
                Some(Input::Release(input::Button::Mouse(map_mouse(button))).into()),
        },

        winit::WindowEvent::Refresh => {
            Some(Input::Redraw)
        },

        _ => None,
    }
}

/// Maps winit's key to a conrod `Key`.
pub fn map_key(keycode: winit::VirtualKeyCode) -> input::keyboard::Key {
    use input::keyboard::Key;

    match keycode {
        winit::VirtualKeyCode::Key0 => Key::D0,
        winit::VirtualKeyCode::Key1 => Key::D1,
        winit::VirtualKeyCode::Key2 => Key::D2,
        winit::VirtualKeyCode::Key3 => Key::D3,
        winit::VirtualKeyCode::Key4 => Key::D4,
        winit::VirtualKeyCode::Key5 => Key::D5,
        winit::VirtualKeyCode::Key6 => Key::D6,
        winit::VirtualKeyCode::Key7 => Key::D7,
        winit::VirtualKeyCode::Key8 => Key::D8,
        winit::VirtualKeyCode::Key9 => Key::D9,
        winit::VirtualKeyCode::A => Key::A,
        winit::VirtualKeyCode::B => Key::B,
        winit::VirtualKeyCode::C => Key::C,
        winit::VirtualKeyCode::D => Key::D,
        winit::VirtualKeyCode::E => Key::E,
        winit::VirtualKeyCode::F => Key::F,
        winit::VirtualKeyCode::G => Key::G,
        winit::VirtualKeyCode::H => Key::H,
        winit::VirtualKeyCode::I => Key::I,
        winit::VirtualKeyCode::J => Key::J,
        winit::VirtualKeyCode::K => Key::K,
        winit::VirtualKeyCode::L => Key::L,
        winit::VirtualKeyCode::M => Key::M,
        winit::VirtualKeyCode::N => Key::N,
        winit::VirtualKeyCode::O => Key::O,
        winit::VirtualKeyCode::P => Key::P,
        winit::VirtualKeyCode::Q => Key::Q,
        winit::VirtualKeyCode::R => Key::R,
        winit::VirtualKeyCode::S => Key::S,
        winit::VirtualKeyCode::T => Key::T,
        winit::VirtualKeyCode::U => Key::U,
        winit::VirtualKeyCode::V => Key::V,
        winit::VirtualKeyCode::W => Key::W,
        winit::VirtualKeyCode::X => Key::X,
        winit::VirtualKeyCode::Y => Key::Y,
        winit::VirtualKeyCode::Z => Key::Z,
        winit::VirtualKeyCode::Apostrophe => Key::Unknown,
        winit::VirtualKeyCode::Backslash => Key::Backslash,
        winit::VirtualKeyCode::Back => Key::Backspace,
        // K::CapsLock => Key::CapsLock,
        winit::VirtualKeyCode::Delete => Key::Delete,
        winit::VirtualKeyCode::Comma => Key::Comma,
        winit::VirtualKeyCode::Down => Key::Down,
        winit::VirtualKeyCode::End => Key::End,
        winit::VirtualKeyCode::Return => Key::Return,
        winit::VirtualKeyCode::Equals => Key::Equals,
        winit::VirtualKeyCode::Escape => Key::Escape,
        winit::VirtualKeyCode::F1 => Key::F1,
        winit::VirtualKeyCode::F2 => Key::F2,
        winit::VirtualKeyCode::F3 => Key::F3,
        winit::VirtualKeyCode::F4 => Key::F4,
        winit::VirtualKeyCode::F5 => Key::F5,
        winit::VirtualKeyCode::F6 => Key::F6,
        winit::VirtualKeyCode::F7 => Key::F7,
        winit::VirtualKeyCode::F8 => Key::F8,
        winit::VirtualKeyCode::F9 => Key::F9,
        winit::VirtualKeyCode::F10 => Key::F10,
        winit::VirtualKeyCode::F11 => Key::F11,
        winit::VirtualKeyCode::F12 => Key::F12,
        winit::VirtualKeyCode::F13 => Key::F13,
        winit::VirtualKeyCode::F14 => Key::F14,
        winit::VirtualKeyCode::F15 => Key::F15,
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
        winit::VirtualKeyCode::Numpad0 => Key::NumPad0,
        winit::VirtualKeyCode::Numpad1 => Key::NumPad1,
        winit::VirtualKeyCode::Numpad2 => Key::NumPad2,
        winit::VirtualKeyCode::Numpad3 => Key::NumPad3,
        winit::VirtualKeyCode::Numpad4 => Key::NumPad4,
        winit::VirtualKeyCode::Numpad5 => Key::NumPad5,
        winit::VirtualKeyCode::Numpad6 => Key::NumPad6,
        winit::VirtualKeyCode::Numpad7 => Key::NumPad7,
        winit::VirtualKeyCode::Numpad8 => Key::NumPad8,
        winit::VirtualKeyCode::Numpad9 => Key::NumPad9,
        winit::VirtualKeyCode::NumpadComma => Key::NumPadDecimal,
        winit::VirtualKeyCode::Divide => Key::NumPadDivide,
        winit::VirtualKeyCode::Multiply => Key::NumPadMultiply,
        winit::VirtualKeyCode::Subtract => Key::NumPadMinus,
        winit::VirtualKeyCode::Add => Key::NumPadPlus,
        winit::VirtualKeyCode::NumpadEnter => Key::NumPadEnter,
        winit::VirtualKeyCode::NumpadEquals => Key::NumPadEquals,
        winit::VirtualKeyCode::LShift => Key::LShift,
        winit::VirtualKeyCode::LControl => Key::LCtrl,
        winit::VirtualKeyCode::LAlt => Key::LAlt,
        winit::VirtualKeyCode::LMenu => Key::LGui,
        winit::VirtualKeyCode::RShift => Key::RShift,
        winit::VirtualKeyCode::RControl => Key::RCtrl,
        winit::VirtualKeyCode::RAlt => Key::RAlt,
        winit::VirtualKeyCode::RMenu => Key::RGui,
        // Map to backslash?
        // K::GraveAccent => Key::Unknown,
        winit::VirtualKeyCode::Home => Key::Home,
        winit::VirtualKeyCode::Insert => Key::Insert,
        winit::VirtualKeyCode::Left => Key::Left,
        winit::VirtualKeyCode::LBracket => Key::LeftBracket,
        // K::Menu => Key::Menu,
        winit::VirtualKeyCode::Minus => Key::Minus,
        winit::VirtualKeyCode::Numlock => Key::NumLockClear,
        winit::VirtualKeyCode::PageDown => Key::PageDown,
        winit::VirtualKeyCode::PageUp => Key::PageUp,
        winit::VirtualKeyCode::Pause => Key::Pause,
        winit::VirtualKeyCode::Period => Key::Period,
        // K::PrintScreen => Key::PrintScreen,
        winit::VirtualKeyCode::Right => Key::Right,
        winit::VirtualKeyCode::RBracket => Key::RightBracket,
        // K::ScrollLock => Key::ScrollLock,
        winit::VirtualKeyCode::Semicolon => Key::Semicolon,
        winit::VirtualKeyCode::Slash => Key::Slash,
        winit::VirtualKeyCode::Space => Key::Space,
        winit::VirtualKeyCode::Tab => Key::Tab,
        winit::VirtualKeyCode::Up => Key::Up,
        // K::World1 => Key::Unknown,
        // K::World2 => Key::Unknown,
        _ => Key::Unknown,
    }
}

/// Maps winit's mouse button to conrod's mouse button.
pub fn map_mouse(mouse_button: winit::MouseButton) -> input::MouseButton {
    use input::MouseButton;
    match mouse_button {
        winit::MouseButton::Left => MouseButton::Left,
        winit::MouseButton::Right => MouseButton::Right,
        winit::MouseButton::Middle => MouseButton::Middle,
        winit::MouseButton::Other(0) => MouseButton::X1,
        winit::MouseButton::Other(1) => MouseButton::X2,
        winit::MouseButton::Other(2) => MouseButton::Button6,
        winit::MouseButton::Other(3) => MouseButton::Button7,
        winit::MouseButton::Other(4) => MouseButton::Button8,
        _ => MouseButton::Unknown
    }
}

/// Convert a given conrod mouse cursor to the corresponding winit cursor type.
pub fn convert_mouse_cursor(cursor: cursor::MouseCursor) -> winit::MouseCursor {
    match cursor {
        cursor::MouseCursor::Text => winit::MouseCursor::Text,
        cursor::MouseCursor::VerticalText => winit::MouseCursor::VerticalText,
        cursor::MouseCursor::Hand => winit::MouseCursor::Hand,
        cursor::MouseCursor::Grab => winit::MouseCursor::Grab,
        cursor::MouseCursor::Grabbing => winit::MouseCursor::Grabbing,
        cursor::MouseCursor::ResizeVertical => winit::MouseCursor::NsResize,
        cursor::MouseCursor::ResizeHorizontal => winit::MouseCursor::EwResize,
        cursor::MouseCursor::ResizeTopLeftBottomRight => winit::MouseCursor::NwseResize,
        cursor::MouseCursor::ResizeTopRightBottomLeft => winit::MouseCursor::NeswResize,
        _ => winit::MouseCursor::Arrow,
    }
}
