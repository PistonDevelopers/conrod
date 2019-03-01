//! A gfx backend for rendering conrod primitives.
extern crate conrod_core;
#[macro_use]
extern crate gfx;
extern crate gfx_core;
extern crate gfx_device_gl;
#[cfg(not(target_arch = "wasm32"))]
extern crate conrod_winit;
#[cfg(not(target_arch = "wasm32"))]
pub extern crate ggez;
#[cfg(not(target_arch = "wasm32"))]
pub use conrod_winit::map_key;
#[cfg(not(target_arch = "wasm32"))]
pub mod ggez_event;
#[cfg(not(target_arch = "wasm32"))]
pub use ggez_event as event;

#[cfg(target_arch = "wasm32")]
pub extern crate good_web_game as ggez;
#[cfg(target_arch = "wasm32")]
pub use conrod_winit::map_key;
#[cfg(target_arch = "wasm32")]
pub mod gwg_event;
#[cfg(target_arch = "wasm32")]
pub use gwg_event as event;
#[cfg(target_arch = "wasm32")]
extern crate stdweb;

use gfx::{Factory, texture, PipelineState,
          handle::RawRenderTargetView,
          traits::FactoryExt,
};

use conrod_core::{
    Rect,
    Scalar,
    color,
    image,
    render,
    text::{rt, GlyphCache},
};

/// A `Command` describing a step in the drawing process.
#[derive(Clone, Debug)]
pub enum Command<'a> {
    /// Draw to the target.
    Draw(Draw<'a>),
    /// Update the scizzor within the pipeline.
    Scizzor(gfx::Rect),
}

/// An iterator yielding `Command`s, produced by the `Renderer::commands` method.
pub struct Commands<'a> {
    commands: std::slice::Iter<'a, PreparedCommand>,
    vertices: &'a [Vertex],
}

