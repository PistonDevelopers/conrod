//! Contains the GfxContext wrapper for convenient integration with `backend::piston::Window`

extern crate window as pistoncore_window;
extern crate graphics as piston_graphics;
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate gfx_graphics;
extern crate shader_version;
extern crate texture;

use self::shader_version::OpenGL;
use self::gfx_graphics::{Gfx2d, GfxGraphics};
use self::gfx_core::factory::Typed;
use self::gfx::Device;

use self::pistoncore_window::{OpenGLWindow, Size};
use piston_input::RenderArgs;

use super::draw;
use image;
use render;
use text;

pub use self::piston_graphics::{Context, DrawState, Graphics, ImageSize, Transformed};
pub use self::gfx_graphics::{GlyphError, Texture, TextureSettings, Flip};

/// Actual gfx::Stream implementation carried by the window.
pub type GfxEncoder = gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>;
/// Glyph cache.
pub type Glyphs = gfx_graphics::GlyphCache<gfx_device_gl::Resources, gfx_device_gl::Factory>;
/// 2D graphics.
pub type G2d<'a> = GfxGraphics<'a, gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>;
/// Texture type compatible with `G2d`.
pub type G2dTexture<'a> = Texture<gfx_device_gl::Resources>;

/// Contains state used by Gfx to draw. Can be stored within a window.
pub struct GfxContext {
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

fn create_main_targets(dim: gfx::tex::Dimensions) ->
    (gfx::handle::RenderTargetView<gfx_device_gl::Resources, gfx::format::Srgba8>,
     gfx::handle::DepthStencilView<gfx_device_gl::Resources, gfx::format::DepthStencil>)
 {
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

impl GfxContext {
    /// Constructor for a new `GfxContext`
    pub fn new<W>(window: &mut W, opengl: OpenGL, samples: u8) -> Self
        where W: OpenGLWindow 
    {
        let (device, mut factory) = gfx_device_gl::create(|s| window.get_proc_address(s) as *const _);

        let draw_size = window.draw_size();
        let (output_color, output_stencil) = {
            let aa = samples as gfx::tex::NumSamples;
            let dim = (draw_size.width as u16, draw_size.height as u16,
                       1, aa.into());
            create_main_targets(dim)
        };

        let g2d = Gfx2d::new(opengl, &mut factory);
        let encoder = factory.create_command_buffer().into();
        GfxContext {
            encoder: encoder,
            device: device,
            output_color: output_color,
            output_stencil: output_stencil,
            g2d: g2d,
            factory: factory,
        }
    }

    /// Renders 2D graphics.
    pub fn draw_2d<F, U>(&mut self, f: F, args: RenderArgs) -> U where
        F: FnOnce(draw::Context, &mut G2d) -> U
    {
        let res = self.g2d.draw(
            &mut self.encoder,
            &self.output_color,
            &self.output_stencil,
            args.viewport(),
            f
        );
        self.encoder.flush(&mut self.device);
        res
    }

    /// Called after frame is rendered to cleanup after gfx device.
    pub fn after_render(&mut self) {
        self.device.cleanup();
    }

    /// Check whether window has resized and update the output.
    pub fn check_resize(&mut self, draw_size: Size) {
        let dim = self.output_color.raw().get_dimensions();
        let (w, h) = (dim.0, dim.1);
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
    /// The `width` and `height` arguments are in pixel values.
    ///
    /// If you need to resize the `GlyphCache`, construct a new one and discard the old one.
    pub fn new_from_factory(factory: &mut gfx_device_gl::Factory, width: u32, height: u32) -> GlyphCache {

        // Construct the rusttype GPU cache with the tolerances recommended by their documentation.
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;
        let cache = text::GlyphCache::new(width, height, SCALE_TOLERANCE, POSITION_TOLERANCE);

        // Construct a `G2dTexture`
        let buffer_len = width as usize * height as usize;
        let init = vec![128; buffer_len];
        let settings = TextureSettings::new();
        let texture = G2dTexture::from_memory_alpha(factory, &init, width, height, &settings).unwrap();

        GlyphCache {
            cache: cache,
            texture: texture,
            vertex_data: Vec::new(),
        }
    }
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
                                      glyph_rectangles: &mut Vec<([f64; 4], [f64; 4])>,
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
