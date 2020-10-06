//! A mesh type dedicated to converting sequences of `render::Primitive`s to a representation in
//! vertices ready for uploading to the GPU.
//!
//! While populating the vertices buffer ready for uploading to the GPU, the `Mesh` will also
//! produce a sequence of commands describing the order in which draw commands should occur and
//! whether or not the `Scizzor` should be updated between draws.

use crate::text::{self, rt};
use crate::{color, image, render};
use crate::{Rect, Scalar};
use std::{fmt, ops};

/// Images within the given image map must know their dimensions in pixels.
pub trait ImageDimensions {
    /// The dimensions of the image in pixels.
    fn dimensions(&self) -> [u32; 2];
}

/// A mesh whose vertices may be populated by a list of render primitives.
///
/// This is a convenience type for simplifying backend implementations.
#[derive(Debug)]
pub struct Mesh {
    glyph_cache: GlyphCache,
    glyph_cache_pixel_buffer: Vec<u8>,
    commands: Vec<PreparedCommand>,
    vertices: Vec<Vertex>,
}

/// Represents the scizzor in pixel coordinates.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Scizzor {
    /// The top left of the scizzor rectangle, where the top-left corner of the viewport is [0, 0].
    pub top_left: [i32; 2],
    /// The dimensions of the `Scizzor` rect.
    pub dimensions: [u32; 2],
}

/// A `Command` describing a step in the drawing process.
#[derive(Clone, Debug)]
pub enum Command {
    /// Draw to the target.
    Draw(Draw),
    /// Update the scizzor within the pipeline.
    Scizzor(Scizzor),
}

/// An iterator yielding `Command`s, produced by the `Renderer::commands` method.
pub struct Commands<'a> {
    commands: std::slice::Iter<'a, PreparedCommand>,
}

/// A `Command` for drawing to the target.
///
/// Each variant describes how to draw the contents of the vertex buffer.
#[derive(Clone, Debug)]
pub enum Draw {
    /// A range of vertices representing triangles textured with the image in the
    /// image_map at the given `widget::Id`.
    Image(image::Id, std::ops::Range<usize>),
    /// A range of vertices representing plain triangles.
    Plain(std::ops::Range<usize>),
}

/// The data associated with a single vertex.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Vertex {
    /// The normalised position of the vertex within vector space.
    ///
    /// [-1.0, 1.0] is the leftmost, bottom position of the display.
    /// [1.0, -1.0] is the rightmost, top position of the display.
    pub position: [f32; 2],
    /// The coordinates of the texture used by this `Vertex`.
    ///
    /// [0.0, 0.0] is the leftmost, top position of the texture.
    /// [1.0, 1.0] is the rightmost, bottom position of the texture.
    pub tex_coords: [f32; 2],
    /// Linear sRGB with an alpha channel.
    pub rgba: [f32; 4],
    /// The mode with which the `Vertex` will be drawn within the fragment shader.
    ///
    /// `0` for rendering text.
    /// `1` for rendering an image.
    /// `2` for rendering non-textured 2D geometry.
    ///
    /// If any other value is given, the fragment shader will not output any color.
    pub mode: u32,
}

/// The result of filling the mesh.
///
/// Provides information on whether or not the glyph cache has been updated and requires
/// re-uploading to the GPU.
#[allow(missing_copy_implementations)]
pub struct Fill {
    /// Whether or not the glyph cache pixel data should be written to the GPU.
    pub glyph_cache_requires_upload: bool,
}

