//! A glium backend for rendering conrod primitives.

extern crate conrod_core;
#[macro_use]
extern crate glium;

use conrod_core::{color, image, render, text, Rect, Scalar};

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
    Image(image::Id, &'a [Vertex]),
    /// A range of vertices representing plain triangles.
    Plain(&'a [Vertex]),
}

enum PreparedCommand {
    Image(image::Id, std::ops::Range<usize>),
    Plain(std::ops::Range<usize>),
    Scizzor(glium::Rect),
}

/// A rusttype `GlyphCache` along with a `glium::texture::Texture2d` for caching text on the `GPU`.
pub struct GlyphCache {
    cache: text::GlyphCache<'static>,
    texture: glium::texture::Texture2d,
}

/// A type used for translating `render::Primitives` into `Command`s that indicate how to draw the
/// conrod GUI using `glium`.
pub struct Renderer {
    program: glium::Program,
    glyph_cache: GlyphCache,
    commands: Vec<PreparedCommand>,
    vertices: Vec<Vertex>,
    positioned_glyphs: Vec<text::PositionedGlyph>,
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

#[allow(unsafe_code)]
mod vertex_impl {
    use super::Vertex;
    implement_vertex!(Vertex, position, tex_coords, color, mode);
}

/// Draw text from the text cache texture `tex` in the fragment shader.
pub const MODE_TEXT: u32 = 0;
/// Draw an image from the texture at `tex` in the fragment shader.
pub const MODE_IMAGE: u32 = 1;
/// Ignore `tex` and draw simple, colored 2D geometry.
pub const MODE_GEOMETRY: u32 = 2;

/// The vertex shader used within the `glium::Program` for OpenGL.
pub const VERTEX_SHADER_120: &'static str = "
    #version 120

    attribute vec2 position;
    attribute vec2 tex_coords;
    attribute vec4 color;
    attribute float mode;

    varying vec2 v_tex_coords;
    varying vec4 v_color;
    varying float v_mode;

    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
        v_tex_coords = tex_coords;
        v_color = color;
        v_mode = mode;
    }
";

/// The fragment shader used within the `glium::Program` for OpenGL.
pub const FRAGMENT_SHADER_120: &'static str = "
    #version 120
    uniform sampler2D tex;

    varying vec2 v_tex_coords;
    varying vec4 v_color;
    varying float v_mode;

    void main() {
        // Text
        if (v_mode == 0.0) {
            gl_FragColor = v_color * vec4(1.0, 1.0, 1.0, texture2D(tex, v_tex_coords).r);

        // Image
        } else if (v_mode == 1.0) {
            gl_FragColor = texture2D(tex, v_tex_coords);

        // 2D Geometry
        } else if (v_mode == 2.0) {
            gl_FragColor = v_color;
        }
    }
";

/// The vertex shader used within the `glium::Program` for OpenGL.
pub const VERTEX_SHADER_140: &'static str = "
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

/// The fragment shader used within the `glium::Program` for OpenGL.
pub const FRAGMENT_SHADER_140: &'static str = "
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

/// The vertex shader used within the `glium::Program` for OpenGL ES.
pub const VERTEX_SHADER_300_ES: &'static str = "
    #version 300 es
    precision mediump float;

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

/// The fragment shader used within the `glium::Program` for OpenGL ES.
pub const FRAGMENT_SHADER_300_ES: &'static str = "
    #version 300 es
    precision mediump float;
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
where
    T: std::ops::Deref<Target = glium::texture::TextureAny>,
{
    fn dimensions(&self) -> (u32, u32) {
        (self.get_width(), self.get_height().unwrap_or(0))
    }
}

/// Construct the glium shader program that can be used to render `Vertex`es.
pub fn program<F>(facade: &F) -> Result<glium::Program, glium::program::ProgramChooserCreationError>
where
    F: glium::backend::Facade,
{
    program!(facade,
             120 => { vertex: VERTEX_SHADER_120, fragment: FRAGMENT_SHADER_120 },
             140 => { vertex: VERTEX_SHADER_140, fragment: FRAGMENT_SHADER_140 },
             300 es => { vertex: VERTEX_SHADER_300_ES, fragment: FRAGMENT_SHADER_300_ES })
}

/// Default glium `DrawParameters` with alpha blending enabled.
pub fn draw_parameters() -> glium::DrawParameters<'static> {
    use glium::{Blend, BlendingFunction, LinearBlendingFactor};
    let blend = Blend {
        color: BlendingFunction::Addition {
            source: LinearBlendingFactor::SourceAlpha,
            destination: LinearBlendingFactor::OneMinusSourceAlpha,
        },
        alpha: BlendingFunction::Addition {
            source: LinearBlendingFactor::One,
            destination: LinearBlendingFactor::OneMinusSourceAlpha,
        },
        constant_value: (0.0, 0.0, 0.0, 0.0),
    };
    glium::DrawParameters {
        multisampling: true,
        blend: blend,
        ..Default::default()
    }
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

/// Return the optimal client format for the text texture given the version.
pub fn text_texture_client_format(opengl_version: &glium::Version) -> glium::texture::ClientFormat {
    match *opengl_version {
        // If the version is greater than or equal to GL 3.0 or GLes 3.0, we can use the `U8` format.
        glium::Version(_, major, _) if major >= 3 => glium::texture::ClientFormat::U8,
        // Otherwise, we must use the `U8U8U8` format to support older versions.
        _ => glium::texture::ClientFormat::U8U8U8,
    }
}

/// Return the optimal uncompressed float format for the text texture given the version.
pub fn text_texture_uncompressed_float_format(
    opengl_version: &glium::Version,
) -> glium::texture::UncompressedFloatFormat {
    match *opengl_version {
        // If the version is greater than or equal to GL 3.0 or GLes 3.0, we can use the `U8` format.
        glium::Version(_, major, _) if major >= 3 => glium::texture::UncompressedFloatFormat::U8,
        // Otherwise, we must use the `U8U8U8` format to support older versions.
        _ => glium::texture::UncompressedFloatFormat::U8U8U8,
    }
}

// Creating the rusttype glyph cache used within a `GlyphCache`.
fn rusttype_glyph_cache(w: u32, h: u32) -> text::GlyphCache<'static> {
    const SCALE_TOLERANCE: f32 = 0.1;
    const POSITION_TOLERANCE: f32 = 0.1;
    text::GlyphCache::builder()
        .dimensions(w, h)
        .scale_tolerance(SCALE_TOLERANCE)
        .position_tolerance(POSITION_TOLERANCE)
        .build()
}

