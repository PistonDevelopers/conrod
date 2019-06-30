//! A collection of macros for generating code related to winit+conrod interop.
//!
//! The reason we provide macros and don't implement functions using the `winit` crate directly is
//! that conrod has many backends that use `winit`, often with differing versions. By providing
//! macros, we allow these backends to generate code necessary for whatever version of winit they
//! are currently using. This means we don't have to wait for all of the backend winit dependencies
//! to synchronise before we can publish new conrod releases.

/// Maps winit's key to a conrod `Key`.
///
/// Expects a `winit::VirtualKeyCode` as input and returns a `conrod_core::input::keyboard::Key`.
///
/// Requires that both the `winit` and `conrod_core` crates exist within the crate root.
#[macro_export]
macro_rules! convert_key {
    ($keycode:expr) => {{
        match $keycode {
            winit::VirtualKeyCode::Key0 => conrod_core::input::keyboard::Key::D0,
            winit::VirtualKeyCode::Key1 => conrod_core::input::keyboard::Key::D1,
            winit::VirtualKeyCode::Key2 => conrod_core::input::keyboard::Key::D2,
            winit::VirtualKeyCode::Key3 => conrod_core::input::keyboard::Key::D3,
            winit::VirtualKeyCode::Key4 => conrod_core::input::keyboard::Key::D4,
            winit::VirtualKeyCode::Key5 => conrod_core::input::keyboard::Key::D5,
            winit::VirtualKeyCode::Key6 => conrod_core::input::keyboard::Key::D6,
            winit::VirtualKeyCode::Key7 => conrod_core::input::keyboard::Key::D7,
            winit::VirtualKeyCode::Key8 => conrod_core::input::keyboard::Key::D8,
            winit::VirtualKeyCode::Key9 => conrod_core::input::keyboard::Key::D9,
            winit::VirtualKeyCode::A => conrod_core::input::keyboard::Key::A,
            winit::VirtualKeyCode::B => conrod_core::input::keyboard::Key::B,
            winit::VirtualKeyCode::C => conrod_core::input::keyboard::Key::C,
            winit::VirtualKeyCode::D => conrod_core::input::keyboard::Key::D,
            winit::VirtualKeyCode::E => conrod_core::input::keyboard::Key::E,
            winit::VirtualKeyCode::F => conrod_core::input::keyboard::Key::F,
            winit::VirtualKeyCode::G => conrod_core::input::keyboard::Key::G,
            winit::VirtualKeyCode::H => conrod_core::input::keyboard::Key::H,
            winit::VirtualKeyCode::I => conrod_core::input::keyboard::Key::I,
            winit::VirtualKeyCode::J => conrod_core::input::keyboard::Key::J,
            winit::VirtualKeyCode::K => conrod_core::input::keyboard::Key::K,
            winit::VirtualKeyCode::L => conrod_core::input::keyboard::Key::L,
            winit::VirtualKeyCode::M => conrod_core::input::keyboard::Key::M,
            winit::VirtualKeyCode::N => conrod_core::input::keyboard::Key::N,
            winit::VirtualKeyCode::O => conrod_core::input::keyboard::Key::O,
            winit::VirtualKeyCode::P => conrod_core::input::keyboard::Key::P,
            winit::VirtualKeyCode::Q => conrod_core::input::keyboard::Key::Q,
            winit::VirtualKeyCode::R => conrod_core::input::keyboard::Key::R,
            winit::VirtualKeyCode::S => conrod_core::input::keyboard::Key::S,
            winit::VirtualKeyCode::T => conrod_core::input::keyboard::Key::T,
            winit::VirtualKeyCode::U => conrod_core::input::keyboard::Key::U,
            winit::VirtualKeyCode::V => conrod_core::input::keyboard::Key::V,
            winit::VirtualKeyCode::W => conrod_core::input::keyboard::Key::W,
            winit::VirtualKeyCode::X => conrod_core::input::keyboard::Key::X,
            winit::VirtualKeyCode::Y => conrod_core::input::keyboard::Key::Y,
            winit::VirtualKeyCode::Z => conrod_core::input::keyboard::Key::Z,
            winit::VirtualKeyCode::Apostrophe => conrod_core::input::keyboard::Key::Unknown,
            winit::VirtualKeyCode::Backslash => conrod_core::input::keyboard::Key::Backslash,
            winit::VirtualKeyCode::Back => conrod_core::input::keyboard::Key::Backspace,
            // K::CapsLock => Key::CapsLock,
            winit::VirtualKeyCode::Delete => conrod_core::input::keyboard::Key::Delete,
            winit::VirtualKeyCode::Comma => conrod_core::input::keyboard::Key::Comma,
            winit::VirtualKeyCode::Down => conrod_core::input::keyboard::Key::Down,
            winit::VirtualKeyCode::End => conrod_core::input::keyboard::Key::End,
            winit::VirtualKeyCode::Return => conrod_core::input::keyboard::Key::Return,
            winit::VirtualKeyCode::Equals => conrod_core::input::keyboard::Key::Equals,
            winit::VirtualKeyCode::Escape => conrod_core::input::keyboard::Key::Escape,
            winit::VirtualKeyCode::F1 => conrod_core::input::keyboard::Key::F1,
            winit::VirtualKeyCode::F2 => conrod_core::input::keyboard::Key::F2,
            winit::VirtualKeyCode::F3 => conrod_core::input::keyboard::Key::F3,
            winit::VirtualKeyCode::F4 => conrod_core::input::keyboard::Key::F4,
            winit::VirtualKeyCode::F5 => conrod_core::input::keyboard::Key::F5,
            winit::VirtualKeyCode::F6 => conrod_core::input::keyboard::Key::F6,
            winit::VirtualKeyCode::F7 => conrod_core::input::keyboard::Key::F7,
            winit::VirtualKeyCode::F8 => conrod_core::input::keyboard::Key::F8,
            winit::VirtualKeyCode::F9 => conrod_core::input::keyboard::Key::F9,
            winit::VirtualKeyCode::F10 => conrod_core::input::keyboard::Key::F10,
            winit::VirtualKeyCode::F11 => conrod_core::input::keyboard::Key::F11,
            winit::VirtualKeyCode::F12 => conrod_core::input::keyboard::Key::F12,
            winit::VirtualKeyCode::F13 => conrod_core::input::keyboard::Key::F13,
            winit::VirtualKeyCode::F14 => conrod_core::input::keyboard::Key::F14,
            winit::VirtualKeyCode::F15 => conrod_core::input::keyboard::Key::F15,
            winit::VirtualKeyCode::Numpad0 => conrod_core::input::keyboard::Key::NumPad0,
            winit::VirtualKeyCode::Numpad1 => conrod_core::input::keyboard::Key::NumPad1,
            winit::VirtualKeyCode::Numpad2 => conrod_core::input::keyboard::Key::NumPad2,
            winit::VirtualKeyCode::Numpad3 => conrod_core::input::keyboard::Key::NumPad3,
            winit::VirtualKeyCode::Numpad4 => conrod_core::input::keyboard::Key::NumPad4,
            winit::VirtualKeyCode::Numpad5 => conrod_core::input::keyboard::Key::NumPad5,
            winit::VirtualKeyCode::Numpad6 => conrod_core::input::keyboard::Key::NumPad6,
            winit::VirtualKeyCode::Numpad7 => conrod_core::input::keyboard::Key::NumPad7,
            winit::VirtualKeyCode::Numpad8 => conrod_core::input::keyboard::Key::NumPad8,
            winit::VirtualKeyCode::Numpad9 => conrod_core::input::keyboard::Key::NumPad9,
            winit::VirtualKeyCode::NumpadComma => conrod_core::input::keyboard::Key::NumPadDecimal,
            winit::VirtualKeyCode::Divide => conrod_core::input::keyboard::Key::NumPadDivide,
            winit::VirtualKeyCode::Multiply => conrod_core::input::keyboard::Key::NumPadMultiply,
            winit::VirtualKeyCode::Subtract => conrod_core::input::keyboard::Key::NumPadMinus,
            winit::VirtualKeyCode::Add => conrod_core::input::keyboard::Key::NumPadPlus,
            winit::VirtualKeyCode::NumpadEnter => conrod_core::input::keyboard::Key::NumPadEnter,
            winit::VirtualKeyCode::NumpadEquals => conrod_core::input::keyboard::Key::NumPadEquals,
            winit::VirtualKeyCode::LShift => conrod_core::input::keyboard::Key::LShift,
            winit::VirtualKeyCode::LControl => conrod_core::input::keyboard::Key::LCtrl,
            winit::VirtualKeyCode::LAlt => conrod_core::input::keyboard::Key::LAlt,
            winit::VirtualKeyCode::RShift => conrod_core::input::keyboard::Key::RShift,
            winit::VirtualKeyCode::RControl => conrod_core::input::keyboard::Key::RCtrl,
            winit::VirtualKeyCode::RAlt => conrod_core::input::keyboard::Key::RAlt,
            winit::VirtualKeyCode::Home => conrod_core::input::keyboard::Key::Home,
            winit::VirtualKeyCode::Insert => conrod_core::input::keyboard::Key::Insert,
            winit::VirtualKeyCode::Left => conrod_core::input::keyboard::Key::Left,
            winit::VirtualKeyCode::LBracket => conrod_core::input::keyboard::Key::LeftBracket,
            winit::VirtualKeyCode::Minus => conrod_core::input::keyboard::Key::Minus,
            winit::VirtualKeyCode::Numlock => conrod_core::input::keyboard::Key::NumLockClear,
            winit::VirtualKeyCode::PageDown => conrod_core::input::keyboard::Key::PageDown,
            winit::VirtualKeyCode::PageUp => conrod_core::input::keyboard::Key::PageUp,
            winit::VirtualKeyCode::Pause => conrod_core::input::keyboard::Key::Pause,
            winit::VirtualKeyCode::Period => conrod_core::input::keyboard::Key::Period,
            winit::VirtualKeyCode::Right => conrod_core::input::keyboard::Key::Right,
            winit::VirtualKeyCode::RBracket => conrod_core::input::keyboard::Key::RightBracket,
            winit::VirtualKeyCode::Semicolon => conrod_core::input::keyboard::Key::Semicolon,
            winit::VirtualKeyCode::Slash => conrod_core::input::keyboard::Key::Slash,
            winit::VirtualKeyCode::Space => conrod_core::input::keyboard::Key::Space,
            winit::VirtualKeyCode::Tab => conrod_core::input::keyboard::Key::Tab,
            winit::VirtualKeyCode::Up => conrod_core::input::keyboard::Key::Up,
            _ => conrod_core::input::keyboard::Key::Unknown,
        }
    }};
}

