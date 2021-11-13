extern crate conrod_core;
#[macro_use]
extern crate vulkano;

use std::error::Error as StdError;
use std::fmt;
use std::sync::Arc;

use conrod_core::mesh::{self, Mesh};
use conrod_core::text::rt;
use conrod_core::{image, render, Rect, Scalar};
use std::ffi::CString;
use vulkano::buffer::{BufferUsage, CpuBufferPool, ImmutableBuffer};
use vulkano::descriptor_set::layout::{
    DescriptorDesc, DescriptorDescImage, DescriptorDescTy, DescriptorSetDesc, DescriptorSetLayout,
};
use vulkano::descriptor_set::{DescriptorSet, DescriptorSetError, SingleLayoutDescSetPool};
use vulkano::device::physical::QueueFamily;
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewType};
use vulkano::image::{
    ImageCreateFlags, ImageCreationError, ImageDimensions, ImageUsage, ImmutableImage, StorageImage,
};
use vulkano::memory::DeviceMemoryAllocError;
use vulkano::pipeline::layout::PipelineLayout;
use vulkano::pipeline::shader::{
    GraphicsShaderType, ShaderInterface, ShaderInterfaceEntry, ShaderModule, ShaderStages,
    SpecializationConstants,
};
use vulkano::pipeline::viewport::{Scissor, Viewport};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineCreationError};
use vulkano::render_pass::Subpass;
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode, SamplerCreationError};
use vulkano::sync::GpuFuture;
use vulkano::OomError;

/// A loaded vulkan texture and it's width/height
pub struct Image {
    /// The immutable image type, represents the data loaded onto the GPU.
    ///
    /// Uses a dynamic format for flexibility on the kinds of images that might be loaded.
    pub image_access: Arc<ImmutableImage>,
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
//shader interface def entries
fn create_shader_interface_vs_in() -> ShaderInterface {
    unsafe {
        ShaderInterface::new_unchecked(vec![
            ShaderInterfaceEntry {
                location: 0..1,
                format: Format::R32G32_SFLOAT,
                name: Some(std::borrow::Cow::Borrowed("position")),
            },
            ShaderInterfaceEntry {
                location: 1..2,
                format: Format::R32G32_SFLOAT,
                name: Some(std::borrow::Cow::Borrowed("tex_coords")),
            },
            ShaderInterfaceEntry {
                location: 2..3,
                format: Format::R32G32B32A32_SFLOAT,
                name: Some(std::borrow::Cow::Borrowed("rgba")),
            },
            ShaderInterfaceEntry {
                location: 3..4,
                format: Format::R32_UINT,
                name: Some(std::borrow::Cow::Borrowed("mode")),
            },
        ])
    }
}
fn create_shader_interface_vs_out() -> ShaderInterface {
    unsafe {
        ShaderInterface::new_unchecked(vec![
            ShaderInterfaceEntry {
                location: 0..1,
                format: Format::R32G32_SFLOAT,
                name: Some(std::borrow::Cow::Borrowed("v_Uv")),
            },
            ShaderInterfaceEntry {
                location: 1..2,
                format: Format::R32G32B32A32_SFLOAT,
                name: Some(std::borrow::Cow::Borrowed("v_Color")),
            },
            ShaderInterfaceEntry {
                location: 2..3,
                format: Format::R32_UINT,
                name: Some(std::borrow::Cow::Borrowed("v_Mode")),
            },
        ])
    }
}
fn create_shader_interface_fs_out() -> ShaderInterface {
    unsafe {
        ShaderInterface::new_unchecked(vec![ShaderInterfaceEntry {
            location: 0..1,
            format: Format::R32G32B32A32_SFLOAT,
            name: Some(std::borrow::Cow::Borrowed("Target0")),
        }])
    }
}

/// A type used for translating `render::Primitives` into `Command`s that indicate how to draw the
/// conrod GUI using `vulkano`.
pub struct Renderer {
    pipeline: Arc<GraphicsPipeline>,
    glyph_uploads: Arc<CpuBufferPool<u8>>,
    glyph_cache_tex: Arc<StorageImage>,
    sampler: Arc<Sampler>,
    tex_descs: SingleLayoutDescSetPool,
    mesh: Mesh,
}

/// An command for uploading an individual glyph.
pub struct GlyphCacheCommand<'a> {
    /// The CPU buffer containing the pixel data.
    pub glyph_cache_pixel_buffer: &'a [u8],
    /// The cpu buffer pool used to upload glyph pixels.
    pub glyph_cpu_buffer_pool: Arc<CpuBufferPool<u8>>,
    /// The GPU image to which the glyphs are cached.
    pub glyph_cache_texture: Arc<StorageImage>,
}