// Create the texture used within a `GlyphCache` of the given size.
fn glyph_cache_texture<F>(
    facade: &F,
    width: u32,
    height: u32,
) -> Result<glium::texture::Texture2d, glium::texture::TextureCreationError>
where
    F: glium::backend::Facade,
{
    // Determine the optimal texture format to use given the opengl version.
    let context = facade.get_context();
    let opengl_version = context.get_opengl_version();
    let client_format = text_texture_client_format(opengl_version);
    let uncompressed_float_format = text_texture_uncompressed_float_format(opengl_version);
    let num_components = client_format.get_num_components() as u32;
    let data_size = num_components as usize * width as usize * height as usize;
    let data = std::borrow::Cow::Owned(vec![128u8; data_size]);
    let grey_image = glium::texture::RawImage2d {
        data: data,
        width: width,
        height: height,
        format: client_format,
    };
    let format = uncompressed_float_format;
    let no_mipmap = glium::texture::MipmapsOption::NoMipmap;
    glium::texture::Texture2d::with_format(facade, grey_image, format, no_mipmap)
}

impl GlyphCache {
    /// Construct a **GlyphCache** with the given texture dimensions.
    ///
    /// When calling `GlyphCache::new`, the `get_framebuffer_dimensions` method is used to produce
    /// the width and height. However, often creating a texture the size of the screen might not be
    /// large enough to cache the necessary text for an application. The following constant
    /// multiplier is used to ensure plenty of room in the cache.
    pub fn with_dimensions<F>(
        facade: &F,
        width: u32,
        height: u32,
    ) -> Result<Self, glium::texture::TextureCreationError>
    where
        F: glium::backend::Facade,
    {
        // First, the rusttype `Cache` which performs the logic for rendering and laying out glyphs
        // in the cache.
        let cache = rusttype_glyph_cache(width, height);

        // Now the texture to which glyphs will be rendered.
        let texture = glyph_cache_texture(facade, width, height)?;

        Ok(GlyphCache {
            cache: cache,
            texture: texture,
        })
    }

