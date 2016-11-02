#![deny(missing_docs)]

//! This code is adapted from the `piston_window` crate for convenience when
//! using the piston window API with a conrod app.
//!
//! Provides a simple API over the gfx graphics backend and the glutin window
//! context and events.
//!
//! Sets up:
//!
//! - [Gfx](https://github.com/gfx-rs/gfx) with an OpenGL back-end.
//! - [gfx_graphics](https://github.com/pistondevelopers/gfx_graphics)
//! for 2D rendering.
//! - [glutin_window](https://github.com/pistondevelopers/glutin_window)
//! as default window back-end, but this can be swapped (see below).
//!
//! ### Example
//!
//! ```no_run
//! extern crate conrod;
//!
//! use conrod::backend::piston_window::*;
//!
//! fn main() {
//!     let mut window: PistonWindow =
//!         WindowSettings::new("Hello World!", [512; 2])
//!             .build().unwrap();
//!     while let Some(e) = window.next() {
//!         window.draw_2d(&e, |c, g| {
//!             clear([0.5, 0.5, 0.5, 1.0], g);
//!             rectangle([1.0, 0.0, 0.0, 1.0], // red
//!                       [0.0, 0.0, 100.0, 100.0], // rectangle
//!                       c.transform, g);
//!         });
//!     }
//! }
//! ```
//!
//! ### Swap to another window back-end
//!
//! Change the generic parameter to the window back-end you want to use.
//!
//! ```ignore
//! extern crate conrod;
//! extern crate sdl2_window;
//!
//! use conrod::backend::piston_window::*;
//! use sdl2_window::Sdl2Window;
//!
//! # fn main() {
//!
//! let window: PistonWindow<Sdl2Window> =
//!     WindowSettings::new("title", [512; 2])
//!         .build().unwrap();
//!
//! # }
//! ```
//!
//! ### sRGB
//!
//! The impl of `BuildFromWindowSettings` in this library turns on
//! `WindowSettings::srgb`, because it is required by gfx_graphics.
//!
//! Most images such as those found on the internet uses sRGB,
//! that has a non-linear gamma corrected space.
//! When rendering 3D, make sure textures and colors are in linear gamma space.
//! Alternative is to use `Srgb8` and `Srgba8` formats for textures.
//!
//! For more information about sRGB, see
//! https://github.com/PistonDevelopers/piston/issues/1014
//!
//! ### Library dependencies
//!
//! This library is meant to be used in applications only.
//! It is not meant to be depended on by generic libraries.
//! Instead, libraries should depend on the lower abstractions,
//! such as the [Piston core](https://github.com/pistondevelopers/piston).

extern crate window as pistoncore_window;
extern crate event_loop as pistoncore_event_loop;
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate gfx_graphics;
extern crate graphics;
extern crate shader_version;
extern crate glutin_window;
pub extern crate texture;

use self::glutin_window::GlutinWindow;
pub use self::shader_version::OpenGL;
pub use self::graphics::{ImageSize};
pub use self::pistoncore_window::*;
pub use piston_input::*;
pub use self::gfx_graphics::{ GlyphError, Texture, TextureSettings, Flip };

use self::gfx_graphics::{ Gfx2d, GfxGraphics };
use std::time::Duration;

use super::piston::draw;
use event;
use image;
use render;
use text;

/// Actual gfx::Stream implementation carried by the window.
pub type GfxEncoder = gfx::Encoder<gfx_device_gl::Resources,
    gfx_device_gl::CommandBuffer>;
/// Glyph cache.
pub type Glyphs = gfx_graphics::GlyphCache<gfx_device_gl::Resources,
    gfx_device_gl::Factory>;
/// 2D graphics.
pub type G2d<'a> = GfxGraphics<'a,
    gfx_device_gl::Resources,
    gfx_device_gl::CommandBuffer>;
/// Texture type compatible with `G2d`.
pub type G2dTexture<'a> = Texture<gfx_device_gl::Resources>;

