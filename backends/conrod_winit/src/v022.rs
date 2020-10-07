#[macro_export]
macro_rules! v022_convert_key {
    ($keycode:expr) => {{
        $crate::v021_convert_key!($keycode)
    }};
}

/// Maps winit's mouse button to conrod's mouse button.
///
/// Expects a `winit::MouseButton` as input and returns a `conrod_core::input::MouseButton` as
/// output.
///
/// Requires that both the `conrod_core` and `winit` crates are in the crate root.
#[macro_export]
macro_rules! v022_convert_mouse_button {
    ($mouse_button:expr) => {{
        $crate::v021_convert_mouse_button!($mouse_button)
    }};
}

/// A macro for converting a `winit::WindowEvent` to a `Option<conrod_core::event::Input>`.
///
/// Expects a `winit::WindowEvent` and a reference to a window implementing `WinitWindow`.
/// Returns an `Option<conrod_core::event::Input>`.
#[macro_export]
macro_rules! v022_convert_window_event {
    ($event:expr, $window:expr) => {{
        $crate::v021_convert_window_event!($event, $window)
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
macro_rules! v022_convert_event {
    ($event:expr, $window:expr) => {{
        $crate::v021_convert_event!($event, $window)
    }};
}

/// Convert a given conrod mouse cursor to the corresponding winit cursor type.
///
/// Expects a `conrod_core::cursor::MouseCursor`, returns a `winit::MouseCursor`.
///
/// Requires that both the `conrod_core` and `winit` crates are in the crate root.
#[macro_export]
macro_rules! v022_convert_mouse_cursor {
    ($cursor:expr) => {{
        $crate::v021_convert_mouse_cursor!($cursor)
    }};
}

#[macro_export]
macro_rules! v022_conversion_fns {
    () => {
        $crate::v021_conversion_fns!();
    };
}
