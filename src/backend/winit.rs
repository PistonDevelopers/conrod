//! A function for converting a `winit::WindowEvent` or a `glium::glutin::Event`
//! to a `conrod::event::Input`.

extern crate winit;

use Scalar;
use event::Input;
use input;
#[cfg(feature = "glium")] use glium;


/// Types that have access to a `winit::Window` and can provide the necessary dimensions and hidpi
/// factor for converting `winit::WindowEvent`s to `conrod::event::Input`.
///
/// This allows users to pass either `glium::Display`, `glium::glutin::Window` or `winit::Window`
/// to the `conrod::backend::winit::convert` function defined below.
pub trait WinitWindow {
    /// Return the inner size of the window.
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
impl WinitWindow for glium::glutin::Window {
    fn get_inner_size(&self) -> Option<(u32, u32)> {
        glium::glutin::Window::as_winit_window(self).get_inner_size()
    }
    fn hidpi_factor(&self) -> f32 {
        glium::glutin::Window::as_winit_window(self).hidpi_factor()
    }
}

#[cfg(feature = "glium")]
impl WinitWindow for glium::Display {
    fn get_inner_size(&self) -> Option<(u32, u32)> {
        self.get_window().and_then(|window| window.get_inner_size())
    }
    fn hidpi_factor(&self) -> f32 {
        match self.get_window() {
            Some(window) => window.hidpi_factor(),
            None => 1.0,
        }
    }
}

/// A trait to unify the new `EventsLoop` API of `winit 0.8.*`, and the old polling iterator
/// API of winit `0.7.*`.
pub trait WinitEvent {
    /// Convert the event into a `conrod::event::Input`.
    fn convert<W: WinitWindow>(self, window: &W) -> Option<Input>;
}

impl WinitEvent for winit::WindowEvent {
    fn convert<W: WinitWindow>(self, window: &W) -> Option<Input> {
        // The window size in points.
        let (win_w, win_h) = match window.get_inner_size() {
            Some((w, h)) => (w as Scalar, h as Scalar),
            None => return None,
        };

        // The "dots per inch" factor. Multiplying this by `win_w` and `win_h` gives the
        // framebuffer width and height.
        let dpi_factor = window.hidpi_factor() as Scalar;

        // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
        //
        // winit produces input events in pixels, so these positions need to be divided by the
        // width and height of the window in order to be DPI agnostic.
        let tx = |x: Scalar| (x / dpi_factor) - win_w / 2.0;
        let ty = |y: Scalar| -((y / dpi_factor) - win_h / 2.0);

        match self {

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

            winit::WindowEvent::KeyboardInput(winit::ElementState::Pressed, _, Some(key), _) =>
                Some(Input::Press(input::Button::Keyboard(map_key(key))).into()),

            winit::WindowEvent::KeyboardInput(winit::ElementState::Released, _, Some(key), _) =>
                Some(Input::Release(input::Button::Keyboard(map_key(key))).into()),

            winit::WindowEvent::Touch(winit::Touch { phase, location: (x, y), id }) => {
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

            winit::WindowEvent::MouseMoved(x, y) => {
                let x = tx(x as Scalar);
                let y = ty(y as Scalar);
                let motion = input::Motion::MouseCursor { x: x, y: y };
                Some(Input::Motion(motion).into())
            },

            winit::WindowEvent::MouseWheel(winit::MouseScrollDelta::PixelDelta(x, y), _) => {
                let x = x as Scalar / dpi_factor;
                let y = -y as Scalar / dpi_factor;
                let motion = input::Motion::Scroll { x: x, y: y };
                Some(Input::Motion(motion).into())
            },

            winit::WindowEvent::MouseWheel(winit::MouseScrollDelta::LineDelta(x, y), _) => {
                // This should be configurable
                // (we should provide a LineDelta event to allow for this).
                const ARBITRARY_POINTS_PER_LINE_FACTOR: Scalar = 10.0;
                let x = ARBITRARY_POINTS_PER_LINE_FACTOR * x as Scalar;
                let y = ARBITRARY_POINTS_PER_LINE_FACTOR * -y as Scalar;
                Some(Input::Motion(input::Motion::Scroll { x: x, y: y }).into())
            },

            winit::WindowEvent::MouseInput(winit::ElementState::Pressed, button) =>
                Some(Input::Press(input::Button::Mouse(map_mouse(button))).into()),

            winit::WindowEvent::MouseInput(winit::ElementState::Released, button) =>
                Some(Input::Release(input::Button::Mouse(map_mouse(button))).into()),

            _ => None,
        }
    }
}

#[cfg(feature = "glium")]
impl WinitEvent for glium::glutin::Event {
    fn convert<W: WinitWindow>(self, window: &W) -> Option<Input> {
        // The window size in points.
        let (win_w, win_h) = match window.get_inner_size() {
            Some((w, h)) => (w as Scalar, h as Scalar),
            None => return None,
        };

        // The "dots per inch" factor. Multiplying this by `win_w` and `win_h` gives the
        // framebuffer width and height.
        let dpi_factor = window.hidpi_factor() as Scalar;

        // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
        //
        // winit produces input events in pixels, so these positions need to be divided by the width
        // and height of the window in order to be DPI agnostic.
        let tx = |x: Scalar| (x / dpi_factor) - win_w / 2.0;
        let ty = |y: Scalar| -((y / dpi_factor) - win_h / 2.0);

        match self {
            glium::glutin::Event::Resized(w, h) => {
                let w = (w as Scalar / dpi_factor) as u32;
                let h = (h as Scalar / dpi_factor) as u32;
                Some(Input::Resize(w, h).into())
            },

            glium::glutin::Event::ReceivedCharacter(ch) => {
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
     
             glium::glutin::Event::Focused(focused) =>
                 Some(Input::Focus(focused).into()),
     
             glium::glutin::Event::KeyboardInput(glium::glutin::ElementState::Pressed, _, Some(key)) =>
                 Some(Input::Press(input::Button::Keyboard(map_key(key.to_winit()))).into()),
     
             glium::glutin::Event::KeyboardInput(glium::glutin::ElementState::Released, _, Some(key)) =>
                 Some(Input::Release(input::Button::Keyboard(map_key(key.to_winit()))).into()),
     
             glium::glutin::Event::Touch(glium::glutin::Touch { phase, location: (x, y), id }) => {
                 let phase = match phase {
                     glium::glutin::TouchPhase::Started => input::touch::Phase::Start,
                     glium::glutin::TouchPhase::Moved => input::touch::Phase::Move,
                     glium::glutin::TouchPhase::Cancelled => input::touch::Phase::Cancel,
                     glium::glutin::TouchPhase::Ended => input::touch::Phase::End,
                 };
                 let xy = [tx(x), ty(y)];
                 let id = input::touch::Id::new(id);
                 let touch = input::Touch { phase: phase, id: id, xy: xy };
                 Some(Input::Touch(touch).into())
             }

             glium::glutin::Event::MouseMoved(x, y) => {
                 let x = tx(x as Scalar);
                 let y = ty(y as Scalar);
                 let motion = input::Motion::MouseCursor { x: x, y: y };
                 Some(Input::Motion(motion).into())
             },

            glium::glutin::Event::MouseWheel(glium::glutin::MouseScrollDelta::PixelDelta(x, y), _) => {
                let x = x as Scalar / dpi_factor;
                let y = -y as Scalar / dpi_factor;
                let motion = input::Motion::Scroll { x: x, y: y };
                Some(Input::Motion(motion).into())
            },

            glium::glutin::Event::MouseWheel(glium::glutin::MouseScrollDelta::LineDelta(x, y), _) => {
                // This should be configurable
                // (we should provide a LineDelta event to allow for this).
                const ARBITRARY_POINTS_PER_LINE_FACTOR: Scalar = 10.0;
                let x = ARBITRARY_POINTS_PER_LINE_FACTOR * x as Scalar;
                let y = ARBITRARY_POINTS_PER_LINE_FACTOR * -y as Scalar;
                Some(Input::Motion(input::Motion::Scroll { x: x, y: y }).into())
            },

            glium::glutin::Event::MouseInput(glium::glutin::ElementState::Pressed, button) =>
                Some(Input::Press(input::Button::Mouse(map_mouse(button.to_winit()))).into()),

            glium::glutin::Event::MouseInput(glium::glutin::ElementState::Released, button) =>
                Some(Input::Release(input::Button::Mouse(map_mouse(button.to_winit()))).into()),

            _ => None,
        }

    }
}

/// A function for converting a `winit::Event` to a `conrod::event::Input`.
pub fn convert<E, W>(event: E, window: &W) -> Option<Input>
    where E: WinitEvent,
          W: WinitWindow,
{
    event.convert(window)
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

#[cfg(feature = "glium")]
trait ToWinit {
    type WinitValue;
    fn to_winit(self) -> Self::WinitValue;
}

#[cfg(feature = "glium")]
impl ToWinit for glium::glutin::MouseButton {
    type WinitValue = winit::MouseButton;
    fn to_winit(self) -> Self::WinitValue {
        match self {
            glium::glutin::MouseButton::Left => winit::MouseButton::Left,
            glium::glutin::MouseButton::Right => winit::MouseButton::Right,
            glium::glutin::MouseButton::Middle => winit::MouseButton::Middle,
            glium::glutin::MouseButton::Other(b) => winit::MouseButton::Other(b),
        }
    }
}

#[cfg(feature = "glium")]
impl ToWinit for glium::glutin::VirtualKeyCode {
    type WinitValue = winit::VirtualKeyCode;
    fn to_winit(self) -> Self::WinitValue {
        match self {
            glium::glutin::VirtualKeyCode::Key1 => winit::VirtualKeyCode::Key1,
            glium::glutin::VirtualKeyCode::Key2 => winit::VirtualKeyCode::Key2,
            glium::glutin::VirtualKeyCode::Key3 => winit::VirtualKeyCode::Key3,
            glium::glutin::VirtualKeyCode::Key4 => winit::VirtualKeyCode::Key4,
            glium::glutin::VirtualKeyCode::Key5 => winit::VirtualKeyCode::Key5,
            glium::glutin::VirtualKeyCode::Key6 => winit::VirtualKeyCode::Key6,
            glium::glutin::VirtualKeyCode::Key7 => winit::VirtualKeyCode::Key7,
            glium::glutin::VirtualKeyCode::Key8 => winit::VirtualKeyCode::Key8,
            glium::glutin::VirtualKeyCode::Key9 => winit::VirtualKeyCode::Key9,
            glium::glutin::VirtualKeyCode::Key0 => winit::VirtualKeyCode::Key0,
            glium::glutin::VirtualKeyCode::A => winit::VirtualKeyCode::A,
            glium::glutin::VirtualKeyCode::B => winit::VirtualKeyCode::B,
            glium::glutin::VirtualKeyCode::C => winit::VirtualKeyCode::C,
            glium::glutin::VirtualKeyCode::D => winit::VirtualKeyCode::D,
            glium::glutin::VirtualKeyCode::E => winit::VirtualKeyCode::E,
            glium::glutin::VirtualKeyCode::F => winit::VirtualKeyCode::F,
            glium::glutin::VirtualKeyCode::G => winit::VirtualKeyCode::G,
            glium::glutin::VirtualKeyCode::H => winit::VirtualKeyCode::H,
            glium::glutin::VirtualKeyCode::I => winit::VirtualKeyCode::I,
            glium::glutin::VirtualKeyCode::J => winit::VirtualKeyCode::J,
            glium::glutin::VirtualKeyCode::K => winit::VirtualKeyCode::K,
            glium::glutin::VirtualKeyCode::L => winit::VirtualKeyCode::L,
            glium::glutin::VirtualKeyCode::M => winit::VirtualKeyCode::M,
            glium::glutin::VirtualKeyCode::N => winit::VirtualKeyCode::N,
            glium::glutin::VirtualKeyCode::O => winit::VirtualKeyCode::O,
            glium::glutin::VirtualKeyCode::P => winit::VirtualKeyCode::P,
            glium::glutin::VirtualKeyCode::Q => winit::VirtualKeyCode::Q,
            glium::glutin::VirtualKeyCode::R => winit::VirtualKeyCode::R,
            glium::glutin::VirtualKeyCode::S => winit::VirtualKeyCode::S,
            glium::glutin::VirtualKeyCode::T => winit::VirtualKeyCode::T,
            glium::glutin::VirtualKeyCode::U => winit::VirtualKeyCode::U,
            glium::glutin::VirtualKeyCode::V => winit::VirtualKeyCode::V,
            glium::glutin::VirtualKeyCode::W => winit::VirtualKeyCode::W,
            glium::glutin::VirtualKeyCode::X => winit::VirtualKeyCode::X,
            glium::glutin::VirtualKeyCode::Y => winit::VirtualKeyCode::Y,
            glium::glutin::VirtualKeyCode::Z => winit::VirtualKeyCode::Z,
            glium::glutin::VirtualKeyCode::Escape => winit::VirtualKeyCode::Escape,
            glium::glutin::VirtualKeyCode::F1 => winit::VirtualKeyCode::F1,
            glium::glutin::VirtualKeyCode::F2 => winit::VirtualKeyCode::F2,
            glium::glutin::VirtualKeyCode::F3 => winit::VirtualKeyCode::F3,
            glium::glutin::VirtualKeyCode::F4 => winit::VirtualKeyCode::F4,
            glium::glutin::VirtualKeyCode::F5 => winit::VirtualKeyCode::F5,
            glium::glutin::VirtualKeyCode::F6 => winit::VirtualKeyCode::F6,
            glium::glutin::VirtualKeyCode::F7 => winit::VirtualKeyCode::F7,
            glium::glutin::VirtualKeyCode::F8 => winit::VirtualKeyCode::F8,
            glium::glutin::VirtualKeyCode::F9 => winit::VirtualKeyCode::F9,
            glium::glutin::VirtualKeyCode::F10 => winit::VirtualKeyCode::F10,
            glium::glutin::VirtualKeyCode::F11 => winit::VirtualKeyCode::F11,
            glium::glutin::VirtualKeyCode::F12 => winit::VirtualKeyCode::F12,
            glium::glutin::VirtualKeyCode::F13 => winit::VirtualKeyCode::F13,
            glium::glutin::VirtualKeyCode::F14 => winit::VirtualKeyCode::F14,
            glium::glutin::VirtualKeyCode::F15 => winit::VirtualKeyCode::F15,
            glium::glutin::VirtualKeyCode::Snapshot => winit::VirtualKeyCode::Snapshot,
            glium::glutin::VirtualKeyCode::Scroll => winit::VirtualKeyCode::Scroll,
            glium::glutin::VirtualKeyCode::Pause => winit::VirtualKeyCode::Pause,
            glium::glutin::VirtualKeyCode::Insert => winit::VirtualKeyCode::Insert,
            glium::glutin::VirtualKeyCode::Home => winit::VirtualKeyCode::Home,
            glium::glutin::VirtualKeyCode::Delete => winit::VirtualKeyCode::Delete,
            glium::glutin::VirtualKeyCode::End => winit::VirtualKeyCode::End,
            glium::glutin::VirtualKeyCode::PageDown => winit::VirtualKeyCode::PageDown,
            glium::glutin::VirtualKeyCode::PageUp => winit::VirtualKeyCode::PageUp,
            glium::glutin::VirtualKeyCode::Left => winit::VirtualKeyCode::Left,
            glium::glutin::VirtualKeyCode::Up => winit::VirtualKeyCode::Up,
            glium::glutin::VirtualKeyCode::Right => winit::VirtualKeyCode::Right,
            glium::glutin::VirtualKeyCode::Down => winit::VirtualKeyCode::Down,
            glium::glutin::VirtualKeyCode::Back => winit::VirtualKeyCode::Back,
            glium::glutin::VirtualKeyCode::Return => winit::VirtualKeyCode::Return,
            glium::glutin::VirtualKeyCode::Space => winit::VirtualKeyCode::Space,
            glium::glutin::VirtualKeyCode::Compose => winit::VirtualKeyCode::Compose,
            glium::glutin::VirtualKeyCode::Numlock => winit::VirtualKeyCode::Numlock,
            glium::glutin::VirtualKeyCode::Numpad0 => winit::VirtualKeyCode::Numpad0,
            glium::glutin::VirtualKeyCode::Numpad1 => winit::VirtualKeyCode::Numpad1,
            glium::glutin::VirtualKeyCode::Numpad2 => winit::VirtualKeyCode::Numpad2,
            glium::glutin::VirtualKeyCode::Numpad3 => winit::VirtualKeyCode::Numpad3,
            glium::glutin::VirtualKeyCode::Numpad4 => winit::VirtualKeyCode::Numpad4,
            glium::glutin::VirtualKeyCode::Numpad5 => winit::VirtualKeyCode::Numpad5,
            glium::glutin::VirtualKeyCode::Numpad6 => winit::VirtualKeyCode::Numpad6,
            glium::glutin::VirtualKeyCode::Numpad7 => winit::VirtualKeyCode::Numpad7,
            glium::glutin::VirtualKeyCode::Numpad8 => winit::VirtualKeyCode::Numpad8,
            glium::glutin::VirtualKeyCode::Numpad9 => winit::VirtualKeyCode::Numpad9,
            glium::glutin::VirtualKeyCode::AbntC1 => winit::VirtualKeyCode::AbntC1,
            glium::glutin::VirtualKeyCode::AbntC2 => winit::VirtualKeyCode::AbntC2,
            glium::glutin::VirtualKeyCode::Add => winit::VirtualKeyCode::Add,
            glium::glutin::VirtualKeyCode::Apostrophe => winit::VirtualKeyCode::Apostrophe,
            glium::glutin::VirtualKeyCode::Apps => winit::VirtualKeyCode::Apps,
            glium::glutin::VirtualKeyCode::At => winit::VirtualKeyCode::At,
            glium::glutin::VirtualKeyCode::Ax => winit::VirtualKeyCode::Ax,
            glium::glutin::VirtualKeyCode::Backslash => winit::VirtualKeyCode::Backslash,
            glium::glutin::VirtualKeyCode::Calculator => winit::VirtualKeyCode::Calculator,
            glium::glutin::VirtualKeyCode::Capital => winit::VirtualKeyCode::Capital,
            glium::glutin::VirtualKeyCode::Colon => winit::VirtualKeyCode::Colon,
            glium::glutin::VirtualKeyCode::Comma => winit::VirtualKeyCode::Comma,
            glium::glutin::VirtualKeyCode::Convert => winit::VirtualKeyCode::Convert,
            glium::glutin::VirtualKeyCode::Decimal => winit::VirtualKeyCode::Decimal,
            glium::glutin::VirtualKeyCode::Divide => winit::VirtualKeyCode::Divide,
            glium::glutin::VirtualKeyCode::Equals => winit::VirtualKeyCode::Equals,
            glium::glutin::VirtualKeyCode::Grave => winit::VirtualKeyCode::Grave,
            glium::glutin::VirtualKeyCode::Kana => winit::VirtualKeyCode::Kana,
            glium::glutin::VirtualKeyCode::Kanji => winit::VirtualKeyCode::Kanji,
            glium::glutin::VirtualKeyCode::LAlt => winit::VirtualKeyCode::LAlt,
            glium::glutin::VirtualKeyCode::LBracket => winit::VirtualKeyCode::LBracket,
            glium::glutin::VirtualKeyCode::LControl => winit::VirtualKeyCode::LControl,
            glium::glutin::VirtualKeyCode::LMenu => winit::VirtualKeyCode::LMenu,
            glium::glutin::VirtualKeyCode::LShift => winit::VirtualKeyCode::LShift,
            glium::glutin::VirtualKeyCode::LWin => winit::VirtualKeyCode::LWin,
            glium::glutin::VirtualKeyCode::Mail => winit::VirtualKeyCode::Mail,
            glium::glutin::VirtualKeyCode::MediaSelect => winit::VirtualKeyCode::MediaSelect,
            glium::glutin::VirtualKeyCode::MediaStop => winit::VirtualKeyCode::MediaStop,
            glium::glutin::VirtualKeyCode::Minus => winit::VirtualKeyCode::Minus,
            glium::glutin::VirtualKeyCode::Multiply => winit::VirtualKeyCode::Multiply,
            glium::glutin::VirtualKeyCode::Mute => winit::VirtualKeyCode::Mute,
            glium::glutin::VirtualKeyCode::MyComputer => winit::VirtualKeyCode::MyComputer,
            glium::glutin::VirtualKeyCode::NavigateForward => winit::VirtualKeyCode::NavigateForward,
            glium::glutin::VirtualKeyCode::NavigateBackward => winit::VirtualKeyCode::NavigateBackward,
            glium::glutin::VirtualKeyCode::NextTrack => winit::VirtualKeyCode::NextTrack,
            glium::glutin::VirtualKeyCode::NoConvert => winit::VirtualKeyCode::NoConvert,
            glium::glutin::VirtualKeyCode::NumpadComma => winit::VirtualKeyCode::NumpadComma,
            glium::glutin::VirtualKeyCode::NumpadEnter => winit::VirtualKeyCode::NumpadEnter,
            glium::glutin::VirtualKeyCode::NumpadEquals => winit::VirtualKeyCode::NumpadEquals,
            glium::glutin::VirtualKeyCode::OEM102 => winit::VirtualKeyCode::OEM102,
            glium::glutin::VirtualKeyCode::Period => winit::VirtualKeyCode::Period,
            glium::glutin::VirtualKeyCode::PlayPause => winit::VirtualKeyCode::PlayPause,
            glium::glutin::VirtualKeyCode::Power => winit::VirtualKeyCode::Power,
            glium::glutin::VirtualKeyCode::PrevTrack => winit::VirtualKeyCode::PrevTrack,
            glium::glutin::VirtualKeyCode::RAlt => winit::VirtualKeyCode::RAlt,
            glium::glutin::VirtualKeyCode::RBracket => winit::VirtualKeyCode::RBracket,
            glium::glutin::VirtualKeyCode::RControl => winit::VirtualKeyCode::RControl,
            glium::glutin::VirtualKeyCode::RMenu => winit::VirtualKeyCode::RMenu,
            glium::glutin::VirtualKeyCode::RShift => winit::VirtualKeyCode::RShift,
            glium::glutin::VirtualKeyCode::RWin => winit::VirtualKeyCode::RWin,
            glium::glutin::VirtualKeyCode::Semicolon => winit::VirtualKeyCode::Semicolon,
            glium::glutin::VirtualKeyCode::Slash => winit::VirtualKeyCode::Slash,
            glium::glutin::VirtualKeyCode::Sleep => winit::VirtualKeyCode::Sleep,
            glium::glutin::VirtualKeyCode::Stop => winit::VirtualKeyCode::Stop,
            glium::glutin::VirtualKeyCode::Subtract => winit::VirtualKeyCode::Subtract,
            glium::glutin::VirtualKeyCode::Sysrq => winit::VirtualKeyCode::Sysrq,
            glium::glutin::VirtualKeyCode::Tab => winit::VirtualKeyCode::Tab,
            glium::glutin::VirtualKeyCode::Underline => winit::VirtualKeyCode::Underline,
            glium::glutin::VirtualKeyCode::Unlabeled => winit::VirtualKeyCode::Unlabeled,
            glium::glutin::VirtualKeyCode::VolumeDown => winit::VirtualKeyCode::VolumeDown,
            glium::glutin::VirtualKeyCode::VolumeUp => winit::VirtualKeyCode::VolumeUp,
            glium::glutin::VirtualKeyCode::Wake => winit::VirtualKeyCode::Wake,
            glium::glutin::VirtualKeyCode::WebBack => winit::VirtualKeyCode::WebBack,
            glium::glutin::VirtualKeyCode::WebFavorites => winit::VirtualKeyCode::WebFavorites,
            glium::glutin::VirtualKeyCode::WebForward => winit::VirtualKeyCode::WebForward,
            glium::glutin::VirtualKeyCode::WebHome => winit::VirtualKeyCode::WebHome,
            glium::glutin::VirtualKeyCode::WebRefresh => winit::VirtualKeyCode::WebRefresh,
            glium::glutin::VirtualKeyCode::WebSearch => winit::VirtualKeyCode::WebSearch,
            glium::glutin::VirtualKeyCode::WebStop => winit::VirtualKeyCode::WebStop,
            glium::glutin::VirtualKeyCode::Yen => winit::VirtualKeyCode::Yen,
        }
    }
}