    /// Construct a `GlyphCache` with a size equal to the given `Display`'s current framebuffer
    /// dimensions.
    pub fn new<F>(facade: &F) -> Result<Self, glium::texture::TextureCreationError>
    where
        F: glium::backend::Facade,
    {
        let (w, h) = facade.get_context().get_framebuffer_dimensions();
        Self::with_dimensions(facade, w, h)
    }

    /// The texture used to cache the glyphs on the GPU.
    pub fn texture(&self) -> &glium::texture::Texture2d {
        &self.texture
    }
}

pub trait Display {
    fn opengl_version(&self) -> &glium::Version;
    fn framebuffer_dimensions(&self) -> (u32, u32);
    fn hidpi_factor(&self) -> f64;
}

impl Display for glium::Display {
    fn opengl_version(&self) -> &glium::Version {
        self.get_opengl_version()
    }

    fn framebuffer_dimensions(&self) -> (u32, u32) {
        self.get_framebuffer_dimensions()
    }

    fn hidpi_factor(&self) -> f64 {
        self.gl_window().window().scale_factor()
    }
}

impl Renderer {
    /// Construct a new empty `Renderer`.
    ///
    /// The dimensions of the inner glyph cache will be equal to the dimensions of the given
    /// facade's framebuffer.
    pub fn new<F>(facade: &F) -> Result<Self, RendererCreationError>
    where
        F: glium::backend::Facade,
    {
        let glyph_cache = GlyphCache::new(facade)?;
        Self::with_glyph_cache(facade, glyph_cache)
    }

    /// Construct a new empty `Renderer` with the given glyph cache dimensions.
    pub fn with_glyph_cache_dimensions<F>(
        facade: &F,
        width: u32,
        height: u32,
    ) -> Result<Self, RendererCreationError>
    where
        F: glium::backend::Facade,
    {
        let glyph_cache = GlyphCache::with_dimensions(facade, width, height)?;
        Self::with_glyph_cache(facade, glyph_cache)
    }

    // Construct a new **Renderer** that uses the given glyph cache for caching text.
    fn with_glyph_cache<F>(facade: &F, gc: GlyphCache) -> Result<Self, RendererCreationError>
    where
        F: glium::backend::Facade,
    {
        let program = program(facade)?;
        Ok(Renderer {
            program: program,
            glyph_cache: gc,
            commands: Vec::new(),
            vertices: Vec::new(),
            positioned_glyphs: Vec::new(),
        })
    }

    /// Produce an `Iterator` yielding `Command`s.
    pub fn commands(&self) -> Commands {
        let Renderer {
            ref commands,
            ref vertices,
            ..
        } = *self;
        Commands {
            commands: commands.iter(),
            vertices: vertices,
        }
    }