/// Maps winit's mouse button to conrod's mouse button.
///
/// Expects a `winit::MouseButton` as input and returns a `conrod_core::input::MouseButton` as
/// output.
///
/// Requires that both the `conrod_core` and `winit` crates are in the crate root.
#[macro_export]
macro_rules! convert_mouse_button {
    ($mouse_button:expr) => {{
        match $mouse_button {
            winit::MouseButton::Left => conrod_core::input::MouseButton::Left,
            winit::MouseButton::Right => conrod_core::input::MouseButton::Right,
            winit::MouseButton::Middle => conrod_core::input::MouseButton::Middle,
            winit::MouseButton::Other(0) => conrod_core::input::MouseButton::X1,
            winit::MouseButton::Other(1) => conrod_core::input::MouseButton::X2,
            winit::MouseButton::Other(2) => conrod_core::input::MouseButton::Button6,
            winit::MouseButton::Other(3) => conrod_core::input::MouseButton::Button7,
            winit::MouseButton::Other(4) => conrod_core::input::MouseButton::Button8,
            _ => conrod_core::input::MouseButton::Unknown,
        }
    }};
}

/// A macro for converting a `winit::WindowEvent` to a `Option<conrod_core::event::Input>`.
///
/// Expects a `winit::WindowEvent` and a reference to a window implementing `WinitWindow`.
/// Returns an `Option<conrod_core::event::Input>`.
#[macro_export]
macro_rules! convert_window_event {
    ($event:expr, $window:expr) => {{
        // The window size in points.
        let (win_w, win_h) = match $window.get_inner_size() {
            Some((w, h)) => (w as conrod_core::Scalar, h as conrod_core::Scalar),
            None => return None,
        };

        // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
        let tx = |x: conrod_core::Scalar| x - win_w / 2.0;
        let ty = |y: conrod_core::Scalar| -(y - win_h / 2.0);

        // Functions for converting keys and mouse buttons.
        let map_key = |key| convert_key!(key);
        let map_mouse = |button| convert_mouse_button!(button);

        match $event {
            winit::WindowEvent::Resized(winit::dpi::LogicalSize { width, height }) => {
                Some(conrod_core::event::Input::Resize(width as _, height as _).into())
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
                Some(conrod_core::event::Input::Text(string).into())
            },

            winit::WindowEvent::Focused(focused) =>
                Some(conrod_core::event::Input::Focus(focused).into()),

            winit::WindowEvent::KeyboardInput { input, .. } => {
                input.virtual_keycode.map(|key| {
                    match input.state {
                        winit::ElementState::Pressed =>
                            conrod_core::event::Input::Press(conrod_core::input::Button::Keyboard(map_key(key))).into(),
                        winit::ElementState::Released =>
                            conrod_core::event::Input::Release(conrod_core::input::Button::Keyboard(map_key(key))).into(),
                    }
                })
            },

            winit::WindowEvent::Touch(winit::Touch { phase, location, id, .. }) => {
                let winit::dpi::LogicalPosition { x, y } = location;
                let phase = match phase {
                    winit::TouchPhase::Started => conrod_core::input::touch::Phase::Start,
                    winit::TouchPhase::Moved => conrod_core::input::touch::Phase::Move,
                    winit::TouchPhase::Cancelled => conrod_core::input::touch::Phase::Cancel,
                    winit::TouchPhase::Ended => conrod_core::input::touch::Phase::End,
                };
                let xy = [tx(x), ty(y)];
                let id = conrod_core::input::touch::Id::new(id);
                let touch = conrod_core::input::Touch { phase: phase, id: id, xy: xy };
                Some(conrod_core::event::Input::Touch(touch).into())
            }

            winit::WindowEvent::CursorMoved { position, .. } => {
                let winit::dpi::LogicalPosition { x, y } = position;
                let x = tx(x as conrod_core::Scalar);
                let y = ty(y as conrod_core::Scalar);
                let motion = conrod_core::input::Motion::MouseCursor { x: x, y: y };
                Some(conrod_core::event::Input::Motion(motion).into())
            },

            winit::WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::MouseScrollDelta::PixelDelta(winit::dpi::LogicalPosition { x, y }) => {
                    let x = x as conrod_core::Scalar;
                    let y = -y as conrod_core::Scalar;
                    let motion = conrod_core::input::Motion::Scroll { x: x, y: y };
                    Some(conrod_core::event::Input::Motion(motion).into())
                },

                winit::MouseScrollDelta::LineDelta(x, y) => {
                    // This should be configurable (we should provide a LineDelta event to allow for this).
                    const ARBITRARY_POINTS_PER_LINE_FACTOR: conrod_core::Scalar = 10.0;
                    let x = ARBITRARY_POINTS_PER_LINE_FACTOR * x as conrod_core::Scalar;
                    let y = ARBITRARY_POINTS_PER_LINE_FACTOR * -y as conrod_core::Scalar;
                    Some(conrod_core::event::Input::Motion(conrod_core::input::Motion::Scroll { x: x, y: y }).into())
                },
            },

            winit::WindowEvent::MouseInput { state, button, .. } => match state {
                winit::ElementState::Pressed =>
                    Some(conrod_core::event::Input::Press(conrod_core::input::Button::Mouse(map_mouse(button))).into()),
                winit::ElementState::Released =>
                    Some(conrod_core::event::Input::Release(conrod_core::input::Button::Mouse(map_mouse(button))).into()),
            },

            winit::WindowEvent::Refresh => {
                Some(conrod_core::event::Input::Redraw)
            },

            _ => None,
        }
    }};
}

