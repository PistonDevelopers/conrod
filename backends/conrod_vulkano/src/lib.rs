extern crate conrod_core;
#[macro_use]
extern crate vulkano;
extern crate vulkano_shaders;

use std::error::Error as StdError;
use std::fmt;
use std::sync::Arc;

use conrod_core::mesh::{self, Mesh};
use conrod_core::text::rt;
use conrod_core::{image, render, Rect, Scalar};

use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::ImmutableBuffer;
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::descriptor_set::{
    DescriptorSet, FixedSizeDescriptorSetsPool, PersistentDescriptorSetBuildError,
    PersistentDescriptorSetError,
};
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

/// A loaded vulkan texture and it's width/height
pub struct Image {
    /// The immutable image type, represents the data loaded onto the GPU.
    ///
    /// Uses a dynamic format for flexibility on the kinds of images that might be loaded.
    pub image_access: Arc<ImmutableImage<Format>>,
    /// The width of the image.
    pub width: u32,
    /// The height of the image.
    pub height: u32,
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

impl_vertex!(Vertex, position, tex_coords, rgba, mode);

mod vs {
    vulkano_shaders::shader! {
    ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 tex_coords;
layout(location = 2) in vec4 rgba;
layout(location = 3) in uint mode;

layout(location = 0) out vec2 v_Uv;
layout(location = 1) out vec4 v_Color;
layout(location = 2) flat out uint v_Mode;

void main() {
    v_Uv = tex_coords;
    v_Color = rgba;
    gl_Position = vec4(position, 0.0, 1.0);
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

/// A type used for translating `render::Primitives` into `Command`s that indicate how to draw the
/// conrod GUI using `vulkano`.
pub struct Renderer {
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    glyph_uploads: Arc<CpuBufferPool<u8>>,
    glyph_cache_tex: Arc<StorageImage<R8Unorm>>,
    sampler: Arc<Sampler>,
    tex_descs: FixedSizeDescriptorSetsPool<Arc<dyn GraphicsPipelineAbstract + Send + Sync>>,
    mesh: Mesh,
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
    pub graphics_pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
    pub dynamic_state: DynamicState,
    pub descriptor_set: Arc<dyn DescriptorSet + Send + Sync>,
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

impl mesh::ImageDimensions for Image {
    fn dimensions(&self) -> [u32; 2] {
        [self.width, self.height]
    }
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
        let glyph_cache_dims = [
            (w as f64 * dpi_factor) as u32,
            (h as f64 * dpi_factor) as u32,
        ];
        Self::with_glyph_cache_dimensions(device, subpass, graphics_queue_family, glyph_cache_dims)
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
                .build(device.clone())?,
        );

        let mesh = Mesh::with_glyph_cache_dimensions(glyph_cache_dims);

        let glyph_cache_tex = {
            let [width, height] = glyph_cache_dims;
            StorageImage::with_usage(
                device.clone(),
                Dimensions::Dim2d { width, height },
                R8Unorm,
                ImageUsage {
                    transfer_destination: true,
                    sampled: true,
                    ..ImageUsage::none()
                },
                vec![graphics_queue_family],
            )?
        };

        let tex_descs = FixedSizeDescriptorSetsPool::new(pipeline.clone() as Arc<_>, 0);
        let glyph_uploads = Arc::new(CpuBufferPool::upload(device.clone()));

        Ok(Renderer {
            pipeline: pipeline,
            glyph_uploads,
            glyph_cache_tex,
            sampler,
            tex_descs,
            mesh,
        })
    }

    /// Produce an `Iterator` yielding `Command`s.
    pub fn commands(&self) -> mesh::Commands {
        self.mesh.commands()
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
        primitives: P,
    ) -> Result<Option<GlyphCacheCommand<'a>>, rt::gpu_cache::CacheWriteErr> {
        let Renderer {
            ref glyph_uploads,
            ref glyph_cache_tex,
            ref mut mesh,
            ..
        } = *self;
        let [vp_l, vp_t, vp_r, vp_b] = viewport;
        let lt = [vp_l as Scalar, vp_t as Scalar];
        let rb = [vp_r as Scalar, vp_b as Scalar];
        let viewport = Rect::from_corners(lt, rb);
        let fill = mesh.fill(viewport, dpi_factor, image_map, primitives)?;
        let glyph_cache_cmd = match fill.glyph_cache_requires_upload {
            false => None,
            true => Some(GlyphCacheCommand {
                glyph_cache_pixel_buffer: mesh.glyph_cache_pixel_buffer(),
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

        let mut current_scizzor = Scissor {
            origin: [viewport[0] as i32, viewport[1] as i32],
            dimensions: [
                (viewport[2] - viewport[0]) as u32,
                (viewport[3] - viewport[1]) as u32,
            ],
        };

        let conv_scizzor = |s: mesh::Scizzor| Scissor {
            origin: s.top_left,
            dimensions: s.dimensions,
        };

        let desc_cache = Arc::new(
            self.tex_descs
                .next()
                .add_sampled_image(self.glyph_cache_tex.clone(), self.sampler.clone())?
                .build()?,
        );

        let tex_descs = &mut self.tex_descs;

        let commands = self.mesh.commands();

        let dynamic_state = |scissor| DynamicState {
            viewports: Some(vec![current_viewport.clone()]),
            scissors: Some(vec![scissor]),
            ..DynamicState::none()
        };

        let mut draw_commands = vec![];

        for command in commands {
            match command {
                // Update the `scizzor` before continuing to draw.
                mesh::Command::Scizzor(scizzor) => current_scizzor = conv_scizzor(scizzor),

                // Draw to the target with the given `draw` command.
                mesh::Command::Draw(draw) => match draw {
                    // Draw text and plain 2D geometry.
                    mesh::Draw::Plain(vert_range) => {
                        if vert_range.len() > 0 {
                            let verts = &self.mesh.vertices()[vert_range];
                            let verts = conv_vertex_buffer(verts);
                            let (vbuf, _vbuf_fut) = ImmutableBuffer::<[Vertex]>::from_iter(
                                verts.iter().cloned(),
                                BufferUsage::vertex_buffer(),
                                queue.clone(),
                            )?;
                            draw_commands.push(DrawCommand {
                                graphics_pipeline: self.pipeline.clone(),
                                dynamic_state: dynamic_state(current_scizzor.clone()),
                                vertex_buffer: vbuf,
                                descriptor_set: desc_cache.clone(),
                            });
                        }
                    }

                    // Draw an image whose texture data lies within the `image_map` at the
                    // given `id`.
                    mesh::Draw::Image(image_id, vert_range) => {
                        if vert_range.len() == 0 {
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
                                    .build()?,
                            );
                            let verts = &self.mesh.vertices()[vert_range];
                            let verts = conv_vertex_buffer(verts);
                            let (vbuf, _vbuf_fut) = ImmutableBuffer::from_iter(
                                verts.iter().cloned(),
                                BufferUsage::vertex_buffer(),
                                queue.clone(),
                            )?;
                            draw_commands.push(DrawCommand {
                                graphics_pipeline: self.pipeline.clone(),
                                dynamic_state: dynamic_state(current_scizzor.clone()),
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

fn conv_vertex_buffer(buffer: &[mesh::Vertex]) -> &[Vertex] {
    unsafe { std::mem::transmute(buffer) }
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
    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            RendererCreationError::SamplerCreation(ref err) => Some(err),
            RendererCreationError::ShaderLoad(ref err) => Some(err),
            RendererCreationError::GraphicsPipelineCreation(ref err) => Some(err),
            RendererCreationError::ImageCreation(ref err) => Some(err),
        }
    }
}

impl StdError for DrawError {
    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            DrawError::PersistentDescriptorSet(ref err) => Some(err),
            DrawError::PersistentDescriptorSetBuild(ref err) => Some(err),
            DrawError::VertexBufferAlloc(ref err) => Some(err),
        }
    }
}

impl fmt::Display for RendererCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RendererCreationError::SamplerCreation(ref err) => err.fmt(f),
            RendererCreationError::ShaderLoad(ref err) => err.fmt(f),
            RendererCreationError::GraphicsPipelineCreation(ref err) => err.fmt(f),
            RendererCreationError::ImageCreation(ref err) => err.fmt(f),
        }
    }
}

impl fmt::Display for DrawError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DrawError::PersistentDescriptorSet(ref err) => err.fmt(f),
            DrawError::PersistentDescriptorSetBuild(ref err) => err.fmt(f),
            DrawError::VertexBufferAlloc(ref err) => err.fmt(f),
        }
    }
}
