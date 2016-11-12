//! A glium backend for rendering conrod primitives.

pub use glium;

use {Rect, Scalar};
use color;
use image;
use render;
use std;
use text;
use widget;


/// A `Command` describing a step in the drawing process.
#[derive(Clone, Debug)]
pub enum Command<'a> {
    /// Draw to the target.
    Draw(Draw<'a>),
    /// Update the scizzor within the `glium::DrawParameters`.
    Scizzor(glium::Rect),
}

/// A `Command` for drawing to the target.
///
/// Each variant describes how to draw the contents of the vertex buffer.
#[derive(Clone, Debug)]
pub enum Draw<'a> {
    /// A range of vertices representing triangles textured with the image in the
    /// image_map at the given `widget::Id`.
    Image(widget::Id, &'a [Vertex]),
    /// A range of vertices representing plain triangles.
    Plain(&'a [Vertex]),
}

enum PreparedCommand {
    Image(widget::Id, std::ops::Range<usize>),
    Plain(std::ops::Range<usize>),
    Scizzor(glium::Rect),
}

/// A rusttype `GlyphCache` along with a `glium::texture::Texture2d` for caching text on the `GPU`.
pub struct GlyphCache {
    cache: text::GlyphCache,
    texture: glium::texture::Texture2d,
}

/// A type used for translating `render::Primitives` into `Command`s that indicate how to draw the
/// conrod GUI using `glium`.
pub struct Renderer {
    program: glium::Program,
    glyph_cache: GlyphCache,
    commands: Vec<PreparedCommand>,
    vertices: Vec<Vertex>,
}

/// An iterator yielding `Command`s, produced by the `Renderer::commands` method.
pub struct Commands<'a> {
    commands: std::slice::Iter<'a, PreparedCommand>,
    vertices: &'a [Vertex],
}

/// Possible errors that may occur during a call to `Renderer::new`.
#[derive(Debug)]
pub enum RendererCreationError {
    /// Errors that might occur when creating the glyph cache texture.
    Texture(glium::texture::TextureCreationError),
    /// Errors that might occur when constructing the shader program.
    Program(glium::program::ProgramChooserCreationError),
}

/// Possible errors that may occur during a call to `Renderer::draw`.
#[derive(Debug)]
pub enum DrawError {
    /// Errors that might occur upon construction of a `glium::VertexBuffer`.
    Buffer(glium::vertex::BufferCreationError),
    /// Errors that might occur when drawing to the `glium::Surface`.
    Draw(glium::DrawError),
}

/// The `Vertex` type passed to the vertex shader.
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    /// The mode with which the `Vertex` will be drawn within the fragment shader.
    ///
    /// `0` for rendering text.
    /// `1` for rendering an image.
    /// `2` for rendering non-textured 2D geometry.
    ///
    /// If any other value is given, the fragment shader will not output any color.
    pub mode: u32,
    /// The position of the vertex within vector space.
    ///
    /// [-1.0, -1.0] is the leftmost, bottom position of the display.
    /// [1.0, 1.0] is the rightmost, top position of the display.
    pub position: [f32; 2],
    /// The coordinates of the texture used by this `Vertex`.
    ///
    /// [0.0, 0.0] is the leftmost, bottom position of the texture.
    /// [1.0, 1.0] is the rightmost, top position of the texture.
    pub tex_coords: [f32; 2],
    /// A color associated with the `Vertex`.
    ///
    /// The way that the color is used depends on the `mode`.
    pub color: [f32; 4],
}

implement_vertex!(Vertex, position, tex_coords, color, mode);

/// Draw text from the text cache texture `tex` in the fragment shader.
pub const MODE_TEXT: u32 = 0;
/// Draw an image from the texture at `tex` in the fragment shader.
pub const MODE_IMAGE: u32 = 1;
/// Ignore `tex` and draw simple, colored 2D geometry.
pub const MODE_GEOMETRY: u32 = 2;


/// The vertex shader used within the `glium::Program`.
pub const VERTEX_SHADER: &'static str = "
    #version 140

    in vec2 position;
    in vec2 tex_coords;
    in vec4 color;
    in uint mode;

    out vec2 v_tex_coords;
    out vec4 v_color;
    flat out uint v_mode;

    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
        v_tex_coords = tex_coords;
        v_color = color;
        v_mode = mode;
    }
