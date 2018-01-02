//! Feature-gated, backend-specific functionality.
//!
//! Conrod can be thought of as a pipe, where its inputs are `conrod::event::Raw`s and its output
//! is `conrod::render::Primitives`. The following feature-gated backend modules provide helper
//! functionality for converting events and rendering primitives in a way that is suitable to each.
//!
//! If there is a popular backend that you would like to see support for that is currently missing
//! from this module, feel free to open an issue or pull request at the conrod repository.

#[cfg(feature="glium")] pub mod glium;
#[cfg(feature="winit")] pub mod winit;
#[cfg(feature="piston")] pub mod piston;
#[cfg(feature="gfx_rs")] pub mod gfx;
/// Functionality for encoding `conrod::render::Primitives` to a `vulkano::command_buffer::AutoCommandBufferBuilder`
#[cfg(feature="vulkano")] pub mod vulkano;
