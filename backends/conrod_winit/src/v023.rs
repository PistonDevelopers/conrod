#[macro_export]
macro_rules! v023_convert_key {
    ($keycode:expr) => {{
        match $keycode {
            winit::event::VirtualKeyCode::Key0 => conrod_core::input::keyboard::Key::D0,
            winit::event::VirtualKeyCode::Key1 => conrod_core::input::keyboard::Key::D1,
            winit::event::VirtualKeyCode::Key2 => conrod_core::input::keyboard::Key::D2,
            winit::event::VirtualKeyCode::Key3 => conrod_core::input::keyboard::Key::D3,
            winit::event::VirtualKeyCode::Key4 => conrod_core::input::keyboard::Key::D4,
            winit::event::VirtualKeyCode::Key5 => conrod_core::input::keyboard::Key::D5,
            winit::event::VirtualKeyCode::Key6 => conrod_core::input::keyboard::Key::D6,
            winit::event::VirtualKeyCode::Key7 => conrod_core::input::keyboard::Key::D7,
            winit::event::VirtualKeyCode::Key8 => conrod_core::input::keyboard::Key::D8,
            winit::event::VirtualKeyCode::Key9 => conrod_core::input::keyboard::Key::D9,
            winit::event::VirtualKeyCode::A => conrod_core::input::keyboard::Key::A,
            winit::event::VirtualKeyCode::B => conrod_core::input::keyboard::Key::B,
            winit::event::VirtualKeyCode::C => conrod_core::input::keyboard::Key::C,
            winit::event::VirtualKeyCode::D => conrod_core::input::keyboard::Key::D,
            winit::event::VirtualKeyCode::E => conrod_core::input::keyboard::Key::E,
            winit::event::VirtualKeyCode::F => conrod_core::input::keyboard::Key::F,
            winit::event::VirtualKeyCode::G => conrod_core::input::keyboard::Key::G,
            winit::event::VirtualKeyCode::H => conrod_core::input::keyboard::Key::H,
            winit::event::VirtualKeyCode::I => conrod_core::input::keyboard::Key::I,
            winit::event::VirtualKeyCode::J => conrod_core::input::keyboard::Key::J,
            winit::event::VirtualKeyCode::K => conrod_core::input::keyboard::Key::K,
            winit::event::VirtualKeyCode::L => conrod_core::input::keyboard::Key::L,
            winit::event::VirtualKeyCode::M => conrod_core::input::keyboard::Key::M,
            winit::event::VirtualKeyCode::N => conrod_core::input::keyboard::Key::N,
            winit::event::VirtualKeyCode::O => conrod_core::input::keyboard::Key::O,
            winit::event::VirtualKeyCode::P => conrod_core::input::keyboard::Key::P,
            winit::event::VirtualKeyCode::Q => conrod_core::input::keyboard::Key::Q,
            winit::event::VirtualKeyCode::R => conrod_core::input::keyboard::Key::R,
            winit::event::VirtualKeyCode::S => conrod_core::input::keyboard::Key::S,
            winit::event::VirtualKeyCode::T => conrod_core::input::keyboard::Key::T,
            winit::event::VirtualKeyCode::U => conrod_core::input::keyboard::Key::U,
            winit::event::VirtualKeyCode::V => conrod_core::input::keyboard::Key::V,
            winit::event::VirtualKeyCode::W => conrod_core::input::keyboard::Key::W,
            winit::event::VirtualKeyCode::X => conrod_core::input::keyboard::Key::X,
            winit::event::VirtualKeyCode::Y => conrod_core::input::keyboard::Key::Y,
            winit::event::VirtualKeyCode::Z => conrod_core::input::keyboard::Key::Z,
            winit::event::VirtualKeyCode::Apostrophe => conrod_core::input::keyboard::Key::Unknown,
            winit::event::VirtualKeyCode::Backslash => conrod_core::input::keyboard::Key::Backslash,
            winit::event::VirtualKeyCode::Back => conrod_core::input::keyboard::Key::Backspace,
            // K::CapsLock => Key::CapsLock,
            winit::event::VirtualKeyCode::Delete => conrod_core::input::keyboard::Key::Delete,
            winit::event::VirtualKeyCode::Comma => conrod_core::input::keyboard::Key::Comma,
            winit::event::VirtualKeyCode::Down => conrod_core::input::keyboard::Key::Down,
            winit::event::VirtualKeyCode::End => conrod_core::input::keyboard::Key::End,
            winit::event::VirtualKeyCode::Return => conrod_core::input::keyboard::Key::Return,
            winit::event::VirtualKeyCode::Equals => conrod_core::input::keyboard::Key::Equals,
            winit::event::VirtualKeyCode::Escape => conrod_core::input::keyboard::Key::Escape,
            winit::event::VirtualKeyCode::F1 => conrod_core::input::keyboard::Key::F1,
            winit::event::VirtualKeyCode::F2 => conrod_core::input::keyboard::Key::F2,
            winit::event::VirtualKeyCode::F3 => conrod_core::input::keyboard::Key::F3,
            winit::event::VirtualKeyCode::F4 => conrod_core::input::keyboard::Key::F4,
            winit::event::VirtualKeyCode::F5 => conrod_core::input::keyboard::Key::F5,
            winit::event::VirtualKeyCode::F6 => conrod_core::input::keyboard::Key::F6,
            winit::event::VirtualKeyCode::F7 => conrod_core::input::keyboard::Key::F7,
            winit::event::VirtualKeyCode::F8 => conrod_core::input::keyboard::Key::F8,
            winit::event::VirtualKeyCode::F9 => conrod_core::input::keyboard::Key::F9,
            winit::event::VirtualKeyCode::F10 => conrod_core::input::keyboard::Key::F10,
            winit::event::VirtualKeyCode::F11 => conrod_core::input::keyboard::Key::F11,
            winit::event::VirtualKeyCode::F12 => conrod_core::input::keyboard::Key::F12,
            winit::event::VirtualKeyCode::F13 => conrod_core::input::keyboard::Key::F13,
            winit::event::VirtualKeyCode::F14 => conrod_core::input::keyboard::Key::F14,
            winit::event::VirtualKeyCode::F15 => conrod_core::input::keyboard::Key::F15,
            winit::event::VirtualKeyCode::Numpad0 => conrod_core::input::keyboard::Key::NumPad0,
            winit::event::VirtualKeyCode::Numpad1 => conrod_core::input::keyboard::Key::NumPad1,
            winit::event::VirtualKeyCode::Numpad2 => conrod_core::input::keyboard::Key::NumPad2,
            winit::event::VirtualKeyCode::Numpad3 => conrod_core::input::keyboard::Key::NumPad3,
            winit::event::VirtualKeyCode::Numpad4 => conrod_core::input::keyboard::Key::NumPad4,
            winit::event::VirtualKeyCode::Numpad5 => conrod_core::input::keyboard::Key::NumPad5,
            winit::event::VirtualKeyCode::Numpad6 => conrod_core::input::keyboard::Key::NumPad6,
            winit::event::VirtualKeyCode::Numpad7 => conrod_core::input::keyboard::Key::NumPad7,
            winit::event::VirtualKeyCode::Numpad8 => conrod_core::input::keyboard::Key::NumPad8,
            winit::event::VirtualKeyCode::Numpad9 => conrod_core::input::keyboard::Key::NumPad9,
            winit::event::VirtualKeyCode::NumpadComma
            | winit::event::VirtualKeyCode::NumpadDecimal => {
                conrod_core::input::keyboard::Key::NumPadDecimal
            }
            winit::event::VirtualKeyCode::NumpadDivide => {
                conrod_core::input::keyboard::Key::NumPadDivide
            }
            winit::event::VirtualKeyCode::NumpadMultiply => {
                conrod_core::input::keyboard::Key::NumPadMultiply
            }
            winit::event::VirtualKeyCode::NumpadSubtract => {
                conrod_core::input::keyboard::Key::NumPadMinus
            }
            winit::event::VirtualKeyCode::NumpadAdd => {
                conrod_core::input::keyboard::Key::NumPadPlus
            }
            winit::event::VirtualKeyCode::NumpadEnter => {
                conrod_core::input::keyboard::Key::NumPadEnter
            }
            winit::event::VirtualKeyCode::NumpadEquals => {
                conrod_core::input::keyboard::Key::NumPadEquals
            }
            winit::event::VirtualKeyCode::LShift => conrod_core::input::keyboard::Key::LShift,
            winit::event::VirtualKeyCode::LControl => conrod_core::input::keyboard::Key::LCtrl,
            winit::event::VirtualKeyCode::LAlt => conrod_core::input::keyboard::Key::LAlt,
            winit::event::VirtualKeyCode::RShift => conrod_core::input::keyboard::Key::RShift,
            winit::event::VirtualKeyCode::RControl => conrod_core::input::keyboard::Key::RCtrl,
            winit::event::VirtualKeyCode::RAlt => conrod_core::input::keyboard::Key::RAlt,
            winit::event::VirtualKeyCode::Home => conrod_core::input::keyboard::Key::Home,
            winit::event::VirtualKeyCode::Insert => conrod_core::input::keyboard::Key::Insert,
            winit::event::VirtualKeyCode::Left => conrod_core::input::keyboard::Key::Left,
            winit::event::VirtualKeyCode::LBracket => {
                conrod_core::input::keyboard::Key::LeftBracket
            }
            winit::event::VirtualKeyCode::Minus => conrod_core::input::keyboard::Key::Minus,
            winit::event::VirtualKeyCode::Numlock => {
                conrod_core::input::keyboard::Key::NumLockClear
            }
            winit::event::VirtualKeyCode::PageDown => conrod_core::input::keyboard::Key::PageDown,
            winit::event::VirtualKeyCode::PageUp => conrod_core::input::keyboard::Key::PageUp,
            winit::event::VirtualKeyCode::Pause => conrod_core::input::keyboard::Key::Pause,
            winit::event::VirtualKeyCode::Period => conrod_core::input::keyboard::Key::Period,
            winit::event::VirtualKeyCode::Right => conrod_core::input::keyboard::Key::Right,
            winit::event::VirtualKeyCode::RBracket => {
                conrod_core::input::keyboard::Key::RightBracket
            }
            winit::event::VirtualKeyCode::Semicolon => conrod_core::input::keyboard::Key::Semicolon,
            winit::event::VirtualKeyCode::Slash => conrod_core::input::keyboard::Key::Slash,
            winit::event::VirtualKeyCode::Space => conrod_core::input::keyboard::Key::Space,
            winit::event::VirtualKeyCode::Tab => conrod_core::input::keyboard::Key::Tab,
            winit::event::VirtualKeyCode::Up => conrod_core::input::keyboard::Key::Up,
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
macro_rules! v023_convert_mouse_button {
    ($mouse_button:expr) => {{
        $crate::v021_convert_mouse_button!($mouse_button)
    }};
}

/// A macro for converting a `winit::WindowEvent` to a `Option<conrod_core::event::Input>`.
///
/// Expects a `winit::WindowEvent` and a reference to a window implementing `WinitWindow`.
/// Returns an `Option<conrod_core::event::Input>`.
#[macro_export]
macro_rules! v023_convert_window_event {
    ($event:expr, $window:expr) => {{
        // The window size in points.
        let scale_factor: f64 = $window.scale_factor();
        let (win_w, win_h): (f64, f64) = $window.inner_size().to_logical::<f64>(scale_factor).into();

        // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
        let tx = |x: conrod_core::Scalar| x - win_w / 2.0;
        let ty = |y: conrod_core::Scalar| -(y - win_h / 2.0);

        // Functions for converting keys and mouse buttons.
        let map_key = |key: winit::event::VirtualKeyCode| $crate::v023_convert_key!(key);
        let map_mouse = |button: winit::event::MouseButton| $crate::v023_convert_mouse_button!(button);

        match $event {
            winit::event::WindowEvent::Resized(physical_size) => {
                let winit::dpi::LogicalSize { width, height } = physical_size.to_logical(scale_factor);
                Some(conrod_core::event::Input::Resize(width, height).into())
            },

            winit::event::WindowEvent::ReceivedCharacter(ch) => {
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

            winit::event::WindowEvent::Focused(focused) =>
                Some(conrod_core::event::Input::Focus(focused.clone()).into()),

            winit::event::WindowEvent::KeyboardInput { input, .. } => {
                input.virtual_keycode.map(|key| {
                    match input.state {
                        winit::event::ElementState::Pressed =>
                            conrod_core::event::Input::Press(conrod_core::input::Button::Keyboard(map_key(key))).into(),
                        winit::event::ElementState::Released =>
                            conrod_core::event::Input::Release(conrod_core::input::Button::Keyboard(map_key(key))).into(),
                    }
                })
            },

            winit::event::WindowEvent::Touch(winit::event::Touch { phase, location, id, .. }) => {
                let winit::dpi::LogicalPosition { x, y } = location.to_logical::<f64>(scale_factor);
                let phase = match phase {
                    winit::event::TouchPhase::Started => conrod_core::input::touch::Phase::Start,
                    winit::event::TouchPhase::Moved => conrod_core::input::touch::Phase::Move,
                    winit::event::TouchPhase::Cancelled => conrod_core::input::touch::Phase::Cancel,
                    winit::event::TouchPhase::Ended => conrod_core::input::touch::Phase::End,
                };
                let xy = [tx(x), ty(y)];
                let id = conrod_core::input::touch::Id::new(id.clone());
                let touch = conrod_core::input::Touch { phase: phase, id: id, xy: xy };
                Some(conrod_core::event::Input::Touch(touch).into())
            }

            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let winit::dpi::LogicalPosition { x, y } = position.to_logical::<f64>(scale_factor);
                let x = tx(x as conrod_core::Scalar);
                let y = ty(y as conrod_core::Scalar);
                let motion = conrod_core::input::Motion::MouseCursor { x: x, y: y };
                Some(conrod_core::event::Input::Motion(motion).into())
            },

            winit::event::WindowEvent::MouseWheel { delta, .. } => match delta {
                winit::event::MouseScrollDelta::PixelDelta(delta) => {
                    let winit::dpi::LogicalPosition { x, y } = delta.to_logical::<f64>(scale_factor);
                    let x = x as conrod_core::Scalar;
                    let y = -y as conrod_core::Scalar;
                    let motion = conrod_core::input::Motion::Scroll { x: x, y: y };
                    Some(conrod_core::event::Input::Motion(motion).into())
                },

                winit::event::MouseScrollDelta::LineDelta(x, y) => {
                    // This should be configurable (we should provide a LineDelta event to allow for this).
                    const ARBITRARY_POINTS_PER_LINE_FACTOR: conrod_core::Scalar = 10.0;
                    let x = ARBITRARY_POINTS_PER_LINE_FACTOR * x.clone() as conrod_core::Scalar;
                    let y = ARBITRARY_POINTS_PER_LINE_FACTOR * -y.clone() as conrod_core::Scalar;
                    Some(conrod_core::event::Input::Motion(conrod_core::input::Motion::Scroll { x: x, y: y }).into())
                },
            },

            winit::event::WindowEvent::MouseInput { state, button, .. } => match state {
                winit::event::ElementState::Pressed =>
                    Some(conrod_core::event::Input::Press(conrod_core::input::Button::Mouse(map_mouse(button.clone()))).into()),
                winit::event::ElementState::Released =>
                    Some(conrod_core::event::Input::Release(conrod_core::input::Button::Mouse(map_mouse(button.clone()))).into()),
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
macro_rules! v023_convert_event {
    ($event:expr, $window:expr) => {{
        match $event {
            winit::event::Event::WindowEvent { event, .. } => {
                $crate::v023_convert_window_event!(event, $window)
            }
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
macro_rules! v023_convert_mouse_cursor {
    ($cursor:expr) => {{
        $crate::v021_convert_mouse_cursor!($cursor)
    }};
}

#[macro_export]
macro_rules! v023_conversion_fns {
    () => {
        /// Generate a set of conversion functions for converting between types of the crate's versions of
        /// `winit` and `conrod_core`.
        /// Maps winit's key to a conrod `Key`.
        ///
        /// Expects a `winit::VirtualKeyCode` as input and returns a `conrod_core::input::keyboard::Key`.
        ///
        /// Requires that both the `winit` and `conrod_core` crates exist within the crate root.
        pub fn convert_key(
            keycode: winit::event::VirtualKeyCode,
        ) -> conrod_core::input::keyboard::Key {
            $crate::v023_convert_key!(keycode)
        }

        /// Convert a `winit::MouseButton` to a `conrod_core::input::MouseButton`.
        pub fn convert_mouse_button(
            mouse_button: winit::event::MouseButton,
        ) -> conrod_core::input::MouseButton {
            $crate::v023_convert_mouse_button!(mouse_button)
        }

        /// Convert a given conrod mouse cursor to the corresponding winit cursor type.
        pub fn convert_mouse_cursor(
            cursor: conrod_core::cursor::MouseCursor,
        ) -> winit::window::CursorIcon {
            $crate::v023_convert_mouse_cursor!(cursor)
        }

        /// A function for converting a `winit::WindowEvent` to a `conrod_core::event::Input`.
        pub fn convert_window_event(
            event: &winit::event::WindowEvent,
            window: &winit::window::Window,
        ) -> Option<conrod_core::event::Input> {
            $crate::v023_convert_window_event!(event, window)
        }

        /// A function for converting a `winit::Event` to a `conrod_core::event::Input`.
        pub fn convert_event<T>(
            event: &winit::event::Event<T>,
            window: &winit::window::Window,
        ) -> Option<conrod_core::event::Input> {
            $crate::v023_convert_event!(event, window)
        }
    };
}