/// Contains everything required for controlling window, graphics, event loop.
pub struct PistonWindow<W: Window = GlutinWindow> {
    /// The window.
    pub window: W,
    /// GFX encoder.
    pub encoder: GfxEncoder,
    /// GFX device.
    pub device: gfx_device_gl::Device,
    /// Output frame buffer.
    pub output_color: gfx::handle::RenderTargetView<
        gfx_device_gl::Resources, gfx::format::Srgba8>,
    /// Output stencil buffer.
    pub output_stencil: gfx::handle::DepthStencilView<
        gfx_device_gl::Resources, gfx::format::DepthStencil>,
    /// Gfx2d.
    pub g2d: Gfx2d<gfx_device_gl::Resources>,
    /// The factory that was created along with the device.
    pub factory: gfx_device_gl::Factory,
}

impl<W> BuildFromWindowSettings for PistonWindow<W>
    where W: Window + OpenGLWindow + BuildFromWindowSettings,
          W::Event: GenericEvent
{
    fn build_from_window_settings(settings: &WindowSettings) -> Result<PistonWindow<W>, String> {
        // Turn on sRGB.
        let settings = settings.clone().srgb(true);

        // Use OpenGL 3.2 by default, because this is what window backends
        // usually do.
        let opengl = settings.get_maybe_opengl().unwrap_or(OpenGL::V3_2);
        let samples = settings.get_samples();

        Ok(PistonWindow::new(opengl, samples, try!(settings.build())))
    }
}

fn create_main_targets(dim: gfx::tex::Dimensions) ->
(gfx::handle::RenderTargetView<
    gfx_device_gl::Resources, gfx::format::Srgba8>,
 gfx::handle::DepthStencilView<
    gfx_device_gl::Resources, gfx::format::DepthStencil>) {
    use self::gfx_core::factory::Typed;
    use self::gfx::format::{DepthStencil, Format, Formatted, Srgba8};

    let color_format: Format = <Srgba8 as Formatted>::get_format();
    let depth_format: Format = <DepthStencil as Formatted>::get_format();
    let (output_color, output_stencil) =
        gfx_device_gl::create_main_targets_raw(dim,
                                               color_format.0,
                                               depth_format.0);
    let output_color = Typed::new(output_color);
    let output_stencil = Typed::new(output_stencil);
    (output_color, output_stencil)
}

impl<W> PistonWindow<W>
    where W: Window, W::Event: GenericEvent
{
    /// Creates a new piston window.
    pub fn new(opengl: OpenGL, samples: u8, mut window: W) -> Self
        where W: OpenGLWindow
    {
        let (device, mut factory) =
            gfx_device_gl::create(|s|
                window.get_proc_address(s) as *const _);

        let (output_color, output_stencil) = {
            let aa = samples as gfx::tex::NumSamples;
            let draw_size = window.draw_size();
            let dim = (draw_size.width as u16, draw_size.height as u16,
                       1, aa.into());
            create_main_targets(dim)
        };

        let g2d = Gfx2d::new(opengl, &mut factory);
        let encoder = factory.create_command_buffer().into();
        PistonWindow {
            window: window,
            encoder: encoder,
            device: device,
            output_color: output_color,
            output_stencil: output_stencil,
            g2d: g2d,
            factory: factory,
        }
    }

    /// Renders 2D graphics.
    pub fn draw_2d<E, F, U>(&mut self, e: &E, f: F) -> Option<U> where
        W: OpenGLWindow,
        E: GenericEvent,
        F: FnOnce(draw::Context, &mut G2d) -> U
    {
        self.window.make_current();
        if let Some(args) = e.render_args() {
            let res = self.g2d.draw(
                &mut self.encoder,
                &self.output_color,
                &self.output_stencil,
                args.viewport(),
                f
            );
            self.encoder.flush(&mut self.device);
            Some(res)
        } else {
            None
        }
    }

    /// Renders 3D graphics.
    pub fn draw_3d<E, F, U>(&mut self, e: &E, f: F) -> Option<U> where
        W: OpenGLWindow,
        E: GenericEvent,
        F: FnOnce(&mut Self) -> U
    {
        self.window.make_current();
        if let Some(_) = e.render_args() {
            let res = f(self);
            self.encoder.flush(&mut self.device);
            Some(res)
        } else {
            None
        }
    }

    /// Let window handle new event.
    /// Cleans up after rendering and resizes frame buffers.
    pub fn event(&mut self, event: &Event<<W as Window>::Event>) {
        use piston_input::*;
        use self::gfx_core::factory::Typed;
        use self::gfx::Device;

        if let Some(_) = event.after_render_args() {
            // After swapping buffers.
            self.device.cleanup();
        }

        // Check whether window has resized and update the output.
        let dim = self.output_color.raw().get_dimensions();
        let (w, h) = (dim.0, dim.1);
        let draw_size = self.window.draw_size();
        if w != draw_size.width as u16 || h != draw_size.height as u16 {
            let dim = (draw_size.width as u16,
                       draw_size.height as u16,
                       dim.2, dim.3);
            let (output_color, output_stencil) = create_main_targets(dim);
            self.output_color = output_color;
            self.output_stencil = output_stencil;
        }
    }
}