/// A `Command` for drawing to the target.
///
/// Each variant describes how to draw the contents of the vertex buffer.
#[derive(Clone, Debug)]
pub enum Draw<'a> {
    /// A range of vertices representing triangles textured with the image in the
    /// image_map at the given `widget::Id`.
    Image(image::Id, &'a [Vertex]),
    /// A range of vertices representing plain triangles.
    Plain(&'a [Vertex]),
}

enum PreparedCommand {
    Image(image::Id, std::ops::Range<usize>),
    Plain(std::ops::Range<usize>),
    Scizzor(gfx::Rect),
}

/// Draw text from the text cache texture `tex` in the fragment shader.
pub const MODE_TEXT: u32 = 0;
/// Draw an image from the texture at `tex` in the fragment shader.
pub const MODE_IMAGE: u32 = 1;
/// Ignore `tex` and draw simple, colored 2D geometry.
pub const MODE_GEOMETRY: u32 = 2;

const FRAGMENT_SHADER: &'static [u8] = b"
    #version 140
    uniform sampler2D t_Color;

    in vec2 v_Uv;
    in vec4 v_Color;
    flat in uint v_Mode;

    out vec4 f_Color;

    void main() {
        // Text
        if (v_Mode == uint(0)) {
            f_Color = v_Color * vec4(1.0, 1.0, 1.0, texture(t_Color, v_Uv).a);

        // Image
        } else if (v_Mode == uint(1)) {
            f_Color = texture(t_Color, v_Uv);

        // 2D Geometry
        } else if (v_Mode == uint(2)) {
            f_Color = v_Color;
        }
    }
";

const VERTEX_SHADER: &'static [u8] = b"
    #version 140

    in vec2 a_Pos;
    in vec2 a_Uv;
    in vec4 a_Color;
    in uint a_Mode;

    out vec2 v_Uv;
    out vec4 v_Color;
    flat out uint v_Mode;

    void main() {
        v_Uv = a_Uv;
        v_Color = a_Color;
        gl_Position = vec4(a_Pos, 0.0, 1.0);
        v_Mode = a_Mode;
    }
";

/// Possible errors that may occur during a call to `Renderer::new`.
#[derive(Debug)]
pub enum RendererCreationError {
    /// Errors that might occur when creating the pipeline.
    PipelineState(gfx::PipelineStateError<String>),
}

// Format definitions (must be pub for gfx_defines to use them)
/// Color format used with gfx buffers.
pub type ColorFormat = gfx::format::Srgba8;
type SurfaceFormat = gfx::format::R8_G8_B8_A8;
type FullFormat = (SurfaceFormat, gfx::format::Unorm);

// This is it's own module to allow_unsafe within it
mod defines {
    //it appears gfx_defines generates unsafe code
    #![allow(unsafe_code)]

    use gfx;
    use super::ColorFormat;
    // Vertex and pipeline declarations
    gfx_defines! {
        vertex Vertex {
            pos: [f32; 2] = "a_Pos",
            uv: [f32; 2] = "a_Uv",
            color: [f32; 4] = "a_Color",
            mode: u32 = "a_Mode",
        }

        pipeline pipe {
            vbuf: gfx::VertexBuffer<Vertex> = (),
            color: gfx::TextureSampler<[f32; 4]> = "t_Color",
            scissor: gfx::Scissor = (),
            //out: gfx::RenderTarget<ColorFormat> = "Target0",
            out: gfx::BlendTarget<ColorFormat> = ("f_Color", ::gfx::state::ColorMask::all(), ::gfx::preset::blend::ALPHA),
        }
        
    }
}

use self::defines::*;

/// This type is used for translating `render::Primitives` into `Commands`s that indicate how to
/// draw the GUI using `gfx`.
pub struct Renderer<'a> {
    pipeline: PipelineState<gfx_device_gl::Resources, pipe::Meta>,
    glyph_cache: GlyphCache<'a>,
    cache_tex: gfx::handle::Texture<gfx_device_gl::Resources, SurfaceFormat>,
    cache_tex_view: gfx::handle::ShaderResourceView<gfx_device_gl::Resources, [f32;4]>,
    data: pipe::Data<gfx_device_gl::Resources>,
    commands: Vec<PreparedCommand>,
    vertices: Vec<Vertex>,
}

impl<'a> Renderer<'a> {
    /// Create a new renderer from a `gfx::Factory`, `gfx::handle::RawRenderTargetView` and
    /// a given `dpi_factor`
    pub fn new<F>(factory: &mut F,
                  rtv: &RawRenderTargetView<gfx_device_gl::Resources>,
                  dpi_factor: f64)
                  -> Result<Self, RendererCreationError>
        where F: Factory<gfx_device_gl::Resources>,
    {
        let sampler_info = texture::SamplerInfo::new(
            texture::FilterMethod::Bilinear,
            texture::WrapMode::Clamp,
        );
        let sampler = factory.create_sampler(sampler_info);

        let vbuf = factory.create_vertex_buffer(&[]);
        let (_, fake_texture) = create_texture(factory, 1, 1, &[0; 4]);

        let (width, height, _depth, _samples) = rtv.get_dimensions();

        let data = pipe::Data {
            vbuf,
            scissor: gfx::Rect { x: 0, y: 0, w: width, h: height },
            color: (fake_texture.clone(), sampler),
            out: gfx::memory::Typed::new(rtv.clone()),
        };

        let shader_set = factory.create_shader_set(VERTEX_SHADER, FRAGMENT_SHADER).unwrap();

        let pipeline = factory.create_pipeline_state(
            &shader_set,
            gfx::Primitive::TriangleList,
            gfx::state::Rasterizer {
                samples: Some(gfx::state::MultiSample {}),
                ..gfx::state::Rasterizer::new_fill()
            },
            pipe::new())?;

        let (glyph_cache, cache_tex, cache_tex_view) = {
            let width = (width as f64 * dpi_factor) as u32;
            let height = (height as f64 * dpi_factor) as u32;

            const SCALE_TOLERANCE: f32 = 0.1;
            const POSITION_TOLERANCE: f32 = 0.1;

            let cache = GlyphCache::builder()
                .dimensions(width, height)
                .scale_tolerance(SCALE_TOLERANCE)
                .position_tolerance(POSITION_TOLERANCE)
                .build();

            let data = vec![0; (width * height * 4) as usize];

            let (texture, texture_view) = create_texture(factory, width, height, &data);

            (cache, texture, texture_view)
        };
        Ok(Renderer {
            pipeline,
            glyph_cache,
            cache_tex,
            cache_tex_view,
            data,
            commands: vec![],
            vertices: vec![],
        })
    }