    /// Fill the inner vertex and command buffers by translating the given `primitives`.
    pub fn fill<D, P, T>(&mut self, display: &D, mut primitives: P, image_map: &image::Map<T>)
    where
        P: render::PrimitiveWalker,
        D: Display,
        T: TextureDimensions,
    {
        let Renderer {
            ref mut commands,
            ref mut vertices,
            ref mut glyph_cache,
            ref mut positioned_glyphs,
            ..
        } = *self;

        commands.clear();
        vertices.clear();

        // This is necessary for supporting rusttype's GPU cache with OpenGL versions older than GL
        // 3.0 and GL ES 3.0. It is used to convert from the `U8` data format given by `rusttype`
        // to the `U8U8U8` format that is necessary for older versions of OpenGL.
        //
        // The buffer is only used if an older version was detected, otherwise the text GPU cache
        // uses the rusttype `data` buffer directly.
        let mut text_data_u8u8u8 = Vec::new();

        // Determine the texture format that we're using.
        let opengl_version = display.opengl_version();
        let client_format = text_texture_client_format(opengl_version);

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
                        current_state = State::Plain {
                            start: vertices.len(),
                        };
                    }
                }
            };
        }

        // Framebuffer dimensions and the "dots per inch" factor.
        let (screen_w, screen_h) = display.framebuffer_dimensions();
        let (win_w, win_h) = (screen_w as Scalar, screen_h as Scalar);
        let half_win_w = win_w / 2.0;
        let half_win_h = win_h / 2.0;
        let dpi_factor = display.hidpi_factor() as Scalar;

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
            let left = (rect.left() * dpi_factor + half_win_w).round() as u32;
            let bottom = (rect.bottom() * dpi_factor + half_win_h).round() as u32;
            let width = (w * dpi_factor).round() as u32;
            let height = (h * dpi_factor).round() as u32;
            glium::Rect {
                left: std::cmp::max(left, 0),
                bottom: std::cmp::max(bottom, 0),
                width: std::cmp::min(width, screen_w),
                height: std::cmp::min(height, screen_h),
            }
        };

        // Draw each primitive in order of depth.
        while let Some(primitive) = primitives.next_primitive() {
            let render::Primitive {
                kind,
                scizzor,
                rect,
                ..
            } = primitive;

            // Check for a `Scizzor` command.
            let new_scizzor = rect_to_glium_rect(scizzor);
            if new_scizzor != current_scizzor {
                // Finish the current command.
                match current_state {
                    State::Plain { start } => {
                        commands.push(PreparedCommand::Plain(start..vertices.len()))
                    }
                    State::Image { image_id, start } => {
                        commands.push(PreparedCommand::Image(image_id, start..vertices.len()))
                    }
                }

                // Update the scizzor and produce a command.
                current_scizzor = new_scizzor;
                commands.push(PreparedCommand::Scizzor(new_scizzor));

                // Set the state back to plain drawing.
                current_state = State::Plain {
                    start: vertices.len(),
                };
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
                }

                render::PrimitiveKind::TrianglesSingleColor { color, triangles } => {
                    if triangles.is_empty() {
                        continue;
                    }

                    switch_to_plain_state!();

                    let color = gamma_srgb_to_linear(color.into());

                    let v = |p: [Scalar; 2]| Vertex {
                        position: [vx(p[0]), vy(p[1])],
                        tex_coords: [0.0, 0.0],
                        color: color,
                        mode: MODE_GEOMETRY,
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

                    let v = |(p, c): ([Scalar; 2], color::Rgba)| Vertex {
                        position: [vx(p[0]), vy(p[1])],
                        tex_coords: [0.0, 0.0],
                        color: gamma_srgb_to_linear(c.into()),
                        mode: MODE_GEOMETRY,
                    };

                    for triangle in triangles {
                        vertices.push(v(triangle[0]));
                        vertices.push(v(triangle[1]));
                        vertices.push(v(triangle[2]));
                    }
                }

                render::PrimitiveKind::Text {
                    color,
                    text,
                    font_id,
                } => {
                    switch_to_plain_state!();

                    positioned_glyphs.clear();
                    positioned_glyphs.extend(text.positioned_glyphs(dpi_factor as f32));

                    let GlyphCache {
                        ref mut cache,
                        ref mut texture,
                    } = *glyph_cache;

                    // Queue the glyphs to be cached.
                    for glyph in positioned_glyphs.iter() {
                        cache.queue_glyph(font_id.index(), glyph.clone());
                    }

                    // Cache the glyphs on the GPU.
                    cache
                        .cache_queued(|rect, data| {
                            let w = rect.width();
                            let h = rect.height();
                            let glium_rect = glium::Rect {
                                left: rect.min.x,
                                bottom: rect.min.y,
                                width: w,
                                height: h,
                            };

                            let data = match client_format {
                                // `rusttype` gives data in the `U8` format so we can use it directly.
                                glium::texture::ClientFormat::U8 => {
                                    std::borrow::Cow::Borrowed(data)
                                }
                                // Otherwise we have to convert to the supported format.
                                glium::texture::ClientFormat::U8U8U8 => {
                                    text_data_u8u8u8.clear();
                                    for &b in data.iter() {
                                        text_data_u8u8u8.push(b);
                                        text_data_u8u8u8.push(b);
                                        text_data_u8u8u8.push(b);
                                    }
                                    std::borrow::Cow::Borrowed(&text_data_u8u8u8[..])
                                }
                                // The text cache is only ever created with U8 or U8U8U8 formats.
                                _ => unreachable!(),
                            };

                            let image = glium::texture::RawImage2d {
                                data: data,
                                width: w,
                                height: h,
                                format: client_format,
                            };
                            texture.main_level().write(glium_rect, image);
                        })
                        .unwrap();

                    let color = gamma_srgb_to_linear(color.to_fsa());

                    let cache_id = font_id.index();

                    let origin = text::rt::point(0.0, 0.0);
                    let to_gl_rect = |screen_rect: text::rt::Rect<i32>| text::rt::Rect {
                        min: origin
                            + (text::rt::vector(
                                screen_rect.min.x as f32 / screen_w as f32 - 0.5,
                                1.0 - screen_rect.min.y as f32 / screen_h as f32 - 0.5,
                            )) * 2.0,
                        max: origin
                            + (text::rt::vector(
                                screen_rect.max.x as f32 / screen_w as f32 - 0.5,
                                1.0 - screen_rect.max.y as f32 / screen_h as f32 - 0.5,
                            )) * 2.0,
                    };

                    for g in positioned_glyphs.drain(..) {
                        if let Ok(Some((uv_rect, screen_rect))) = cache.rect_for(cache_id, &g) {
                            let gl_rect = to_gl_rect(screen_rect);
                            let v = |p, t| Vertex {
                                position: p,
                                tex_coords: t,
                                color: color,
                                mode: MODE_TEXT,
                            };
                            let mut push_v = |p, t| vertices.push(v(p, t));
                            push_v(
                                [gl_rect.min.x, gl_rect.max.y],
                                [uv_rect.min.x, uv_rect.max.y],
                            );
                            push_v(
                                [gl_rect.min.x, gl_rect.min.y],
                                [uv_rect.min.x, uv_rect.min.y],
                            );
                            push_v(
                                [gl_rect.max.x, gl_rect.min.y],
                                [uv_rect.max.x, uv_rect.min.y],
                            );
                            push_v(
                                [gl_rect.max.x, gl_rect.min.y],
                                [uv_rect.max.x, uv_rect.min.y],
                            );
                            push_v(
                                [gl_rect.max.x, gl_rect.max.y],
                                [uv_rect.max.x, uv_rect.max.y],
                            );
                            push_v(
                                [gl_rect.min.x, gl_rect.max.y],
                                [uv_rect.min.x, uv_rect.max.y],
                            );
                        }
                    }
                }

                render::PrimitiveKind::Image {
                    image_id,
                    color,
                    source_rect,
                } => {
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

                    if let Some(image) = image_map.get(&image_id) {
                        let (image_w, image_h) = image.dimensions();
                        let (image_w, image_h) = (image_w as Scalar, image_h as Scalar);

                        // Get the sides of the source rectangle as uv coordinates.
                        //
                        // Texture coordinates range:
                        // - left to right: 0.0 to 1.0
                        // - bottom to top: 0.0 to 1.0
                        let (uv_l, uv_r, uv_b, uv_t) = match source_rect {
                            Some(src_rect) => {
                                let (l, r, b, t) = src_rect.l_r_b_t();
                                (
                                    (l / image_w) as f32,
                                    (r / image_w) as f32,
                                    (b / image_h) as f32,
                                    (t / image_h) as f32,
                                )
                            }
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
                    }
                }

                // We have no special case widgets to handle.
                render::PrimitiveKind::Other(_) => (),
            }
        }

        // Enter the final command.
        match current_state {
            State::Plain { start } => commands.push(PreparedCommand::Plain(start..vertices.len())),
            State::Image { image_id, start } => {
                commands.push(PreparedCommand::Image(image_id, start..vertices.len()))
            }
        }
    }

    /// Draws using the inner list of `Command`s to the given `display`.
    ///
    /// Note: If you require more granular control over rendering, you may want to use the `fill`
    /// and `commands` methods separately. This method is simply a convenience wrapper around those
    /// methods for the case that the user does not require accessing or modifying conrod's draw
    /// parameters, uniforms or generated draw commands.
    pub fn draw<F, S, T>(
        &self,
        facade: &F,
        surface: &mut S,
        image_map: &image::Map<T>,
    ) -> Result<(), DrawError>
    where
        F: glium::backend::Facade,
        S: glium::Surface,
        for<'a> glium::uniforms::Sampler<'a, T>: glium::uniforms::AsUniformValue,
    {
        let mut draw_params = draw_parameters();
        let no_indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let uniforms = uniform! {
            tex: self.glyph_cache.texture()
                .sampled()
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                .minify_filter(glium::uniforms::MinifySamplerFilter::Linear)
        };

        const NUM_VERTICES_IN_TRIANGLE: usize = 3;

        for command in self.commands() {
            match command {
                // Update the `scizzor` before continuing to draw.
                Command::Scizzor(scizzor) => draw_params.scissor = Some(scizzor),

                // Draw to the target with the given `draw` command.
                Command::Draw(draw) => match draw {
                    // Draw text and plain 2D geometry.
                    //
                    // Only submit the vertices if there is enough for at least one triangle.
                    Draw::Plain(slice) => {
                        if slice.len() >= NUM_VERTICES_IN_TRIANGLE {
                            let vertex_buffer = glium::VertexBuffer::new(facade, slice)?;
                            surface
                                .draw(
                                    &vertex_buffer,
                                    no_indices,
                                    &self.program,
                                    &uniforms,
                                    &draw_params,
                                )
                                .unwrap();
                        }
                    }

                    // Draw an image whose texture data lies within the `image_map` at the
                    // given `id`.
                    //
                    // Only submit the vertices if there is enough for at least one triangle.
                    Draw::Image(image_id, slice) => {
                        if slice.len() >= NUM_VERTICES_IN_TRIANGLE {
                            let vertex_buffer = glium::VertexBuffer::new(facade, slice).unwrap();
                            if let Some(image) = image_map.get(&image_id) {
                                let image_uniforms = uniform! {
                                    tex: glium::uniforms::Sampler::new(image)
                                        .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                                        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                                };
                                surface
                                    .draw(
                                        &vertex_buffer,
                                        no_indices,
                                        &self.program,
                                        &image_uniforms,
                                        &draw_params,
                                    )
                                    .unwrap();
                            }
                        }
                    }
                },
            }
        }

        Ok(())
    }
}

impl<'a> Iterator for Commands<'a> {
    type Item = Command<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let Commands {
            ref mut commands,
            ref vertices,
        } = *self;
        commands.next().map(|command| match *command {
            PreparedCommand::Scizzor(scizzor) => Command::Scizzor(scizzor),
            PreparedCommand::Plain(ref range) => {
                Command::Draw(Draw::Plain(&vertices[range.clone()]))
            }
            PreparedCommand::Image(id, ref range) => {
                Command::Draw(Draw::Image(id, &vertices[range.clone()]))
            }
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

impl std::error::Error for RendererCreationError {}

impl std::fmt::Display for RendererCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            RendererCreationError::Texture(ref e) => std::fmt::Display::fmt(e, f),
            RendererCreationError::Program(ref e) => std::fmt::Display::fmt(e, f),
        }
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

impl std::error::Error for DrawError {}

impl std::fmt::Display for DrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            DrawError::Buffer(ref e) => std::fmt::Display::fmt(e, f),
            DrawError::Draw(ref e) => std::fmt::Display::fmt(e, f),
        }
    }
}
