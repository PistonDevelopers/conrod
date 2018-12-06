extern crate conrod_core;
#[macro_use]
extern crate vulkano;
extern crate vulkano_shaders;

use std::sync::Arc;

use conrod_core::{render, color, image, Rect, Scalar};
use conrod_core::text::{rt,GlyphCache};

use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::pipeline::viewport::Scissor;
use vulkano::instance::QueueFamily;
use vulkano::device::*;
use vulkano::pipeline::*;
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::framebuffer::*;
use vulkano::image::*;
use vulkano::format::*;
use vulkano::sampler::*;
use vulkano::pipeline::viewport::Viewport;
use vulkano::command_buffer::DynamicState;

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
        Target0 = v_Color * vec4(1.0, 1.0, 1.0, texture(t_Color, v_Uv).a);

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
pub  struct Vertex { 
    /// The position of the vertex within vector space.
    ///
    /// [-1.0, -1.0] is the leftmost, bottom position of the display.
    /// [1.0, 1.0] is the rightmost, top position of the display.
    pub pos:   [f32; 2],
    /// The coordinates of the texture used by this `Vertex`.
    ///
    /// [0.0, 0.0] is the leftmost, top position of the texture.
    /// [1.0, 1.0] is the rightmost, bottom position of the texture.
    pub uv:    [f32; 2],
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
    pub mode:  u32, 
}

impl_vertex!(Vertex, pos, uv, color, mode);

/// A type used for translating `render::Primitives` into `Command`s that indicate how to draw the
/// conrod GUI using `vulkano`.
pub struct Renderer {
    pipeline: Box<Arc<GraphicsPipelineAbstract+Send+Sync>>,
    glyph_cache: GlyphCache<'static>,
    glyph_uploads: CpuBufferPool<[u8; 4]>,
    glyph_cache_tex: Arc<StorageImage<R8G8B8A8Unorm>>,
    sampler: Arc<Sampler>,
    dpi_factor: f64,
    commands: Vec<PreparedCommand>,
    vertices: Vec<Vertex>,
    tex_descs: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract+Send+Sync>>,
}