    /// Produce an `Iterator` yielding `Command`s.
    pub fn commands(&self) -> Commands {
        let Renderer { ref commands, ref vertices, .. } = *self;
        Commands {
            commands: commands.iter(),
            vertices: vertices,
        }
    }

    /// Fill the inner vertex and command buffers by translating the given `primitives`.
    pub fn fill<P, C>(&mut self,
                      encoder: &mut gfx::Encoder<gfx_device_gl::Resources, C>,
                      dims: (f32, f32),
                      dpi_factor: f64,
                      mut primitives: P,
                      image_map: &image::Map<(gfx::handle::ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>,
                                              (u32, u32))>)
        where P: render::PrimitiveWalker,
              C: gfx::CommandBuffer<gfx_device_gl::Resources>,
    {
        let Renderer { ref mut commands, ref mut vertices, ref mut glyph_cache, ref mut cache_tex, .. } = *self;

        commands.clear();
        vertices.clear();

        enum State {
            Image { image_id: image::Id, start: usize },
            Plain { start: usize },
        }

        let mut current_state = State::Plain { start: 0 };

        // Switches to the `Plain` state and completes the previous `Command` if not already in the
        // `Plain` state.
        macro_rules! switch_to_plain_state {
            () => {
                match current_state {
                    State::Plain { .. } => (),
                    State::Image { image_id, start } => {
                        commands.push(PreparedCommand::Image(image_id, start..vertices.len()));
                        current_state = State::Plain { start: vertices.len() };
                    },
                }
            };
        }

        // Framebuffer dimensions and the "dots per inch" factor.
        let (screen_w, screen_h) = dims;
        let (win_w, win_h) = (screen_w as Scalar, screen_h as Scalar);
        let half_win_w = win_w / 2.0;
        let half_win_h = win_h / 2.0;

        // Functions for converting for conrod scalar coords to GL vertex coords (-1.0 to 1.0).
        let vx = |x: Scalar| (x * dpi_factor / half_win_w) as f32;
        let vy = |y: Scalar| (y * dpi_factor / half_win_h) as f32;

        let mut current_scizzor = gfx::Rect {
            x: 0,
            w: screen_w as u16,
            y: 0,
            h: screen_h as u16,
        };

        let rect_to_gfx_rect = |rect: Rect| {
            let (w, h) = rect.w_h();
            let left = (rect.left() * dpi_factor + half_win_w) as u16;
            let bottom = (rect.bottom() * dpi_factor + half_win_h) as u16;
            let width = (w * dpi_factor) as u16;
            let height = (h * dpi_factor) as u16;
            gfx::Rect {
                x: std::cmp::max(left, 0),
                w: std::cmp::min(width, screen_w as u16),
                y: std::cmp::max(bottom, 0),
                h: std::cmp::min(height, screen_h as u16),
            }
        };

        // Draw each primitive in order of depth.
        while let Some(primitive) = primitives.next_primitive() {
            let render::Primitive { kind, scizzor, rect, .. } = primitive;

            // Check for a `Scizzor` command.
            let new_scizzor = rect_to_gfx_rect(scizzor);
            if new_scizzor != current_scizzor {
                // Finish the current command.
                match current_state {
                    State::Plain { start } =>
                        commands.push(PreparedCommand::Plain(start..vertices.len())),
                    State::Image { image_id, start } =>
                        commands.push(PreparedCommand::Image(image_id, start..vertices.len())),
                }

                // Update the scizzor and produce a command.
                current_scizzor = new_scizzor;
                commands.push(PreparedCommand::Scizzor(new_scizzor));

                // Set the state back to plain drawing.
                current_state = State::Plain { start: vertices.len() };
            }

            match kind {
                render::PrimitiveKind::Rectangle { color } => {
                    switch_to_plain_state!();

                    let color = gamma_srgb_to_linear(color.to_fsa());
                    let (l, r, b, t) = rect.l_r_b_t();

                    let v = |x, y| {
                        // Convert from conrod Scalar range to GL range -1.0 to 1.0.
                        Vertex {
                            pos: [vx(x), vy(y)],
                            uv: [0.0, 0.0],
                            color: color,
                            mode: MODE_GEOMETRY,
                        }
                    };

                    let mut push_v = |x, y| vertices.push(v(x, y));

                    // Bottom left triangle.
                    push_v(l, t);
                    push_v(r, b);
                    push_v(l, b);

                    // Top right triangle.
                    push_v(l, t);
                    push_v(r, b);
                    push_v(r, t);
                },
                _ =>{}
/*
                render::PrimitiveKind::TrianglesSingleColor { color, triangles } => {
                    if triangles.is_empty() {
                        continue;
                    }

                    switch_to_plain_state!();

                    let color = gamma_srgb_to_linear(color.into());

                    let v = |p: [Scalar; 2]| {
                        Vertex {
                            pos: [vx(p[0]), vy(p[1])],
                            uv: [0.0, 0.0],
                            color: color,
                            mode: MODE_GEOMETRY,
                        }
                    };

                    for triangle in triangles {
                        vertices.push(v(triangle[0]));
                        vertices.push(v(triangle[1]));
                        vertices.push(v(triangle[2]));
                    }
                }

                render::PrimitiveKind::TrianglesMultiColor { triangles } => {
                    if triangles.is_empty() {
                        continue;
                    }

                    switch_to_plain_state!();

                    let v = |(p, c): ([Scalar; 2], color::Rgba)| {
                        Vertex {
                            pos: [vx(p[0]), vy(p[1])],
                            uv: [0.0, 0.0],
                            color: gamma_srgb_to_linear(c.into()),
                            mode: MODE_GEOMETRY,
                        }
                    };

                    for triangle in triangles {
                        vertices.push(v(triangle[0]));
                        vertices.push(v(triangle[1]));
                        vertices.push(v(triangle[2]));
                    }
                }

                render::PrimitiveKind::Text { color, text, font_id } => {
                    switch_to_plain_state!();

                    let positioned_glyphs = text.positioned_glyphs(dpi_factor as f32);

                    // Queue the glyphs to be cached
                    for glyph in positioned_glyphs {
                        glyph_cache.queue_glyph(font_id.index(), glyph.clone());
                    }

                    glyph_cache.cache_queued(|rect, data| {
                        let offset = [rect.min.x as u16, rect.min.y as u16];
                        let size = [rect.width() as u16, rect.height() as u16];

                        let new_data = data.iter().map(|x| [255, 255, 255, *x]).collect::<Vec<_>>();

                        update_texture(encoder, &cache_tex, offset, size, &new_data);
                    }).unwrap();

                    let color = gamma_srgb_to_linear(color.to_fsa());
                    let cache_id = font_id.index();
                    let origin = rt::point(0.0, 0.0);

                    // A closure to convert RustType rects to GL rects
                    let to_gl_rect = |screen_rect: rt::Rect<i32>| rt::Rect {
                        min: origin
                            + (rt::vector(screen_rect.min.x as f32 / screen_w - 0.5,
                                          1.0 - screen_rect.min.y as f32 / screen_h - 0.5)) * 2.0,
                        max: origin
                            + (rt::vector(screen_rect.max.x as f32 / screen_w - 0.5,
                                          1.0 - screen_rect.max.y as f32 / screen_h - 0.5)) * 2.0,
                    };

                    for g in positioned_glyphs {
                        if let Ok(Some((uv_rect, screen_rect))) = glyph_cache.rect_for(cache_id, g) {
                            let gl_rect = to_gl_rect(screen_rect);
                            let v = |p, t| Vertex {
                                pos: p,
                                uv: t,
                                color: color,
                                mode: MODE_TEXT,
                            };
                            let mut push_v = |p, t| vertices.push(v(p, t));
                            push_v([gl_rect.min.x, gl_rect.max.y], [uv_rect.min.x, uv_rect.max.y]);
                            push_v([gl_rect.min.x, gl_rect.min.y], [uv_rect.min.x, uv_rect.min.y]);
                            push_v([gl_rect.max.x, gl_rect.min.y], [uv_rect.max.x, uv_rect.min.y]);
                            push_v([gl_rect.max.x, gl_rect.min.y], [uv_rect.max.x, uv_rect.min.y]);
                            push_v([gl_rect.max.x, gl_rect.max.y], [uv_rect.max.x, uv_rect.max.y]);
                            push_v([gl_rect.min.x, gl_rect.max.y], [uv_rect.min.x, uv_rect.max.y]);
                        }
                    }
                }

                render::PrimitiveKind::Image { image_id, color, source_rect } => {

                    // Switch to the `Image` state for this image if we're not in it already.
                    let new_image_id = image_id;
                    match current_state {

                        // If we're already in the drawing mode for this image, we're done.
                        State::Image { image_id, .. } if image_id == new_image_id => (),

                        // If we were in the `Plain` drawing state, switch to Image drawing state.
                        State::Plain { start } => {
                            commands.push(PreparedCommand::Plain(start..vertices.len()));
                            current_state = State::Image {
                                image_id: new_image_id,
                                start: vertices.len(),
                            };
                        }

                        // If we were drawing a different image, switch state to draw *this* image.
                        State::Image { image_id, start } => {
                            commands.push(PreparedCommand::Image(image_id, start..vertices.len()));
                            current_state = State::Image {
                                image_id: new_image_id,
                                start: vertices.len(),
                            };
                        }
                    }

                    let color = color.unwrap_or(color::WHITE).to_fsa();

                    let (image_w, image_h) = image_map.get(&image_id).unwrap().1;
                    let (image_w, image_h) = (image_w as Scalar, image_h as Scalar);

                    // Get the sides of the source rectangle as uv coordinates.
                    //
                    // Texture coordinates range:
                    // - left to right: 0.0 to 1.0
                    // - bottom to top: 1.0 to 0.0
                    // Note bottom and top are flipped in comparison to glium so that we don't need to flip images when loading
                    let (uv_l, uv_r, uv_t, uv_b) = match source_rect {
                        Some(src_rect) => {
                            let (l, r, b, t) = src_rect.l_r_b_t();
                            ((l / image_w) as f32,
                             (r / image_w) as f32,
                             (b / image_h) as f32,
                             (t / image_h) as f32)
                        }
                        None => (0.0, 1.0, 0.0, 1.0),
                    };

                    let v = |x, y, t| {
                        // Convert from conrod Scalar range to GL range -1.0 to 1.0.
                        let x = (x * dpi_factor as Scalar / half_win_w) as f32;
                        let y = (y * dpi_factor as Scalar / half_win_h) as f32;
                        Vertex {
                            pos: [x, y],
                            uv: t,
                            color: color,
                            mode: MODE_IMAGE,
                        }
                    };

                    let mut push_v = |x, y, t| vertices.push(v(x, y, t));

                    let (l, r, b, t) = rect.l_r_b_t();

                    // Bottom left triangle.
                    push_v(l, t, [uv_l, uv_t]);
                    push_v(r, b, [uv_r, uv_b]);
                    push_v(l, b, [uv_l, uv_b]);

                    // Top right triangle.
                    push_v(l, t, [uv_l, uv_t]);
                    push_v(r, b, [uv_r, uv_b]);
                    push_v(r, t, [uv_r, uv_t]);
                }

                // We have no special case widgets to handle.
                render::PrimitiveKind::Other(_) => (),
            */
            }
            
        }

        // Enter the final command.
        match current_state {
            State::Plain { start } =>
                commands.push(PreparedCommand::Plain(start..vertices.len())),
            State::Image { image_id, start } =>
                commands.push(PreparedCommand::Image(image_id, start..vertices.len())),
        }
    }

