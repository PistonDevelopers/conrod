extern crate conrod_core;
#[macro_use]
extern crate vulkano;
extern crate vulkano_shaders;

use std::error::Error as StdError;
use std::fmt;
use std::sync::Arc;

use conrod_core::text::{rt, GlyphCache};
use conrod_core::{color, image, render, Rect, Scalar};

use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::ImmutableBuffer;
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::descriptor_set::{DescriptorSet, FixedSizeDescriptorSetsPool,
                                          PersistentDescriptorSetError,
                                          PersistentDescriptorSetBuildError};
use vulkano::device::*;
use vulkano::format::*;
use vulkano::framebuffer::*;
use vulkano::image::*;
use vulkano::instance::QueueFamily;
use vulkano::memory::DeviceMemoryAllocError;
use vulkano::pipeline::viewport::Scissor;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::*;
use vulkano::sampler::*;

/// A `Command` describing a step in the drawing process.
#[derive(Clone, Debug)]
pub enum Command<'a> {
    /// Draw to the target.
    Draw(Draw<'a>),
    /// Update the scizzor within the pipeline.
    Scizzor(Scissor),
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
    Scizzor(Scissor),
}

/// A loaded vulkan texture and it's width/height
pub struct Image {
    /// The actual image.
    pub image_access: Arc<ImmutableImage<R8G8B8A8Unorm>>,
    /// The width of the image.
    pub width: u32,
    /// The height of the image.
    pub height: u32,
}

/// Draw text from the text cache texture `tex` in the fragment shader.
pub const MODE_TEXT: u32 = 0;
/// Draw an image from the texture at `tex` in the fragment shader.
pub const MODE_IMAGE: u32 = 1;
/// Ignore `tex` and draw simple, colored 2D geometry.
pub const MODE_GEOMETRY: u32 = 2;

mod vs {
    vulkano_shaders::shader! {
    ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 color;
layout(location = 3) in uint mode;

layout(location = 0) out vec2 v_Uv;
layout(location = 1) out vec4 v_Color;
layout(location = 2) flat out uint v_Mode;

void main() {
    v_Uv = uv;
    v_Color = color;
    gl_Position = vec4(pos, 0.0, 1.0);
    v_Mode = mode;
}
"
    }
}

mod fs {
    vulkano_shaders::shader! {
    ty: "fragment",
        src: "
#version 450

layout(set = 0, binding = 0) uniform sampler2D t_Color;

layout(location = 0) in vec2 v_Uv;
layout(location = 1) in vec4 v_Color;
layout(location = 2) flat in uint v_Mode;

layout(location = 0) out vec4 Target0;

void main() {
    // Text
    if (v_Mode == uint(0)) {
        Target0 = v_Color * vec4(1.0, 1.0, 1.0, texture(t_Color, v_Uv).r);

    // Image
    } else if (v_Mode == uint(1)) {
        Target0 = texture(t_Color, v_Uv);

    // 2D Geometry
    } else if (v_Mode == uint(2)) {
        Target0 = v_Color;
    }
}
"
    }
}

/// The `Vertex` type passed to the vertex shader.
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    /// The position of the vertex within vector space.
    ///
    /// [-1.0, 1.0] is the leftmost, bottom position of the display.
    /// [1.0, -1.0] is the rightmost, top position of the display.
    pub pos: [f32; 2],
    /// The coordinates of the texture used by this `Vertex`.
    ///
    /// [0.0, 0.0] is the leftmost, top position of the texture.
    /// [1.0, 1.0] is the rightmost, bottom position of the texture.
    pub uv: [f32; 2],
    /// A color associated with the `Vertex`.
    ///
    /// The way that the color is used depends on the `mode`.
    pub color: [f32; 4],
    /// The mode with which the `Vertex` will be drawn within the fragment shader.
    ///
    /// `0` for rendering text.
    /// `1` for rendering an image.
    /// `2` for rendering non-textured 2D geometry.
    ///
    /// If any other value is given, the fragment shader will not output any color.
    pub mode: u32,
}

impl_vertex!(Vertex, pos, uv, color, mode);