impl Renderer {
    /// Construct a new empty `Renderer`.
    pub fn new<
        'a,
        L: RenderPassDesc + RenderPassAbstract + Send + Sync + 'static
    > (
        device: Arc<Device>,
        subpass: Subpass<L>,
        graphics_queue_family: QueueFamily<'a>,
        width: u32,
        height: u32,
        dpi_factor: f64
    ) -> Self {
        let sampler = Sampler::new(
            device.clone(), 
            Filter::Linear,
            Filter::Linear, 
            MipmapMode::Nearest,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            0.0, 1.0, 0.0, 0.0
        ).unwrap();

        let vertex_shader = vs::Shader::load(device.clone())
            .expect("failed to create shader module");

        let fragment_shader = fs::Shader::load(device.clone())
            .expect("failed to create shader module");

        let pipeline = Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vertex_shader.main_entry_point(), ())
            .depth_stencil_disabled()
            .triangle_list()
            .front_face_clockwise()
            //.cull_mode_back()
            .viewports_scissors_dynamic(1)
            .fragment_shader(fragment_shader.main_entry_point(), ())
            .blend_alpha_blending()
            .render_pass(subpass)
            .build(device.clone())
            .unwrap());

        let (glyph_cache, glyph_cache_tex) = {

            let width = (width as f64 * dpi_factor) as u32;
            let height = (height as f64 * dpi_factor) as u32;

            const SCALE_TOLERANCE: f32 = 0.1;
            const POSITION_TOLERANCE: f32 = 0.1;

            let glyph_cache = GlyphCache::builder()
                .dimensions(width, height)
                .scale_tolerance(SCALE_TOLERANCE)
                .position_tolerance(POSITION_TOLERANCE)
                .build();

            let glyph_cache_tex = StorageImage::with_usage(
                device.clone(), 
                Dimensions::Dim2d{ width, height }, 
                R8G8B8A8Unorm, 
                ImageUsage {
                    transfer_destination: true,
                    sampled: true,
                    .. ImageUsage::none()
                }, 
                vec![graphics_queue_family]
            ).unwrap();

            (glyph_cache, glyph_cache_tex)
        };

        let tex_descs = FixedSizeDescriptorSetsPool::new(pipeline.clone() as Arc<_>, 0);

        Renderer {
            pipeline: Box::new(pipeline),
            glyph_cache,
            glyph_uploads: CpuBufferPool::upload(device.clone()),
            glyph_cache_tex,
            sampler,
            dpi_factor,
            commands: Vec::new(),
            vertices: Vec::new(),
            tex_descs,            
        }
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
    pub fn fill<
        P: render::PrimitiveWalker
    > (
        &mut self,
        mut cmd: AutoCommandBufferBuilder,
        image_map: &image::Map<Image>,
        viewport: [f32; 4],
        mut primitives: P   
    ) -> AutoCommandBufferBuilder {
        let Renderer { 
            ref mut commands, 
            ref mut vertices, 
            ref mut glyph_cache, 
            ref     glyph_uploads,
            ref mut glyph_cache_tex, 
            dpi_factor, 
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
                        current_state = State::Plain { start: vertices.len() };
                    },
                }
            };
        }

        // Framebuffer dimensions and the "dots per inch" factor.
        let (screen_w, screen_h) = (viewport[2]-viewport[0], viewport[3]-viewport[1]);
        let (win_w, win_h) = (screen_w as Scalar, screen_h as Scalar);
        let half_win_w = win_w / 2.0;
        let half_win_h = win_h / 2.0;

        // Functions for converting for conrod scalar coords to GL vertex coords (-1.0 to 1.0).
        let vx = |x: Scalar| (x * dpi_factor / half_win_w) as f32;
        let vy = |y: Scalar| -1.0 * (y * dpi_factor / half_win_h) as f32;

        let mut current_scizzor = Scissor {
            origin: [0, 0],
            dimensions: [screen_w as u32, screen_h as u32],
        };

        let rect_to_scissor = |rect: Rect| {
            let (w, h) = rect.w_h();
            let left = (rect.left() * dpi_factor + half_win_w) as i32;
            let bottom = (rect.bottom() * dpi_factor + half_win_h) as i32;
            let width = (w * dpi_factor) as u32;
            let height = (h * dpi_factor) as u32;
            Scissor {
                origin: [left.max(0), bottom.max(0)],
                dimensions: [width.min(screen_w as u32), height.min(screen_h as u32)],
            }
        };

        // Draw each primitive in order of depth.
        while let Some(primitive) = primitives.next_primitive() {
            let render::Primitive { kind, scizzor, rect, .. } = primitive;

            // Check for a `Scizzor` command.
            let new_scizzor = rect_to_scissor(scizzor);
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
                },

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
                },

                render::PrimitiveKind::Text { color, text, font_id } => {
                    switch_to_plain_state!();

                    let positioned_glyphs = text.positioned_glyphs(dpi_factor as f32);

                    // Queue the glyphs to be cached
                    for glyph in positioned_glyphs {
                        glyph_cache.queue_glyph(font_id.index(), glyph.clone());
                    }

                    let mut capture_cmd = Some(cmd);

                    glyph_cache.cache_queued(|rect, data| {
                        let offset = [rect.min.x as u32, rect.min.y as u32];
                        let size = [rect.width() as u32, rect.height() as u32];

                        let new_data = data.iter()
                            .map(|x| [255, 255, 255, *x])
                            .collect::<Vec<[u8; 4]>>();

                        capture_cmd = Some(update_texture(
                            capture_cmd.take().unwrap(), 
                            glyph_uploads, 
                            glyph_cache_tex.clone(), 
                            offset, 
                            size, 
                            new_data
                        ));
                    }).unwrap();

                    cmd = capture_cmd.take().unwrap();

                    let color = gamma_srgb_to_linear(color.to_fsa());
                    let cache_id = font_id.index();
                    let origin = rt::point(0.0, 0.0);

                    // A closure to convert RustType rects to GL rects
                    let to_vk_rect = |screen_rect: rt::Rect<i32>| rt::Rect {
                        min: origin
                            + (rt::vector(screen_rect.min.x as f32 / screen_w - 0.5,
                                          screen_rect.min.y as f32 / screen_h - 0.5)) * 2.0,
                        max: origin
                            + (rt::vector(screen_rect.max.x as f32 / screen_w - 0.5,
                                          screen_rect.max.y as f32 / screen_h - 0.5)) * 2.0,
                    };

                    for g in positioned_glyphs {
                        if let Ok(Some((uv_rect, screen_rect))) = 
                            glyph_cache.rect_for(cache_id, g) {

                            let vk_rect = to_vk_rect(screen_rect);
                            let v = |p, t| Vertex {
                                pos: p,
                                uv: t,
                                color: color,
                                mode: MODE_TEXT,
                            };
                            let mut push_v = |p, t| vertices.push(v(p, t));
                            push_v([vk_rect.min.x, vk_rect.max.y], [uv_rect.min.x, uv_rect.max.y]);
                            push_v([vk_rect.min.x, vk_rect.min.y], [uv_rect.min.x, uv_rect.min.y]);
                            push_v([vk_rect.max.x, vk_rect.min.y], [uv_rect.max.x, uv_rect.min.y]);
                            push_v([vk_rect.max.x, vk_rect.min.y], [uv_rect.max.x, uv_rect.min.y]);
                            push_v([vk_rect.max.x, vk_rect.max.y], [uv_rect.max.x, uv_rect.max.y]);
                            push_v([vk_rect.min.x, vk_rect.max.y], [uv_rect.min.x, uv_rect.max.y]);
                        }
                    }
                },

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
                        },

                        // If we were drawing a different image, switch state to draw *this* image.
                        State::Image { image_id, start } => {
                            commands.push(PreparedCommand::Image(image_id, start..vertices.len()));
                            current_state = State::Image {
                                image_id: new_image_id,
                                start: vertices.len(),
                            };
                        },
                    }

                    let color = color.unwrap_or(color::WHITE).to_fsa();

                    let image_ref = image_map.get(&image_id).unwrap();
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
                            ((l / image_w) as f32,
                             (r / image_w) as f32,
                             (t / image_h) as f32,
                             (b / image_h) as f32)
                        },
                        None => (0.0, 1.0, 0.0, 1.0),
                    };

                    let v = |x, y, t| {
                        // Convert from conrod Scalar range to GL range -1.0 to 1.0.
                        let x = (x * dpi_factor / half_win_w) as f32;
                        let y = (y * dpi_factor / half_win_h) as f32;
                        Vertex {
                            pos: [x, y],
                            uv: t,
                            color: color,
                            mode: MODE_IMAGE,
                        }
                    };

                    let mut push_v = |x, y, t| vertices.push(v(x, y, t));

                    let (l, r, t, b) = rect.l_r_b_t();

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
            State::Image { image_id, start } =>
                commands.push(PreparedCommand::Image(image_id, start..vertices.len())),
        }

        cmd
    }

    /// Draws using the inner list of `Command`s to the given `display`.
    ///
    /// Note: If you require more granular control over rendering, you may want to use the `fill`
    /// and `commands` methods separately. This method is simply a convenience wrapper around those
    /// methods for the case that the user does not require accessing or modifying conrod's draw
    /// parameters, uniforms or generated draw commands.
    pub fn draw(
        &mut self, 
        mut cmd: AutoCommandBufferBuilder,
        device: Arc<Device>,
        image_map: &image::Map<Image>,
        viewport: [f32; 4]        
    ) -> AutoCommandBufferBuilder {
        let current_viewport = Viewport {
            origin: [viewport[0], viewport[1]],
            dimensions: [viewport[2]-viewport[0], viewport[3]-viewport[1]],
            depth_range: 0.0 .. 1.0,
        };

        let mut current_scissor = Scissor {
            origin: [viewport[0] as i32, viewport[1] as i32],
            dimensions: [(viewport[2]-viewport[0]) as u32, (viewport[3]-viewport[1]) as u32],
        };

        let desc_cache = Arc::new(self.tex_descs.next()
            .add_sampled_image(self.glyph_cache_tex.clone(), self.sampler.clone()).unwrap()
            .build().unwrap());

        let tex_descs = &mut self.tex_descs;

        let commands = Commands {
            commands: self.commands.iter(),
            vertices: &self.vertices,
        };

        let dynamic_state = |scissor| {
            DynamicState {
                line_width: None,
                viewports: Some(vec![current_viewport.clone()]),
                scissors: Some(vec![scissor]),
            }
        };

        for command in commands {
            match command {

                // Update the `scizzor` before continuing to draw.
                Command::Scizzor(scizzor) => current_scissor = scizzor,

                // Draw to the target with the given `draw` command.
                Command::Draw(draw) => match draw {

                    // Draw text and plain 2D geometry.
                    Draw::Plain(verts) => {
                        if verts.len() > 0 {
                            let vbuf = CpuAccessibleBuffer::<[Vertex]>::from_iter(
                                device.clone(), 
                                BufferUsage::vertex_buffer(), 
                                verts.iter().cloned()
                            ).unwrap();

                            cmd = cmd
                                .draw(self.pipeline.clone(),
                                    &dynamic_state(current_scissor.clone()),
                                    vec![vbuf],
                                    desc_cache.clone(), 
                                    ())
                                .unwrap();
                        }
                    },

                    // Draw an image whose texture data lies within the `image_map` at the
                    // given `id`.
                    Draw::Image(image_id, verts) => {
                        if verts.len() > 0 {
                            let image = image_map.get(&image_id).unwrap().image_access.clone();

                            let desc_image = Arc::new(tex_descs.next()
                                .add_sampled_image(image, self.sampler.clone()).unwrap()
                                .build().unwrap());

                            let vbuf = CpuAccessibleBuffer::from_iter(
                                device.clone(), 
                                BufferUsage::vertex_buffer(), 
                                verts.iter().cloned()
                            ).unwrap();

                            cmd = cmd
                                .draw(self.pipeline.clone(),
                                    &dynamic_state(current_scissor.clone()),
                                    vec![vbuf],
                                    desc_image, 
                                    ())
                                .unwrap();
                        }
                    },
                }
            }
        }

        cmd
    }
}

fn update_texture(
    cmd: AutoCommandBufferBuilder,
    pool: &CpuBufferPool<[u8;4]>,
    texture: Arc<StorageImage<R8G8B8A8Unorm>>,
    offset: [u32;2],
    size: [u32;2],
    data: Vec<[u8;4]>
) -> AutoCommandBufferBuilder {
    let buffer = pool.chunk(data.iter().cloned()).unwrap();

    cmd.copy_buffer_to_image_dimensions(
        buffer, 
        texture, 
        [offset[0], offset[1], 0], 
        [size[0], size[1], 1], 
        0, 
        1, 
        0
    ).unwrap()
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