    /// Draws using the inner list of `Command`s to the given `display`.
    ///
    /// Note: If you require more granular control over rendering, you may want to use the `fill`
    /// and `commands` methods separately. This method is simply a convenience wrapper around those
    /// methods for the case that the user does not require accessing or modifying conrod's draw
    /// parameters, uniforms or generated draw commands.
    pub fn draw<F, C>(&self, factory: &mut F, encoder: &mut gfx::Encoder<gfx_device_gl::Resources, C>, image_map: &image::Map<(gfx::handle::ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>, (u32, u32))>)
        where F: Factory<gfx_device_gl::Resources>,
              C: gfx::CommandBuffer<gfx_device_gl::Resources>,
    {
        let Renderer { ref pipeline, ref data, ref cache_tex_view, .. } = *self;

        let mut data = data.clone();

        for command in self.commands() {
            match command {

                // Update the `scizzor` before continuing to draw.
                Command::Scizzor(scizzor) => data.scissor = scizzor,

                // Draw to the target with the given `draw` command.
                Command::Draw(draw) => match draw {

                    // Draw text and plain 2D geometry.
                    Draw::Plain(verts) => {
                        data.color.0 = cache_tex_view.clone();
                        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&verts, ());
                        data.vbuf = vbuf;
                        encoder.draw(&slice, &pipeline, &data);
                    }

                    // Draw an image whose texture data lies within the `image_map` at the
                    // given `id`.
                    Draw::Image(image_id, verts) => {
                        let image = &image_map.get(&image_id).unwrap().0;
                        data.color.0 = image.clone();
                        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&verts, ());
                        data.vbuf = vbuf;
                        encoder.draw(&slice, &pipeline, &data);
                    }
                }
            }
        }
    }

    /// Call this routine when a window has been resized. This ensures that conrod primitives are
    /// drawn properly with the `draw` call.
    pub fn on_resize(&mut self, rtv: RawRenderTargetView<gfx_device_gl::Resources>) {
        let (width, height, _depth, _samples) = rtv.get_dimensions();
        self.data.out = gfx::memory::Typed::new(rtv.clone());
        self.data.scissor = gfx::Rect { x: 0, y: 0, w: width, h: height };
    }

    /// Call this routine to clear the render target.
    pub fn clear<C>(&self, encoder: &mut gfx::Encoder<gfx_device_gl::Resources, C>, clear_color: [f32; 4])
        where C: gfx::CommandBuffer<gfx_device_gl::Resources>,
    {
        encoder.clear(&self.data.out, clear_color);
    }
}

