//! A function for converting a `winit::Event` to a `conrod::event::Input`.

pub mod macros;

/// Types that have access to a `winit::Window` and can provide the necessary dimensions and hidpi
/// factor for converting `winit::Event`s to `conrod::event::Input`, as well as set the mouse
/// cursor.
///
/// This allows users to pass references to window types like `glium::Display`,
/// `glium::glutin::Window` or `winit::Window`
pub trait WinitWindow {
    /// Return the inner size of the window in logical pixels.
    fn get_inner_size(&self) -> Option<(u32, u32)>;
    /// Return the window's DPI factor so that we can convert from pixel values to scalar values.
    fn hidpi_factor(&self) -> f32;
}