/// A macro for converting a `winit::Event` to a `conrod_core::event::Input`.
///
/// Expects a `winit::Event` and a reference to a window implementing `WinitWindow`.
/// Returns an `Option<conrod_core::event::Input>`.
///
/// Invocations of this macro require that a version of the `winit` and `conrod_core` crates are
/// available in the crate root.
#[macro_export]
macro_rules! convert_event {
    ($event:expr, $window:expr) => {{
        match $event {
            winit::Event::WindowEvent { event, .. } => convert_window_event!(event, $window),
            _ => None,
        }
    }};
}

/// Convert a given conrod mouse cursor to the corresponding winit cursor type.
///
/// Expects a `conrod_core::cursor::MouseCursor`, returns a `winit::MouseCursor`.
///
/// Requires that both the `conrod_core` and `winit` crates are in the crate root.
#[macro_export]
macro_rules! convert_mouse_cursor {
    ($cursor:expr) => {{
        match $cursor {
            conrod_core::cursor::MouseCursor::Text => winit::MouseCursor::Text,
            conrod_core::cursor::MouseCursor::VerticalText => winit::MouseCursor::VerticalText,
            conrod_core::cursor::MouseCursor::Hand => winit::MouseCursor::Hand,
            conrod_core::cursor::MouseCursor::Grab => winit::MouseCursor::Grab,
            conrod_core::cursor::MouseCursor::Grabbing => winit::MouseCursor::Grabbing,
            conrod_core::cursor::MouseCursor::ResizeVertical => winit::MouseCursor::NsResize,
            conrod_core::cursor::MouseCursor::ResizeHorizontal => winit::MouseCursor::EwResize,
            conrod_core::cursor::MouseCursor::ResizeTopLeftBottomRight => winit::MouseCursor::NwseResize,
            conrod_core::cursor::MouseCursor::ResizeTopRightBottomLeft => winit::MouseCursor::NeswResize,
            _ => winit::MouseCursor::Arrow,
        }
    }};
}