fn gamma_srgb_to_linear(c: [f32; 4]) -> [f32; 4] {
    fn component(f: f32) -> f32 {
        // Taken from https://github.com/PistonDevelopers/graphics/src/color.rs#L42
        if f <= 0.04045 {
            f / 12.92
        } else {
            ((f + 0.055) / 1.055).powf(2.4)
        }
    }
    [component(c[0]), component(c[1]), component(c[2]), c[3]]
}

// Creates a gfx texture with the given data
fn create_texture<F>(factory: &mut F, width: u32, height: u32, data: &[u8])
                        -> (gfx::handle::Texture<gfx_device_gl::Resources, SurfaceFormat>, gfx::handle::ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>)
    where F: gfx::Factory<gfx_device_gl::Resources>
{
    // Modified `Factory::create_texture_immutable_u8` for dynamic texture.
    fn create_texture<T, F>(
        factory: &mut F,
        kind: gfx::texture::Kind,
        data: &[&[u8]],
    ) -> Result<(
        gfx::handle::Texture<gfx_device_gl::Resources, T::Surface>,
        gfx::handle::ShaderResourceView<gfx_device_gl::Resources, T::View>
    ), gfx::CombinedError>
        where F: gfx::Factory<gfx_device_gl::Resources>,
              T: gfx::format::TextureFormat
    {
        use gfx::{format, texture};
        use gfx::memory::Usage;
        use gfx::memory::Bind;

        use gfx_core::memory::Typed;

        let surface = <T::Surface as format::SurfaceTyped>::get_surface_type();
        let num_slices = kind.get_num_slices().unwrap_or(1) as usize;
        let num_faces = if kind.is_cube() { 6 } else { 1 };
        let desc = texture::Info {
            kind: kind,
            levels: (data.len() / (num_slices * num_faces)) as texture::Level,
            format: surface,
            bind: Bind::SHADER_RESOURCE,
            usage: Usage::Dynamic,
        };
        let cty = <T::Channel as format::ChannelTyped>::get_channel_type();
        let raw = factory.create_texture_raw(
            desc,
            Some(cty),
            Some((data, gfx::texture::Mipmap::Provided)))?;
        let levels = (0, raw.get_info().levels - 1);
        let tex = Typed::new(raw);
        let view = factory.view_texture_as_shader_resource::<T>(
            &tex, levels, format::Swizzle::new(),
        )?;
        Ok((tex, view))
    }

    let kind = texture::Kind::D2(
        width as texture::Size,
        height as texture::Size,
        texture::AaMode::Single,
    );
    create_texture::<ColorFormat, F>(factory, kind, &[data]).unwrap()
}