";

/// The fragment shader used within the `glium::Program`.
pub const FRAGMENT_SHADER: &'static str = "
    #version 140
    uniform sampler2D tex;

    in vec2 v_tex_coords;
    in vec4 v_color;
    flat in uint v_mode;

    out vec4 f_color;

    void main() {
        // Text
        if (v_mode == uint(0)) {
            f_color = v_color * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);

        // Image
        } else if (v_mode == uint(1)) {
            f_color = texture(tex, v_tex_coords);

        // 2D Geometry
        } else if (v_mode == uint(2)) {
            f_color = v_color;
        }
    }
";


/// Glium textures that have two dimensions.
pub trait TextureDimensions {
    /// The width and height of the texture.
    fn dimensions(&self) -> (u32, u32);
}

impl<T> TextureDimensions for T
    where T: std::ops::Deref<Target=glium::texture::TextureAny>,
{
    fn dimensions(&self) -> (u32, u32) {
        (self.get_width(), self.get_height().unwrap_or(0))
    }
}


/// Construct the glium shader program that can be used to render `Vertex`es.
pub fn program<F>(facade: &F) -> Result<glium::Program, glium::program::ProgramChooserCreationError>
    where F: glium::backend::Facade,
{
    program!(facade, 140 => { vertex: VERTEX_SHADER, fragment: FRAGMENT_SHADER })
}

/// Default glium `DrawParameters` with alpha blending enabled.
pub fn draw_parameters() -> glium::DrawParameters<'static> {
    let blend = glium::Blend::alpha_blending();
    glium::DrawParameters { blend: blend, ..Default::default() }
}