impl<W> Window for PistonWindow<W>
    where W: Window
{
    type Event = <W as Window>::Event;

    fn should_close(&self) -> bool { self.window.should_close() }
    fn set_should_close(&mut self, value: bool) {
        self.window.set_should_close(value)
    }
    fn size(&self) -> Size { self.window.size() }
    fn draw_size(&self) -> Size { self.window.draw_size() }
    fn swap_buffers(&mut self) { self.window.swap_buffers() }
    fn wait_event(&mut self) -> Self::Event {
        Window::wait_event(&mut self.window)
    }
    fn wait_event_timeout(&mut self, timeout: Duration) -> Option<Self::Event> {
        Window::wait_event_timeout(&mut self.window, timeout)
    }
    fn poll_event(&mut self) -> Option<Self::Event> {
        Window::poll_event(&mut self.window)
    }
}

impl<W> AdvancedWindow for PistonWindow<W>
    where W: AdvancedWindow
{
    fn get_title(&self) -> String { self.window.get_title() }
    fn set_title(&mut self, title: String) {
        self.window.set_title(title)
    }
    fn get_exit_on_esc(&self) -> bool { self.window.get_exit_on_esc() }
    fn set_exit_on_esc(&mut self, value: bool) {
        self.window.set_exit_on_esc(value)
    }
    fn set_capture_cursor(&mut self, value: bool) {
        self.window.set_capture_cursor(value)
    }
    fn show(&mut self) { self.window.show() }
    fn hide(&mut self) { self.window.hide() }
    fn get_position(&self) -> Option<Position> {
        self.window.get_position()
    }
    fn set_position<P: Into<Position>>(&mut self, pos: P) {
        self.window.set_position(pos)
    }
}

/// Used to integrate `PistonWindow` with an event loop, enables
/// `PistonWindow` to handle some events, if necessary
pub trait EventWindow {
    /// receive next event from event loop and handle it
    fn next(&mut self, events: &mut PistonWindow) -> Option<Event>;
}

/// This module allows use of `WindowEvents` from `pistoncore_event_loop` with `PistonWindow`
pub mod piston_event_loop {
    extern crate event_loop;

    pub use self::event_loop::{WindowEvents, EventLoop};
    use super::{PistonWindow, EventWindow};
    use piston_input::Event;

    impl EventWindow for WindowEvents {
        fn next(&mut self, window: &mut PistonWindow) -> Option<Event> {
            if let Some(e) = self.next(window) {
                window.event(&e);
                Some(e)
            } else { None }
        }
    }
}

/// A wrapper around a `G2dTexture` and a rusttype GPU `Cache`
///
/// Using a wrapper simplifies the construction of both caches and ensures that they maintain
/// identical dimensions.
pub struct GlyphCache {
    cache: text::GlyphCache,
    texture: G2dTexture<'static>,
    vertex_data: Vec<u8>,
}


impl GlyphCache {

    /// Constructor for a new `GlyphCache`.
    ///
    /// The `PistonWindow` provides the `Factory` argument to the `G2dTexture` constructor.
    ///
    /// The `width` and `height` arguments are in pixel values.
    ///
    /// If you need to resize the `GlyphCache`, construct a new one and discard the old one.
    pub fn new<B>(window: &mut PistonWindow<B>, width: u32, height: u32) -> GlyphCache 
        where B: Window {

        // Construct the rusttype GPU cache with the tolerances recommended by their documentation.
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;
        let cache = text::GlyphCache::new(width, height, SCALE_TOLERANCE, POSITION_TOLERANCE);

        // Construct a `G2dTexture`
        let buffer_len = width as usize * height as usize;
        let init = vec![128; buffer_len];
        let settings = TextureSettings::new();
        let factory = &mut window.factory;
        let texture = G2dTexture::from_memory_alpha(factory, &init, width, height, &settings).unwrap();

        GlyphCache {
            cache: cache,
            texture: texture,
            vertex_data: Vec::new(),
        }
    }

}