/// A type used for translating `render::Primitives` into `Command`s that indicate how to draw the
/// conrod GUI using `vulkano`.
pub struct Renderer {
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    glyph_cache: GlyphCache<'static>,
    glyph_uploads: Arc<CpuBufferPool<u8>>,
    glyph_cache_tex: Arc<StorageImage<R8Unorm>>,
    glyph_cache_pixel_buffer: Vec<u8>,
    sampler: Arc<Sampler>,
    commands: Vec<PreparedCommand>,
    vertices: Vec<Vertex>,
    tex_descs: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>,
}

/// An command for uploading an individual glyph.
pub struct GlyphCacheCommand<'a> {
    /// The CPU buffer containing the pixel data.
    pub glyph_cache_pixel_buffer: &'a [u8],
    /// The cpu buffer pool used to upload glyph pixels.
    pub glyph_cpu_buffer_pool: Arc<CpuBufferPool<u8>>,
    /// The GPU image to which the glyphs are cached.
    pub glyph_cache_texture: Arc<StorageImage<R8Unorm>>,
}

/// A draw command that maps directly to the `AutoCommandBufferBuilder::draw` method. By returning
/// `DrawCommand`s, we can avoid consuming the entire `AutoCommandBufferBuilder` itself which might
/// not always be available from APIs that wrap Vulkan.
pub struct DrawCommand {
    pub graphics_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    pub dynamic_state: DynamicState,
    pub descriptor_set: Arc<DescriptorSet + Send + Sync>,
    pub vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,
}

/// Errors that might occur during creation of the renderer.
#[derive(Debug)]
pub enum RendererCreationError {
    SamplerCreation(SamplerCreationError),
    ShaderLoad(vulkano::OomError),
    GraphicsPipelineCreation(GraphicsPipelineCreationError),
    ImageCreation(ImageCreationError),
}

/// Errors that might occur during draw calls.
#[derive(Debug)]
pub enum DrawError {
    PersistentDescriptorSet(PersistentDescriptorSetError),
    PersistentDescriptorSetBuild(PersistentDescriptorSetBuildError),
    VertexBufferAlloc(DeviceMemoryAllocError),
}