/// Converts gamma (brightness) from sRGB to linear color space.
///
/// sRGB is the default color space for image editors, pictures, internet etc.
/// Linear gamma yields better results when doing math with colors.
pub fn gamma_srgb_to_linear(c: [f32; 4]) -> [f32; 4] {
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


impl GlyphCache {

    /// Construct a `GlyphCache` with a size equal to the given `Display`'s current framebuffer
    /// dimensions.
    pub fn new<F>(facade: &F) -> Result<Self, glium::texture::TextureCreationError>
        where F: glium::backend::Facade,
    {
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;

        let (w, h) = facade.get_context().get_framebuffer_dimensions();

        // First, the rusttype `Cache` which performs the logic for rendering and laying out glyphs
        // in the cache.
        let cache = text::GlyphCache::new(w, h, SCALE_TOLERANCE, POSITION_TOLERANCE);

        // Now the texture to which glyphs will be rendered.
        let grey_image = glium::texture::RawImage2d {
            data: std::borrow::Cow::Owned(vec![128u8; w as usize * h as usize]),
            width: w,
            height: h,
            format: glium::texture::ClientFormat::U8
        };
        let format = glium::texture::UncompressedFloatFormat::U8;
        let no_mipmap = glium::texture::MipmapsOption::NoMipmap;
        let texture = try!(glium::texture::Texture2d::with_format(facade, grey_image, format, no_mipmap));

        Ok(GlyphCache {
            cache: cache,
            texture: texture,
        })
    }

    /// The texture used to cache the glyphs on the GPU.
    pub fn texture(&self) -> &glium::texture::Texture2d {
        &self.texture
    }

}


impl Renderer {

    /// Construct a new empty `Renderer`.
    pub fn new<F>(facade: &F) -> Result<Self, RendererCreationError>
        where F: glium::backend::Facade,
    {
        let program = try!(program(facade));
        let glyph_cache = try!(GlyphCache::new(facade));
        Ok(Renderer {
            program: program,
            glyph_cache: glyph_cache,
            commands: Vec::new(),
            vertices: Vec::new(),
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
    pub fn fill<P, T>(&mut self,
                      display: &glium::Display,
                      mut primitives: P,
                      image_map: &image::Map<T>)
        where P: render::PrimitiveWalker,
              T: TextureDimensions,
    {
        let Renderer { ref mut commands, ref mut vertices, ref mut glyph_cache, .. } = *self;

        commands.clear();
        vertices.clear();

        enum State {
            Image { id: widget::Id, start: usize },
            Plain { start: usize },
        }

        let mut current_state = State::Plain { start: 0 };

        // Switches to the `Plain` state and completes the previous `Command` if not already in the
        // `Plain` state.
        macro_rules! switch_to_plain_state {
            () => {
                match current_state {
                    State::Plain { .. } => (),
                    State::Image { id, start } => {
                        commands.push(PreparedCommand::Image(id, start..vertices.len()));
                        current_state = State::Plain { start: vertices.len() };
                    },
                }
            };
        }

        // Framebuffer dimensions and the "dots per inch" factor.
        let (screen_w, screen_h) = display.get_framebuffer_dimensions();
        let (win_w, win_h) = (screen_w as Scalar, screen_h as Scalar);
        let dpi_factor = display.get_window().map(|w| w.hidpi_factor()).unwrap_or(1.0) as Scalar;
        let half_win_w = win_w / 2.0;
        let half_win_h = win_h / 2.0;

        // Functions for converting for conrod scalar coords to GL vertex coords (-1.0 to 1.0).
        let vx = |x: Scalar| (x * dpi_factor / half_win_w) as f32;
        let vy = |y: Scalar| (y * dpi_factor / half_win_h) as f32;

        let mut current_scizzor = glium::Rect {
            left: 0,
            width: screen_w,
            bottom: 0,
            height: screen_h,
        };

        let rect_to_glium_rect = |rect: Rect| {
            let (w, h) = rect.w_h();
            let half_w = w / 2.0;
            let half_h = h / 2.0;
            glium::Rect {
                left: ((rect.left() + half_w) * dpi_factor) as u32,
                bottom: ((rect.bottom() + half_h) * dpi_factor) as u32,
                width: (w * dpi_factor) as u32,
                height: (h * dpi_factor) as u32,
            }
        };

        // Draw each primitive in order of depth.
        while let Some(primitive) = primitives.next_primitive() {
            let render::Primitive { id, kind, scizzor, rect } = primitive;

            let new_scizzor = rect_to_glium_rect(scizzor);
            if new_scizzor != current_scizzor {
                // Finish the current command.
                match current_state {
                    State::Plain { start } =>
                        commands.push(PreparedCommand::Plain(start..vertices.len())),
                    State::Image { id, start } =>
                        commands.push(PreparedCommand::Image(id, start..vertices.len())),
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
                            position: [vx(x), vy(y)],
                            tex_coords: [0.0, 0.0],
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

                render::PrimitiveKind::Polygon { color, points } => {
                    // If we don't at least have a triangle, keep looping.
                    if points.len() < 3 {
                        continue;
                    }

                    switch_to_plain_state!();

                    let color = gamma_srgb_to_linear(color.to_fsa());

                    let v = |p: [Scalar; 2]| {
                        Vertex {
                            position: [vx(p[0]), vy(p[1])],
                            tex_coords: [0.0, 0.0],
                            color: color,
                            mode: MODE_GEOMETRY,
                        }
                    };

                    // Triangulate the polygon.
                    //
                    // Make triangles between the first point and every following pair of
                    // points.
                    //
                    // For example, for a polygon with 6 points (a to f), this makes the
                    // following triangles: abc, acd, ade, aef.
                    let first = points[0];
                    let first_v = v(first);
                    let mut prev_v = v(points[1]);
                    for &p in &points[2..] {
                        let v = v(p);
                        vertices.push(first_v);
                        vertices.push(prev_v);
                        vertices.push(v);
                        prev_v = v;
                    }
                },

                render::PrimitiveKind::Lines { color, cap, thickness, points } => {

                    // We need at least two points to draw any lines.
                    if points.len() < 2 {
                        continue;
                    }

                    switch_to_plain_state!();

                    let color = gamma_srgb_to_linear(color.to_fsa());

                    let v = |p: [Scalar; 2]| {
                        Vertex {
                            position: [vx(p[0]), vy(p[1])],
                            tex_coords: [0.0, 0.0],
                            color: color,
                            mode: MODE_GEOMETRY,
                        }
                    };

                    // Convert each line to a rectangle for triangulation.
                    //
                    // TODO: handle `cap` and properly join consecutive lines considering
                    // the miter. Discussion here:
                    // https://forum.libcinder.org/topic/smooth-thick-lines-using-geometry-shader#23286000001269127
                    let mut a = points[0];
                    for &b in &points[1..] {

                        let direction = [b[0] - a[0], b[1] - a[1]];
                        let mag = (direction[0].powi(2) + direction[1].powi(2)).sqrt();
                        let unit = [direction[0] / mag, direction[1] / mag];
                        let normal = [-unit[1], unit[0]];
                        let half_thickness = thickness / 2.0;

                        // A perpendicular line with length half the thickness.
                        let n = [normal[0] * half_thickness, normal[1] * half_thickness];

                        // The corners of the rectangle as GL vertices.
                        let (r1, r2, r3, r4);
                        r1 = v([a[0] + n[0], a[1] + n[1]]);
                        r2 = v([a[0] - n[0], a[1] - n[1]]);
                        r3 = v([b[0] + n[0], b[1] + n[1]]);
                        r4 = v([b[0] - n[0], b[1] - n[1]]);

                        // Push the rectangle's vertices.
                        let mut push_v = |v| vertices.push(v);
                        push_v(r1);
                        push_v(r4);
                        push_v(r2);
                        push_v(r1);
                        push_v(r4);
                        push_v(r3);

                        a = b;
                    }
                },

                render::PrimitiveKind::Text { color, text, font_id } => {
                    switch_to_plain_state!();

                    let positioned_glyphs = text.positioned_glyphs(dpi_factor as f32);

                    let GlyphCache { ref mut cache, ref mut texture } = *glyph_cache;

                    // Queue the glyphs to be cached.
                    for glyph in positioned_glyphs.iter() {
                        cache.queue_glyph(font_id.index(), glyph.clone());
                    }

                    // Cache the glyphs on the GPU.
                    cache.cache_queued(|rect, data| {
                        let glium_rect = glium::Rect {
                            left: rect.min.x,
                            bottom: rect.min.y,
                            width: rect.width(),
                            height: rect.height()
                        };
                        let image = glium::texture::RawImage2d {
                            data: std::borrow::Cow::Borrowed(data),
                            width: rect.width(),
                            height: rect.height(),
                            format: glium::texture::ClientFormat::U8
                        };
                        texture.main_level().write(glium_rect, image);
                    }).unwrap();

                    let color = gamma_srgb_to_linear(color.to_fsa());

                    let cache_id = font_id.index();

                    let origin = text::rt::point(0.0, 0.0);
                    let to_gl_rect = |screen_rect: text::rt::Rect<i32>| text::rt::Rect {
                        min: origin
                            + (text::rt::vector(screen_rect.min.x as f32 / screen_w as f32 - 0.5,
                                          1.0 - screen_rect.min.y as f32 / screen_h as f32 - 0.5)) * 2.0,
                        max: origin
                            + (text::rt::vector(screen_rect.max.x as f32 / screen_w as f32 - 0.5,
                                          1.0 - screen_rect.max.y as f32 / screen_h as f32 - 0.5)) * 2.0
                    };

                    for g in positioned_glyphs {
                        if let Ok(Some((uv_rect, screen_rect))) = cache.rect_for(cache_id, g) {
                            let gl_rect = to_gl_rect(screen_rect);
                            let v = |p, t| Vertex {
                                position: p,
                                tex_coords: t,
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
                },

                render::PrimitiveKind::Image { color, source_rect } => {
                    // Switch to the `Textured` state if we're not in it already.
                    let widget_id = id;
                    match current_state {
                        State::Image { id, .. } if id == widget_id => (),
                        State::Plain { start } => {
                            commands.push(PreparedCommand::Plain(start..vertices.len()));
                            current_state = State::Image { id: id, start: vertices.len() };
                        },
                        State::Image { id, start } => {
                            commands.push(PreparedCommand::Image(id, start..vertices.len()));
                            current_state = State::Image { id: id, start: vertices.len() };
                        },
                    }

                    let color = gamma_srgb_to_linear(color.unwrap_or(color::WHITE).to_fsa());

                    let (image_w, image_h) = image_map.get(&id).unwrap().dimensions();
                    let (image_w, image_h) = (image_w as Scalar, image_h as Scalar);

                    // Get the sides of the source rectangle as uv coordinates.
                    //
                    // Texture coordinates range:
                    // - left to right: 0.0 to 1.0
                    // - bottom to top: 0.0 to 1.0
                    let (uv_l, uv_r, uv_b, uv_t) = match source_rect {
                        Some(src_rect) => {
                            let (l, r, b, t) = src_rect.l_r_b_t();
                            ((l / image_w) as f32,
                             (r / image_w) as f32,
                             (b / image_h) as f32,
                             (t / image_h) as f32)
                        },
                        None => (0.0, 1.0, 0.0, 1.0),
                    };

                    let v = |x, y, t| {
                        // Convert from conrod Scalar range to GL range -1.0 to 1.0.
                        let x = (x * dpi_factor as Scalar / half_win_w) as f32;
                        let y = (y * dpi_factor as Scalar / half_win_h) as f32;
                        Vertex {
                            position: [x, y],
                            tex_coords: t,
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
                },

                // We have no special case widgets to handle.
                render::PrimitiveKind::Other(_) => (),
            }

        }

        // Enter the final command.
        match current_state {
            State::Plain { start } =>
                commands.push(PreparedCommand::Plain(start..vertices.len())),
            State::Image { id, start } =>
                commands.push(PreparedCommand::Image(id, start..vertices.len())),
        }
    }

    /// Draws using the inner list of `Command`s to the given `display`.
    ///
    /// Note: If you require more granular control over rendering, you may want to use the `fill`
    /// and `commands` methods separately. This method is simply a convenience wrapper around those
    /// methods for the case that the user does not require accessing or modifying conrod's draw
    /// parameters, uniforms or generated draw commands.
    pub fn draw<F, S, T>(&self, facade: &F, surface: &mut S, image_map: &image::Map<T>)
        -> Result<(), DrawError>
        where F: glium::backend::Facade,
              S: glium::Surface,
              for<'a> glium::uniforms::Sampler<'a, T>: glium::uniforms::AsUniformValue,
    {
        let mut draw_params = draw_parameters();
        let no_indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let uniforms = uniform! {
            tex: self.glyph_cache.texture()
                .sampled()
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        };

        for command in self.commands() {
            match command {

                // Update the `scizzor` before continuing to draw.
                Command::Scizzor(scizzor) => draw_params.scissor = Some(scizzor),

                // Draw to the target with the given `draw` command.
                Command::Draw(draw) => match draw {

                    // Draw text and plain 2D geometry.
                    Draw::Plain(slice) => {
                        let vertex_buffer = try!(glium::VertexBuffer::new(facade, slice));
                        surface.draw(&vertex_buffer, no_indices, &self.program, &uniforms, &draw_params).unwrap();
                    },

                    // Draw an image whose texture data lies within the `image_map` at the
                    // given `id`.
                    Draw::Image(id, slice) => {
                        let vertex_buffer = glium::VertexBuffer::new(facade, slice).unwrap();
                        let image = image_map.get(&id).unwrap();
                        let image_uniforms = uniform! {
                            tex: glium::uniforms::Sampler::new(image)
                                .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                        };
                        surface.draw(&vertex_buffer, no_indices, &self.program, &image_uniforms, &draw_params).unwrap();
                    },

                }
            }
        }

        Ok(())
    }

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

impl From<glium::texture::TextureCreationError> for RendererCreationError {
    fn from(err: glium::texture::TextureCreationError) -> Self {
        RendererCreationError::Texture(err)
    }
}

impl From<glium::program::ProgramChooserCreationError> for RendererCreationError {
    fn from(err: glium::program::ProgramChooserCreationError) -> Self {
        RendererCreationError::Program(err)
    }
}

impl From<glium::vertex::BufferCreationError> for DrawError {
    fn from(err: glium::vertex::BufferCreationError) -> Self {
        DrawError::Buffer(err)
    }
}

impl From<glium::DrawError> for DrawError {
    fn from(err: glium::DrawError) -> Self {
        DrawError::Draw(err)
    }
}