/// Generate a set of conversion functions for converting between types of the crate's versions of
/// `winit` and `conrod_core`.
#[macro_export]
macro_rules! conversion_fns {
    () => {
        /// Convert a `winit::VirtualKeyCode` to a `conrod_core::input::keyboard::Key`.
        pub fn convert_key(keycode: winit::VirtualKeyCode) -> conrod_core::input::keyboard::Key {
            convert_key!(keycode)
        }

        /// Convert a `winit::MouseButton` to a `conrod_core::input::MouseButton`.
        pub fn convert_mouse_button(
            mouse_button: winit::MouseButton,
        ) -> conrod_core::input::MouseButton {
            convert_mouse_button!(mouse_button)
        }

        /// Convert a given conrod mouse cursor to the corresponding winit cursor type.
        pub fn convert_mouse_cursor(
            cursor: conrod_core::cursor::MouseCursor,
        ) -> winit::MouseCursor {
            convert_mouse_cursor!(cursor)
        }

        /// A function for converting a `winit::WindowEvent` to a `conrod_core::event::Input`.
        pub fn convert_window_event<W>(
            event: winit::WindowEvent,
            window: &W,
        ) -> Option<conrod_core::event::Input>
        where
            W: $crate::WinitWindow,
        {
            convert_window_event!(event, window)
        }

        /// A function for converting a `winit::Event` to a `conrod_core::event::Input`.
        pub fn convert_event<W>(
            event: winit::Event,
            window: &W,
        ) -> Option<conrod_core::event::Input>
        where
            W: $crate::WinitWindow,
        {
            convert_event!(event, window)
        }
    };
}