/// A draw command that maps directly to the `AutoCommandBufferBuilder::draw` method. By returning
/// `DrawCommand`s, we can avoid consuming the entire `AutoCommandBufferBuilder` itself which might
/// not always be available from APIs that wrap Vulkan.
pub struct DrawCommand {
    pub graphics_pipeline: Arc<GraphicsPipeline>,
    pub scissor: Scissor,
    pub viewport: Viewport,
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
    DescriptorSet(DescriptorSetError),

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
    pub fn new(
        device: Arc<Device>,
        subpass: Subpass,
        graphics_queue_family: QueueFamily,
        window_dims: [u32; 2],
        dpi_factor: f64,
    ) -> Result<Self, RendererCreationError> {
        // TODO: Check that necessary subpass properties exist?
        let [w, h] = window_dims;
        let glyph_cache_dims = [
            (w as f64 * dpi_factor) as u32,
            (h as f64 * dpi_factor) as u32,
        ];
        Self::with_glyph_cache_dimensions(device, subpass, graphics_queue_family, glyph_cache_dims)
    }

    /// Construct a new empty `Renderer`.
    pub fn with_glyph_cache_dimensions(
        device: Arc<Device>,
        subpass: Subpass,
        graphics_queue_family: QueueFamily,
        glyph_cache_dims: [u32; 2],
    ) -> Result<Self, RendererCreationError> {
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
        let descriptor_set_desc = DescriptorSetDesc::new(vec![Some(DescriptorDesc {
            ty: DescriptorDescTy::CombinedImageSampler {
                image_desc: DescriptorDescImage {
                    format: None,
                    multisampled: false,
                    view_type: ImageViewType::Dim2d,
                },
                immutable_samplers: vec![],
            },
            stages: ShaderStages {
                vertex: false,
                tessellation_control: false,
                tessellation_evaluation: false,
                geometry: false,
                fragment: true,
                compute: false,
            },
            variable_count: false,
            descriptor_count: 1,
            mutable: false,
        })]);
        let descriptor_set_layout = Arc::new(
            DescriptorSetLayout::new(device.clone(), descriptor_set_desc.clone()).unwrap(),
        );
        let layout = Arc::new(
            PipelineLayout::new(device.clone(), vec![descriptor_set_layout], vec![]).unwrap(),
        );
        let vs_module =
            unsafe { ShaderModule::new(device.clone(), include_bytes!("shaders/vert.spv")) }?;

        let fs_module =
            unsafe { ShaderModule::new(device.clone(), include_bytes!("shaders/frag.spv")) }?;
        let main = CString::new("main").unwrap();
        let vs_in = create_shader_interface_vs_in();
        let vs_out = create_shader_interface_vs_out();
        let fs_out = create_shader_interface_fs_out();
        let vs = unsafe {
            vs_module.graphics_entry_point(
                &main,
                vec![descriptor_set_desc.clone()],
                None,
                <()>::descriptors(),
                vs_in,
                vs_out.clone(),
                GraphicsShaderType::Vertex,
            )
        };
        let fs = unsafe {
            fs_module.graphics_entry_point(
                &main,
                vec![descriptor_set_desc.clone()],
                None,
                <()>::descriptors(),
                vs_out,
                fs_out,
                GraphicsShaderType::Fragment,
            )
        };

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs, ())
                .depth_stencil_disabled()
                .triangle_list()
                .front_face_clockwise()
                .viewports_scissors_dynamic(1)
                .fragment_shader(fs, ())
                .blend_alpha_blending()
                .render_pass(subpass)
                .with_pipeline_layout(device.clone(), layout)?,
        );
        let mesh = Mesh::with_glyph_cache_dimensions(glyph_cache_dims);

        let glyph_cache_tex = {
            let [width, height] = glyph_cache_dims;
            StorageImage::with_usage(
                device.clone(),
                ImageDimensions::Dim2d {
                    width,
                    height,
                    array_layers: 1,
                },
                Format::R8_UNORM,
                ImageUsage {
                    transfer_destination: true,
                    sampled: true,
                    ..ImageUsage::none()
                },
                ImageCreateFlags {
                    sparse_binding: false,
                    sparse_residency: false,
                    sparse_aliased: false,
                    mutable_format: false,
                    cube_compatible: false,
                    array_2d_compatible: false,
                    block_texel_view_compatible: false,
                },
                vec![graphics_queue_family],
            )?
        };

        let tex_descs =
            SingleLayoutDescSetPool::new(pipeline.layout().descriptor_set_layouts()[0].clone());
        let glyph_uploads = Arc::new(CpuBufferPool::upload(device));

        Ok(Renderer {
            pipeline,
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
    pub fn fill<P: render::PrimitiveWalker>(
        &mut self,
        image_map: &image::Map<Image>,
        viewport: [f32; 4],
        dpi_factor: f64,
        primitives: P,
    ) -> Result<Option<GlyphCacheCommand>, rt::gpu_cache::CacheWriteErr> {
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
            origin: [viewport[0] as u32, viewport[1] as u32],
            dimensions: [
                (viewport[2] - viewport[0]) as u32,
                (viewport[3] - viewport[1]) as u32,
            ],
        };

        let conv_scizzor = |s: mesh::Scizzor| Scissor {
            origin: [s.top_left[0] as u32, s.top_left[1] as u32],
            dimensions: s.dimensions,
        };
        let mut desc_set_builder = self.tex_descs.next();
        desc_set_builder.add_sampled_image(
            ImageView::new(self.glyph_cache_tex.clone()).unwrap(),
            self.sampler.clone(),
        )?;
        let desc_cache = Arc::new(desc_set_builder.build()?);

        let commands = self.mesh.commands();

        let mut draw_commands = vec![];

        for command in commands {
            match command {
                // Update the `scizzor` before continuing to draw.
                mesh::Command::Scizzor(scizzor) => current_scizzor = conv_scizzor(scizzor),

                // Draw to the target with the given `draw` command.
                mesh::Command::Draw(draw) => match draw {
                    // Draw text and plain 2D geometry.
                    mesh::Draw::Plain(vert_range) => {
                        if !vert_range.is_empty() {
                            let verts = &self.mesh.vertices()[vert_range];
                            let verts = conv_vertex_buffer(verts);
                            let (vbuf, vbuf_fut) = ImmutableBuffer::<[Vertex]>::from_iter(
                                verts.iter().cloned(),
                                BufferUsage::vertex_buffer(),
                                queue.clone(),
                            )?;
                            vbuf_fut
                                .then_signal_fence_and_flush()
                                .expect("failed to flush future")
                                .wait(None)
                                .unwrap();
                            draw_commands.push(DrawCommand {
                                graphics_pipeline: self.pipeline.clone(),
                                scissor: current_scizzor,
                                viewport: current_viewport.clone(),
                                vertex_buffer: vbuf,
                                descriptor_set: desc_cache.clone(),
                            });
                        }
                    }

                    // Draw an image whose texture data lies within the `image_map` at the
                    // given `id`.
                    mesh::Draw::Image(image_id, vert_range) => {
                        if vert_range.is_empty() {
                            continue;
                        }
                        if let Some(image) = image_map.get(&image_id) {
                            let mut desc_set_builder = self.tex_descs.next();
                            desc_set_builder.add_sampled_image(
                                ImageView::new(image.image_access.clone()).unwrap(),
                                self.sampler.clone(),
                            )?;
                            let desc_image = Arc::new(desc_set_builder.build()?);

                            let verts = &self.mesh.vertices()[vert_range];
                            let verts = conv_vertex_buffer(verts);
                            let (vbuf, vbuf_fut) = ImmutableBuffer::from_iter(
                                verts.iter().cloned(),
                                BufferUsage::vertex_buffer(),
                                queue.clone(),
                            )?;
                            vbuf_fut
                                .then_signal_fence_and_flush()
                                .unwrap()
                                .wait(None)
                                .unwrap();
                            draw_commands.push(DrawCommand {
                                graphics_pipeline: self.pipeline.clone(),
                                scissor: current_scizzor,
                                viewport: current_viewport.clone(),
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
    unsafe { &*(buffer as *const [conrod_core::mesh::Vertex] as *const [Vertex]) }
}

impl From<vulkano::OomError> for RendererCreationError {
    fn from(err: OomError) -> Self {
        RendererCreationError::ShaderLoad(err)
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

impl From<DescriptorSetError> for DrawError {
    fn from(err: DescriptorSetError) -> Self {
        DrawError::DescriptorSet(err)
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
            DrawError::DescriptorSet(ref err) => Some(err),
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
            DrawError::DescriptorSet(ref err) => err.fmt(f),
            DrawError::VertexBufferAlloc(ref err) => err.fmt(f),
        }
    }
}