// A wrapper around an owned glyph cache, providing `Debug` and `Deref` impls.
struct GlyphCache(text::GlyphCache<'static>);

#[derive(Debug)]
enum PreparedCommand {
    Image(image::Id, std::ops::Range<usize>),
    Plain(std::ops::Range<usize>),
    Scizzor(Scizzor),
}

/// Draw text from the text cache texture `tex` in the fragment shader.
pub const MODE_TEXT: u32 = 0;
/// Draw an image from the texture at `tex` in the fragment shader.
pub const MODE_IMAGE: u32 = 1;
/// Ignore `tex` and draw simple, colored 2D geometry.
pub const MODE_GEOMETRY: u32 = 2;

/// Default dimensions to use for the glyph cache.
pub const DEFAULT_GLYPH_CACHE_DIMS: [u32; 2] = [1_024; 2];

impl Mesh {
    /// Construct a new empty `Mesh` with default glyph cache dimensions.
    pub fn new() -> Self {
        Self::with_glyph_cache_dimensions(DEFAULT_GLYPH_CACHE_DIMS)
    }

    /// Construct a `Mesh` with the given glyph cache dimensions.
    pub fn with_glyph_cache_dimensions(glyph_cache_dims: [u32; 2]) -> Self {
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;
        let [gc_width, gc_height] = glyph_cache_dims;
        let glyph_cache = text::GlyphCache::builder()
            .dimensions(gc_width, gc_height)
            .scale_tolerance(SCALE_TOLERANCE)
            .position_tolerance(POSITION_TOLERANCE)
            .build()
            .into();
        let glyph_cache_pixel_buffer = vec![0u8; gc_width as usize * gc_height as usize];
        let commands = vec![];
        let vertices = vec![];
        Mesh {
            glyph_cache,
            glyph_cache_pixel_buffer,
            commands,
            vertices,
        }
    }

    /// Fill the inner vertex buffer from the given primitives.
    ///
    /// - `viewport`: the window in which the UI is drawn. The width and height should be the
    ///   physical size (pixels).
    /// - `dpi_factor`: the factor for converting from conrod's DPI agnostic point space to the
    ///   pixel space of the viewport.
    /// - `image_map`: a map from image IDs to images.
    /// - `primitives`: the sequence of UI primitives in order of depth to be rendered.
    pub fn fill<P, I>(
        &mut self,
        viewport: Rect,
        dpi_factor: f64,
        image_map: &image::Map<I>,
        mut primitives: P,
    ) -> Result<Fill, rt::gpu_cache::CacheWriteErr>
    where
        P: render::PrimitiveWalker,
        I: ImageDimensions,
    {
        let Mesh {
            ref mut glyph_cache,
            ref mut glyph_cache_pixel_buffer,
            ref mut commands,
            ref mut vertices,
        } = *self;

        commands.clear();
        vertices.clear();

        enum State {
            Image { image_id: image::Id, start: usize },
            Plain { start: usize },
        }

        let mut current_state = State::Plain { start: 0 };

        // Keep track of whether or not the glyph cache texture needs to be updated.
        let mut glyph_cache_requires_upload = false;

        // Viewport dimensions and the "dots per inch" factor.
        let (viewport_w, viewport_h) = viewport.w_h();
        let half_viewport_w = viewport_w / 2.0;
        let half_viewport_h = viewport_h / 2.0;

        // Width of the glyph cache is useful when writing to the pixel buffer.
        let (glyph_cache_w, _) = glyph_cache.dimensions();
        let glyph_cache_w = glyph_cache_w as usize;

        // Functions for converting for conrod scalar coords to normalised vertex coords (-1.0 to 1.0).
        let vx = |x: Scalar| (x * dpi_factor / half_viewport_w) as f32;
        let vy = |y: Scalar| -1.0 * (y * dpi_factor / half_viewport_h) as f32;

        let rect_to_scizzor = |rect: Rect| {
            let (w, h) = rect.w_h();
            let left = (rect.left() * dpi_factor + half_viewport_w).round() as i32;
            let top = (rect.top() * dpi_factor - half_viewport_h).round().abs() as i32;
            let width = (w * dpi_factor).round() as u32;
            let height = (h * dpi_factor).round() as u32;
            Scizzor {
                top_left: [left.max(0), top.max(0)],
                dimensions: [width.min(viewport_w as u32), height.min(viewport_h as u32)],
            }
        };

        // Keep track of the scizzor as it changes.
        let mut current_scizzor = rect_to_scizzor(viewport);

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

        // Draw each primitive in order of depth.
        while let Some(primitive) = primitives.next_primitive() {
            let render::Primitive {
                kind,
                scizzor,
                rect,
                ..
            } = primitive;

            // Check for a `Scizzor` command.
            let new_scizzor = rect_to_scizzor(scizzor);
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
                            rgba: color,
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
                        rgba: color,
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
                        rgba: gamma_srgb_to_linear(c.into()),
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

                    let positioned_glyphs = text.positioned_glyphs(dpi_factor as f32);

                    // Queue the glyphs to be cached
                    for glyph in positioned_glyphs {
                        glyph_cache.queue_glyph(font_id.index(), glyph.clone());
                    }

                    glyph_cache.cache_queued(|rect, data| {
                        let width = (rect.max.x - rect.min.x) as usize;
                        let height = (rect.max.y - rect.min.y) as usize;
                        let mut dst_ix = rect.min.y as usize * glyph_cache_w + rect.min.x as usize;
                        let mut src_ix = 0;
                        for _ in 0..height {
                            let dst_range = dst_ix..dst_ix + width;
                            let src_range = src_ix..src_ix + width;
                            let dst_slice = &mut glyph_cache_pixel_buffer[dst_range];
                            let src_slice = &data[src_range];
                            dst_slice.copy_from_slice(src_slice);
                            dst_ix += glyph_cache_w;
                            src_ix += width;
                        }
                        glyph_cache_requires_upload = true;
                    })?;

                    let color = gamma_srgb_to_linear(color.to_fsa());
                    let cache_id = font_id.index();
                    let origin = rt::point(0.0, 0.0);

                    // A closure to convert RustType rects to GL rects
                    let to_vk_rect = |screen_rect: rt::Rect<i32>| rt::Rect {
                        min: origin
                            + (rt::vector(
                                screen_rect.min.x as f32 / viewport_w as f32 - 0.5,
                                screen_rect.min.y as f32 / viewport_h as f32 - 0.5,
                            )) * 2.0,
                        max: origin
                            + (rt::vector(
                                screen_rect.max.x as f32 / viewport_w as f32 - 0.5,
                                screen_rect.max.y as f32 / viewport_h as f32 - 0.5,
                            )) * 2.0,
                    };

                    for g in positioned_glyphs {
                        if let Ok(Some((uv_rect, screen_rect))) = glyph_cache.rect_for(cache_id, g)
                        {
                            let vk_rect = to_vk_rect(screen_rect);
                            let v = |p, t| Vertex {
                                position: p,
                                tex_coords: t,
                                rgba: color,
                                mode: MODE_TEXT,
                            };
                            let mut push_v = |p, t| vertices.push(v(p, t));
                            push_v(
                                [vk_rect.min.x, vk_rect.max.y],
                                [uv_rect.min.x, uv_rect.max.y],
                            );
                            push_v(
                                [vk_rect.min.x, vk_rect.min.y],
                                [uv_rect.min.x, uv_rect.min.y],
                            );
                            push_v(
                                [vk_rect.max.x, vk_rect.min.y],
                                [uv_rect.max.x, uv_rect.min.y],
                            );
                            push_v(
                                [vk_rect.max.x, vk_rect.min.y],
                                [uv_rect.max.x, uv_rect.min.y],
                            );
                            push_v(
                                [vk_rect.max.x, vk_rect.max.y],
                                [uv_rect.max.x, uv_rect.max.y],
                            );
                            push_v(
                                [vk_rect.min.x, vk_rect.max.y],
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
                    let image_ref = match image_map.get(&image_id) {
                        None => continue,
                        Some(img) => img,
                    };

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
                    let [image_w, image_h] = image_ref.dimensions();
                    let (image_w, image_h) = (image_w as Scalar, image_h as Scalar);

                    // Get the sides of the source rectangle as uv coordinates.
                    //
                    // Texture coordinates range:
                    // - left to right: 0.0 to 1.0
                    // - bottom to top: 1.0 to 0.0
                    let (uv_l, uv_r, uv_b, uv_t) = match source_rect {
                        Some(src_rect) => {
                            let (l, r, b, t) = src_rect.l_r_b_t();
                            (
                                (l / image_w) as f32,
                                (r / image_w) as f32,
                                1.0 - (b / image_h) as f32,
                                1.0 - (t / image_h) as f32,
                            )
                        }
                        None => (0.0, 1.0, 1.0, 0.0),
                    };

                    let v = |x, y, t| {
                        // Convert from conrod Scalar range to normalised range -1.0 to 1.0.
                        let x = (x * dpi_factor / half_viewport_w) as f32;
                        let y = -((y * dpi_factor / half_viewport_h) as f32);
                        Vertex {
                            position: [x, y],
                            tex_coords: t,
                            rgba: color,
                            mode: MODE_IMAGE,
                        }
                    };

                    let mut push_v = |x, y, t| vertices.push(v(x, y, t));

                    // Swap bottom and top to suit reversed vulkan coords.
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
            }
        }

        // Enter the final command.
        match current_state {
            State::Plain { start } => commands.push(PreparedCommand::Plain(start..vertices.len())),
            State::Image { image_id, start } => {
                commands.push(PreparedCommand::Image(image_id, start..vertices.len()))
            }
        }

        let fill = Fill {
            glyph_cache_requires_upload,
        };

        Ok(fill)
    }

    /// The rusttype glyph cache used for managing caching of glyphs into the pixel buffer.
    pub fn glyph_cache(&self) -> &text::GlyphCache {
        &self.glyph_cache.0
    }

    /// The CPU-side of the glyph cache, storing all necessary pixel data in a single slice.
    pub fn glyph_cache_pixel_buffer(&self) -> &[u8] {
        &self.glyph_cache_pixel_buffer
    }

    /// Produce an `Iterator` yielding `Command`s.
    ///
    /// These commands describe the order in which unique draw commands and scizzor updates should
    /// occur.
    pub fn commands(&self) -> Commands {
        let Mesh { ref commands, .. } = *self;
        Commands {
            commands: commands.iter(),
        }
    }

    /// The slice containing all `vertices` produced by the `fill` function.
    ///
    /// Note that these vertices may be represent geometry across multiple `Command`s.
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
}

impl<'a> Iterator for Commands<'a> {
    type Item = Command;
    fn next(&mut self) -> Option<Self::Item> {
        let Commands { ref mut commands } = *self;
        commands.next().map(|command| match *command {
            PreparedCommand::Scizzor(scizzor) => Command::Scizzor(scizzor),
            PreparedCommand::Plain(ref range) => Command::Draw(Draw::Plain(range.clone())),
            PreparedCommand::Image(id, ref range) => Command::Draw(Draw::Image(id, range.clone())),
        })
    }
}

impl ops::Deref for GlyphCache {
    type Target = text::GlyphCache<'static>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for GlyphCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Debug for GlyphCache {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GlyphCache")
    }
}

impl From<text::GlyphCache<'static>> for GlyphCache {
    fn from(gc: text::GlyphCache<'static>) -> Self {
        GlyphCache(gc)
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