impl Renderer {
    /// Construct a new empty `Renderer`.
    ///
    /// The dimensions of the glyph cache will be the dimensions of the window multiplied by the
    /// DPI factor.
    pub fn new<'a, L>(
        device: Arc<Device>,
        subpass: Subpass<L>,
        graphics_queue_family: QueueFamily<'a>,
        window_dims: [u32; 2],
        dpi_factor: f64,
    ) -> Result<Self, RendererCreationError>
    where
        L: RenderPassDesc + RenderPassAbstract + Send + Sync + 'static,
    {
        // TODO: Check that necessary subpass properties exist?
        let [w, h] = window_dims;
        let glyph_cache_dims = [(w as f64 * dpi_factor) as u32, (h as f64 * dpi_factor) as u32];
        Self::with_glyph_cache_dimensions(
            device,
            subpass,
            graphics_queue_family,
            glyph_cache_dims,
        )
    }

    /// Construct a new empty `Renderer`.
    pub fn with_glyph_cache_dimensions<'a, L>(
        device: Arc<Device>,
        subpass: Subpass<L>,
        graphics_queue_family: QueueFamily<'a>,
        glyph_cache_dims: [u32; 2],
    ) -> Result<Self, RendererCreationError>
    where
        L: RenderPassDesc + RenderPassAbstract + Send + Sync + 'static,
    {
        let sampler = Sampler::new(
            device.clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            0.0,
            1.0,
            0.0,
            0.0,
        )?;

        let vertex_shader = vs::Shader::load(device.clone())
            .map_err(|err| RendererCreationError::ShaderLoad(err))?;
        let fragment_shader = fs::Shader::load(device.clone())
            .map_err(|err| RendererCreationError::ShaderLoad(err))?;

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vertex_shader.main_entry_point(), ())
                .depth_stencil_disabled()
                .triangle_list()
                .front_face_clockwise()
                .viewports_scissors_dynamic(1)
                .fragment_shader(fragment_shader.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(subpass)
                .build(device.clone())?
        );

        let (glyph_cache, glyph_cache_tex, glyph_cache_pixel_buffer) = {
            let [width, height] = glyph_cache_dims;

            const SCALE_TOLERANCE: f32 = 0.1;
            const POSITION_TOLERANCE: f32 = 0.1;

            let glyph_cache = GlyphCache::builder()
                .dimensions(width, height)
                .scale_tolerance(SCALE_TOLERANCE)
                .position_tolerance(POSITION_TOLERANCE)
                .build();

            let glyph_cache_tex = StorageImage::with_usage(
                device.clone(),
                Dimensions::Dim2d { width, height },
                R8Unorm,
                ImageUsage {
                    transfer_destination: true,
                    sampled: true,
                    ..ImageUsage::none()
                },
                vec![graphics_queue_family],
            )?;

            let glyph_cache_pixel_buffer = vec![0u8; width as usize * height as usize];

            (glyph_cache, glyph_cache_tex, glyph_cache_pixel_buffer)
        };

        let tex_descs = FixedSizeDescriptorSetsPool::new(pipeline.clone() as Arc<_>, 0);
        let glyph_uploads = Arc::new(CpuBufferPool::upload(device.clone()));

        Ok(Renderer {
            pipeline: pipeline,
            glyph_cache,
            glyph_uploads,
            glyph_cache_tex,
            glyph_cache_pixel_buffer,
            sampler,
            commands: Vec::new(),
            vertices: Vec::new(),
            tex_descs,
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
    ///
    /// This method may return an `Option<GlyphCacheCommand>`, in which case the user should use
    /// the contained `glyph_cpu_buffer_pool` to write the pixel data to the GPU, and then use a
    /// `copy_buffer_to_image` command to write the data to the given `glyph_cache_texture` image.
    pub fn fill<'a, P: render::PrimitiveWalker>(
        &'a mut self,
        image_map: &image::Map<Image>,
        viewport: [f32; 4],
        dpi_factor: f64,
        mut primitives: P,
    ) -> Result<Option<GlyphCacheCommand<'a>>, rt::gpu_cache::CacheWriteErr> {
        let Renderer {
            ref mut commands,
            ref mut vertices,
            ref mut glyph_cache,
            ref glyph_uploads,
            ref glyph_cache_tex,
            ref mut glyph_cache_pixel_buffer,
            ..
        } = *self;

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
                        current_state = State::Plain {
                            start: vertices.len(),
                        };
                    }
                }
            };
        }

        // Viewport dimensions and the "dots per inch" factor.
        let (viewport_w, viewport_h) = (viewport[2] - viewport[0], viewport[3] - viewport[1]);
        let (win_w, win_h) = (viewport_w as Scalar, viewport_h as Scalar);
        let half_win_w = win_w / 2.0;
        let half_win_h = win_h / 2.0;

        // Functions for converting for conrod scalar coords to GL vertex coords (-1.0 to 1.0).
        let vx = |x: Scalar| (x * dpi_factor / half_win_w) as f32;
        let vy = |y: Scalar| -1.0 * (y * dpi_factor / half_win_h) as f32;

        // The width of the glyph cache, useful for copying pixel data.
        let glyph_cache_w = StorageImage::dimensions(glyph_cache_tex).width() as usize;

        // Keep track of whether or not the glyph cache texture needs to be updated.
        let mut update_glyph_cache_tex = false;

        let mut current_scizzor = Scissor {
            origin: [0, 0],
            dimensions: [viewport_w as u32, viewport_h as u32],
        };

        let rect_to_scissor = |rect: Rect| {
            let (w, h) = rect.w_h();
            let left = (rect.left() * dpi_factor + half_win_w) as i32;
            let top = (rect.top() * dpi_factor - half_win_h).abs() as i32;
            let width = (w * dpi_factor) as u32;
            let height = (h * dpi_factor) as u32;
            Scissor {
                origin: [left.max(0), top.max(0)],
                dimensions: [width.min(viewport_w as u32), height.min(viewport_h as u32)],
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
            let new_scizzor = rect_to_scissor(scizzor);
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
                }

                render::PrimitiveKind::TrianglesSingleColor { color, triangles } => {
                    if triangles.is_empty() {
                        continue;
                    }

                    switch_to_plain_state!();

                    let color = gamma_srgb_to_linear(color.into());

                    let v = |p: [Scalar; 2]| Vertex {
                        pos: [vx(p[0]), vy(p[1])],
                        uv: [0.0, 0.0],
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
                        pos: [vx(p[0]), vy(p[1])],
                        uv: [0.0, 0.0],
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
                        update_glyph_cache_tex = true;
                    })?;

                    let color = gamma_srgb_to_linear(color.to_fsa());
                    let cache_id = font_id.index();
                    let origin = rt::point(0.0, 0.0);

                    // A closure to convert RustType rects to GL rects
                    let to_vk_rect = |screen_rect: rt::Rect<i32>| rt::Rect {
                        min: origin
                            + (rt::vector(
                                screen_rect.min.x as f32 / viewport_w - 0.5,
                                screen_rect.min.y as f32 / viewport_h - 0.5,
                            )) * 2.0,
                        max: origin
                            + (rt::vector(
                                screen_rect.max.x as f32 / viewport_w - 0.5,
                                screen_rect.max.y as f32 / viewport_h - 0.5,
                            )) * 2.0,
                    };

                    for g in positioned_glyphs {
                        if let Ok(Some((uv_rect, screen_rect))) = glyph_cache.rect_for(cache_id, g)
                        {
                            let vk_rect = to_vk_rect(screen_rect);
                            let v = |p, t| Vertex {
                                pos: p,
                                uv: t,
                                color: color,
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
                    let (image_w, image_h) = (image_ref.width, image_ref.height);
                    let (image_w, image_h) = (image_w as Scalar, image_h as Scalar);

                    // Get the sides of the source rectangle as uv coordinates.
                    //
                    // Texture coordinates range:
                    // - left to right: 0.0 to 1.0
                    // - bottom to top: 0.0 to 0.1
                    // Note bottom and top are flipped in comparison to glium so that we don't need
                    //  to flip images when loading
                    let (uv_l, uv_r, uv_t, uv_b) = match source_rect {
                        Some(src_rect) => {
                            let (l, r, b, t) = src_rect.l_r_b_t();
                            (
                                (l / image_w) as f32,
                                (r / image_w) as f32,
                                (t / image_h) as f32,
                                (b / image_h) as f32,
                            )
                        }
                        None => (0.0, 1.0, 0.0, 1.0),
                    };

                    let v = |x, y, t| {
                        // Convert from conrod Scalar range to GL range -1.0 to 1.0.
                        let x = (x * dpi_factor / half_win_w) as f32;
                        let y = -((y * dpi_factor / half_win_h) as f32);
                        Vertex {
                            pos: [x, y],
                            uv: t,
                            color: color,
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

        let glyph_cache_cmd = match update_glyph_cache_tex {
            false => None,
            true => Some(GlyphCacheCommand {
                glyph_cache_pixel_buffer: glyph_cache_pixel_buffer,
                glyph_cpu_buffer_pool: glyph_uploads.clone(),
                glyph_cache_texture: glyph_cache_tex.clone(),
            }),
        };

        Ok(glyph_cache_cmd)
    }

    /// Draws using the inner list of `Command`s to a list of `DrawCommand`s compatible with the
    /// vulkano command buffer builders.
    ///
    /// Uses the given `queue` for submitting vertex buffers.
    ///
    /// Note: If you require more granular control over rendering, you may want to use the `fill`
    /// and `commands` methods separately. This method is simply a convenience wrapper around those
    /// methods for the case that the user does not require accessing or modifying conrod's draw
    /// parameters, uniforms or generated draw commands.
    pub fn draw(
        &mut self,
        queue: Arc<Queue>,
        image_map: &image::Map<Image>,
        viewport: [f32; 4],
    ) -> Result<Vec<DrawCommand>, DrawError> {
        let current_viewport = Viewport {
            origin: [viewport[0], viewport[1]],
            dimensions: [viewport[2] - viewport[0], viewport[3] - viewport[1]],
            depth_range: 0.0..1.0,
        };

        let mut current_scissor = Scissor {
            origin: [viewport[0] as i32, viewport[1] as i32],
            dimensions: [
                (viewport[2] - viewport[0]) as u32,
                (viewport[3] - viewport[1]) as u32,
            ],
        };

        let desc_cache = Arc::new(
            self.tex_descs
                .next()
                .add_sampled_image(self.glyph_cache_tex.clone(), self.sampler.clone())?
                .build()?
        );

        let tex_descs = &mut self.tex_descs;

        let commands = Commands {
            commands: self.commands.iter(),
            vertices: &self.vertices,
        };

        let dynamic_state = |scissor| DynamicState {
            line_width: None,
            viewports: Some(vec![current_viewport.clone()]),
            scissors: Some(vec![scissor]),
        };

        let mut draw_commands = vec![];

        for command in commands {
            match command {
                // Update the `scizzor` before continuing to draw.
                Command::Scizzor(scizzor) => current_scissor = scizzor,

                // Draw to the target with the given `draw` command.
                Command::Draw(draw) => match draw {
                    // Draw text and plain 2D geometry.
                    Draw::Plain(verts) => {
                        if verts.len() > 0 {
                            let (vbuf, _vbuf_fut) = ImmutableBuffer::<[Vertex]>::from_iter(
                                verts.iter().cloned(),
                                BufferUsage::vertex_buffer(),
                                queue.clone(),
                            )?;
                            draw_commands.push(DrawCommand {
                                graphics_pipeline: self.pipeline.clone(),
                                dynamic_state: dynamic_state(current_scissor.clone()),
                                vertex_buffer: vbuf,
                                descriptor_set: desc_cache.clone(),
                            });
                        }
                    }

                    // Draw an image whose texture data lies within the `image_map` at the
                    // given `id`.
                    Draw::Image(image_id, verts) => {
                        if verts.len() == 0 {
                            continue;
                        }
                        if let Some(image) = image_map.get(&image_id) {
                            let desc_image = Arc::new(
                                tex_descs
                                    .next()
                                    .add_sampled_image(
                                        image.image_access.clone(),
                                        self.sampler.clone(),
                                    )?
                                    .build()?
                            );
                            let (vbuf, _vbuf_fut) = ImmutableBuffer::from_iter(
                                verts.iter().cloned(),
                                BufferUsage::vertex_buffer(),
                                queue.clone(),
                            )?;
                            draw_commands.push(DrawCommand {
                                graphics_pipeline: self.pipeline.clone(),
                                dynamic_state: dynamic_state(current_scissor.clone()),
                                vertex_buffer: vbuf,
                                descriptor_set: desc_image,
                            });
                        }
                    }
                },
            }
        }

        Ok(draw_commands)
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

impl From<SamplerCreationError> for RendererCreationError {
    fn from(err: SamplerCreationError) -> Self {
        RendererCreationError::SamplerCreation(err)
    }
}

impl From<GraphicsPipelineCreationError> for RendererCreationError {
    fn from(err: GraphicsPipelineCreationError) -> Self {
        RendererCreationError::GraphicsPipelineCreation(err)
    }
}

impl From<ImageCreationError> for RendererCreationError {
    fn from(err: ImageCreationError) -> Self {
        RendererCreationError::ImageCreation(err)
    }
}

impl From<PersistentDescriptorSetError> for DrawError {
    fn from(err: PersistentDescriptorSetError) -> Self {
        DrawError::PersistentDescriptorSet(err)
    }
}

impl From<PersistentDescriptorSetBuildError> for DrawError {
    fn from(err: PersistentDescriptorSetBuildError) -> Self {
        DrawError::PersistentDescriptorSetBuild(err)
    }
}

impl From<DeviceMemoryAllocError> for DrawError {
    fn from(err: DeviceMemoryAllocError) -> Self {
        DrawError::VertexBufferAlloc(err)
    }
}

impl StdError for RendererCreationError {
    fn description(&self) -> &str {
        match *self {
            RendererCreationError::SamplerCreation(ref err) => err.description(),
            RendererCreationError::ShaderLoad(ref err) => err.description(),
            RendererCreationError::GraphicsPipelineCreation(ref err) => err.description(),
            RendererCreationError::ImageCreation(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            RendererCreationError::SamplerCreation(ref err) => Some(err),
            RendererCreationError::ShaderLoad(ref err) => Some(err),
            RendererCreationError::GraphicsPipelineCreation(ref err) => Some(err),
            RendererCreationError::ImageCreation(ref err) => Some(err),
        }
    }
}

impl StdError for DrawError {
    fn description(&self) -> &str {
        match *self {
            DrawError::PersistentDescriptorSet(ref err) => err.description(),
            DrawError::PersistentDescriptorSetBuild(ref err) => err.description(),
            DrawError::VertexBufferAlloc(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            DrawError::PersistentDescriptorSet(ref err) => Some(err),
            DrawError::PersistentDescriptorSetBuild(ref err) => Some(err),
            DrawError::VertexBufferAlloc(ref err) => Some(err),
        }
    }
}

impl fmt::Display for RendererCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl fmt::Display for DrawError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}