// Updates a texture with the given data (used for updating the GlyphCache texture)
fn update_texture<C>(encoder: &mut gfx::Encoder<gfx_device_gl::Resources, C>,
                        texture: &gfx::handle::Texture<gfx_device_gl::Resources, SurfaceFormat>,
                        offset: [u16; 2],
                        size: [u16; 2],
                        data: &[[u8; 4]])
    where C: gfx::CommandBuffer<gfx_device_gl::Resources>
{
    let info = texture::ImageInfoCommon {
        xoffset: offset[0],
        yoffset: offset[1],
        zoffset: 0,
        width: size[0],
        height: size[1],
        depth: 0,
        format: (),
        mipmap: 0,
    };

    encoder.update_texture::<SurfaceFormat, FullFormat>(texture, None, info, data).unwrap();
}

impl<'a> Iterator for Commands<'a> {
    type Item = Command<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let Commands { ref mut commands, ref vertices } = *self;
        commands.next().map(|command| match *command {
            PreparedCommand::Scizzor(scizzor) => Command::Scizzor(scizzor),
            PreparedCommand::Plain(ref range) =>
                Command::Draw(Draw::Plain(&vertices[range.clone()])),
            PreparedCommand::Image(id, ref range) =>
                Command::Draw(Draw::Image(id, &vertices[range.clone()])),
        })
    }
}

impl From<gfx::PipelineStateError<String>> for RendererCreationError {
    fn from(err: gfx::PipelineStateError<String>) -> Self {
        RendererCreationError::PipelineState(err)
    }
}

impl std::error::Error for RendererCreationError {
    fn description(&self) -> &str {
        match *self {
            RendererCreationError::PipelineState(ref e) => std::error::Error::description(e),
        }
    }
}

impl std::fmt::Display for RendererCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            RendererCreationError::PipelineState(ref e) => std::fmt::Display::fmt(e, f),
        }
    }
}