/// Converts any `GenericEvent` to a `Raw` conrod event.
pub fn convert_event<E, B>(event: E, window: &PistonWindow<B>) -> Option<event::Raw>
    where E: GenericEvent,
          B: Window,
{
    use Scalar;

    let size = window.size();
    let (w, h) = (size.width as Scalar, size.height as Scalar);
    super::piston::event::convert(event, w, h)
}


/// Renders the given sequence of conrod primitives.
pub fn draw<'a, 'b, P, Img, F>(context: draw::Context,
                               graphics: &'a mut G2d<'b>,
                               primitives: P,
                               glyph_cache: &'a mut GlyphCache,
                               image_map: &'a image::Map<Img>,
                               texture_from_image: F)
    where P: render::PrimitiveWalker,
          F: FnMut(&Img) -> &G2dTexture<'static>,
{
    let GlyphCache { ref mut texture, ref mut cache, ref mut vertex_data } = *glyph_cache;

    // A function used for caching glyphs from `Text` widgets.
    let cache_queued_glyphs_fn = |graphics: &mut G2d,
                                  cache: &mut G2dTexture<'static>,
                                  rect: text::rt::Rect<u32>,
                                  data: &[u8]|
    {
        cache_queued_glyphs(graphics, cache, rect, data, vertex_data);
    };

    draw::primitives(
        primitives,
        context,
        graphics,
        texture,
        cache,
        image_map,
        cache_queued_glyphs_fn,
        texture_from_image,
    );
}

/// Draw a single `Primitive` to the screen.
///
/// This is useful if the user requires rendering primitives individually, perhaps to perform their
/// own rendering in between, etc.
pub fn draw_primitive<'a, 'b, Img, F>(context: draw::Context,
                                      graphics: &'a mut G2d<'b>,
                                      primitive: render::Primitive,
                                      glyph_cache: &'a mut GlyphCache,
                                      image_map: &'a image::Map<Img>,
                                      glyph_rectangles: &mut Vec<([f64; 4], [i32; 4])>,
                                      texture_from_image: F)
    where F: FnMut(&Img) -> &G2dTexture<'static>,
{
    let GlyphCache { ref mut texture, ref mut cache, ref mut vertex_data } = *glyph_cache;

    // A function used for caching glyphs from `Text` widgets.
    let cache_queued_glyphs_fn = |graphics: &mut G2d,
                                  cache: &mut G2dTexture<'static>,
                                  rect: text::rt::Rect<u32>,
                                  data: &[u8]|
    {
        cache_queued_glyphs(graphics, cache, rect, data, vertex_data);
    };

    draw::primitive(
        primitive,
        context,
        graphics,
        texture,
        cache,
        image_map,
        glyph_rectangles,
        cache_queued_glyphs_fn,
        texture_from_image,
    );
}

fn cache_queued_glyphs(graphics: &mut G2d,
                       cache: &mut G2dTexture<'static>,
                       rect: text::rt::Rect<u32>,
                       data: &[u8],
                       vertex_data: &mut Vec<u8>)
{
    use self::texture::UpdateTexture;

    // An iterator that efficiently maps the `byte`s yielded from `data` to `[r, g, b, byte]`;
    //
    // This is only used within the `cache_queued_glyphs` below, however due to a bug in rustc we
    // are unable to declare types inside the closure scope.
    struct Bytes { b: u8, i: u8 }
    impl Iterator for Bytes {
        type Item = u8;
        fn next(&mut self) -> Option<Self::Item> {
            let b = match self.i {
                0 => 255,
                1 => 255,
                2 => 255,
                3 => self.b,
                _ => return None,
            };
            self.i += 1;
            Some(b)
        }
    }

    let offset = [rect.min.x, rect.min.y];
    let size = [rect.width(), rect.height()];
    let format = self::texture::Format::Rgba8;
    let encoder = &mut graphics.encoder;

    vertex_data.clear();
    vertex_data.extend(data.iter().flat_map(|&b| Bytes { b: b, i: 0 }));
    UpdateTexture::update(cache, encoder, format, &vertex_data[..], offset, size)
        .expect("Failed to update texture");
}
