use conrod_core::{
    image,
    mesh::{self, Mesh},
    render,
    text::rt,
    Rect,
    Scalar,
};
use std::collections::HashMap;

/// A loaded wgpu texture and it's width/height
pub struct Image {
    /// The immutable image type, represents the data loaded onto the GPU.
    ///
    /// Uses a dynamic format for flexibility on the kinds of images that might be loaded.
    pub texture: wgpu::Texture,
    /// The format of the `texture`.
    pub texture_format: wgpu::TextureFormat,
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

/// A helper type aimed at simplifying the rendering of conrod primitives via wgpu.
pub struct Renderer {
    _vs_mod: wgpu::ShaderModule,
    _fs_mod: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    render_pipeline: wgpu::RenderPipeline,
    glyph_cache_tex: wgpu::Texture,
    _default_image_tex: wgpu::Texture,
    default_bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    mesh: Mesh,
    // In order to support a dynamic number of images we maintain a unique bind group for each.
    bind_groups: HashMap<image::Id, wgpu::BindGroup>,
}

/// An command for uploading an individual glyph.
pub struct GlyphCacheCommand<'a> {
    /// The CPU buffer containing the pixel data.
    pub glyph_cache_pixel_buffer: &'a [u8],
    /// The GPU image to which the glyphs are cached.
    pub glyph_cache_texture: &'a wgpu::Texture,
    /// The width of the texture.
    pub width: u32,
    /// The height of the texture.
    pub height: u32,
}

/// A render produced by the `Renderer::render` method.
pub struct Render<'a> {
    pub pipeline: &'a wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub commands: Vec<RenderPassCommand<'a>>,
}

/// A draw command that maps directly to the `wgpu::CommandEncoder` method. By returning
/// `RenderPassCommand`s, we can avoid consuming the entire `AutoCommandBufferBuilder` itself which might
/// not always be available from APIs that wrap Vulkan.
pub enum RenderPassCommand<'a> {
    /// Specify the rectangle to which drawing should be cropped.
    SetScissor {
        top_left: [u32; 2],
        dimensions: [u32; 2],
    },
    /// Draw the specified range of vertices.
    Draw {
        vertex_range: std::ops::Range<u32>,
    },
    /// A new image requires drawing and in turn a new bind group requires setting.
    SetBindGroup {
        bind_group: &'a wgpu::BindGroup,
    },
}

const GLYPH_TEX_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
const DEFAULT_IMAGE_TEX_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

impl mesh::ImageDimensions for Image {
    fn dimensions(&self) -> [u32; 2] {
        [self.width, self.height]
    }
}

impl Renderer {
    /// Construct a new `Renderer`.
    pub fn new(device: &wgpu::Device, msaa_samples: u32) -> Self {
        let glyph_cache_dims = mesh::DEFAULT_GLYPH_CACHE_DIMS;
        Self::with_glyph_cache_dimensions(device, msaa_samples, glyph_cache_dims)
    }

    /// Create a renderer with a specific size for the glyph cache.
    pub fn with_glyph_cache_dimensions(
        device: &wgpu::Device,
        msaa_samples: u32,
        glyph_cache_dims: [u32; 2],
    ) -> Self {
        assert_eq!(glyph_cache_dims[0] % 256, 0, "wgpu glyph cache width must be multiple of 256");

        // The mesh for converting primitives into vertices.
        let mesh = Mesh::with_glyph_cache_dimensions(glyph_cache_dims);

        // Load shader modules.
        let vs = include_bytes!("shaders/vert.spv");
        let vs_spirv = wgpu::read_spirv(std::io::Cursor::new(&vs[..]))
            .expect("failed to read hard-coded SPIRV");
        let vs_mod = device.create_shader_module(&vs_spirv);
        let fs = include_bytes!("shaders/frag.spv");
        let fs_spirv = wgpu::read_spirv(std::io::Cursor::new(&fs[..]))
            .expect("failed to read hard-coded SPIRV");
        let fs_mod = device.create_shader_module(&fs_spirv);

        // Create the glyph cache texture.
        let glyph_cache_tex_desc = glyph_cache_tex_desc(glyph_cache_dims);
        let glyph_cache_tex = device.create_texture(&glyph_cache_tex_desc);

        // Create the sampler for sampling from the glyph cache and image textures.
        let sampler_desc = sampler_desc();
        let sampler = device.create_sampler(&sampler_desc);

        // Create the render pipeline.
        let bind_group_layout = bind_group_layout(device);
        let pipeline_layout = pipeline_layout(device, &bind_group_layout);
        let render_pipeline =
            render_pipeline(device, &pipeline_layout, &vs_mod, &fs_mod, msaa_samples);

        // Create the default image that is bound to `image_texture` along with a default bind
        // group for use in the case that there are no user supplied images.
        let default_image_tex_desc = default_image_tex_desc();
        let default_image_tex = device.create_texture(&default_image_tex_desc);
        let default_bind_group = bind_group(
            device,
            &bind_group_layout,
            &glyph_cache_tex,
            &sampler,
            &default_image_tex,
        );

        // The empty set of bind groups to be associated with user images.
        let bind_groups = Default::default();

        Self {
            _vs_mod: vs_mod,
            _fs_mod: fs_mod,
            glyph_cache_tex,
            _default_image_tex: default_image_tex,
            default_bind_group,
            sampler,
            bind_group_layout,
            bind_groups,
            render_pipeline,
            mesh,
        }
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
    pub fn fill<'a, P>(
        &'a mut self,
        image_map: &image::Map<Image>,
        viewport: [f32; 4],
        scale_factor: f64,
        primitives: P,
    ) -> Result<Option<GlyphCacheCommand<'a>>, rt::gpu_cache::CacheWriteErr>
    where
        P: render::PrimitiveWalker,
    {
        // Convert the given primitives into vertices.
        let [vp_l, vp_t, vp_r, vp_b] = viewport;
        let lt = [vp_l as Scalar, vp_t as Scalar];
        let rb = [vp_r as Scalar, vp_b as Scalar];
        let viewport = Rect::from_corners(lt, rb);
        let fill = self.mesh.fill(viewport, scale_factor, image_map, primitives)?;

        // Check whether or not we need a glyph cache update.
        let glyph_cache_cmd = match fill.glyph_cache_requires_upload {
            false => None,
            true => {
                let (width, height) = self.mesh.glyph_cache().dimensions();
                Some(GlyphCacheCommand {
                    glyph_cache_pixel_buffer: self.mesh.glyph_cache_pixel_buffer(),
                    glyph_cache_texture: &self.glyph_cache_tex,
                    width,
                    height,
                })
            },
        };
        Ok(glyph_cache_cmd)
    }

    /// Converts the inner list of `Command`s generated via `fill` to a list of
    /// `RenderPassCommand`s that are easily digestible by a `wgpu::RenderPass` produced by a
    /// `wgpu::CommandEncoder`.
    pub fn render(&mut self, device: &wgpu::Device, image_map: &image::Map<Image>) -> Render {
        let mut commands = vec![];

        // Ensure we have a descriptor set ready for each image in the map and no more.
        self.bind_groups.retain(|k, _| image_map.contains_key(k));
        for (id, img) in image_map.iter() {
            if self.bind_groups.contains_key(id) {
                continue;
            }
            let bind_group = bind_group(
                device,
                &self.bind_group_layout,
                &self.glyph_cache_tex,
                &self.sampler,
                &img.texture,
            );
            self.bind_groups.insert(*id, bind_group);
        }

        // Prepare a single vertex buffer containing all vertices for all geometry.
        let vertices = self.mesh.vertices();
        let vertices = conv_vertex_buffer(vertices);
        let vertex_buffer = device
            .create_buffer_mapped(vertices.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(vertices);

        // Keep track of the currently set bind group.
        #[derive(PartialEq)]
        enum BindGroup { Default, Image(image::Id) }
        let mut bind_group = None;

        for command in self.mesh.commands() {
            match command {
                // Update the `scizzor` before continuing to draw.
                mesh::Command::Scizzor(s) => {
                    let top_left = [s.top_left[0] as u32, s.top_left[1] as u32];
                    let dimensions = s.dimensions;
                    let cmd = RenderPassCommand::SetScissor { top_left, dimensions };
                    commands.push(cmd);
                }

                // Draw to the target with the given `draw` command.
                mesh::Command::Draw(draw) => match draw {
                    // Draw text and plain 2D geometry.
                    mesh::Draw::Plain(vertex_range) => {
                        let vertex_count = vertex_range.len();
                        if vertex_count <= 0 {
                            continue;
                        }
                        // Ensure a bind group is set.
                        if bind_group.is_none() {
                            bind_group = Some(BindGroup::Default);
                            let cmd = RenderPassCommand::SetBindGroup {
                                bind_group: &self.default_bind_group
                            };
                            commands.push(cmd);
                        }
                        let cmd = RenderPassCommand::Draw {
                            vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                        };
                        commands.push(cmd);
                    }

                    // Draw an image whose texture data lies within the `image_map` at the
                    // given `id`.
                    mesh::Draw::Image(image_id, vertex_range) => {
                        let vertex_count = vertex_range.len();
                        if vertex_count == 0 {
                            continue;
                        }
                        // Ensure the bind group matches this image.
                        let expected_bind_group = Some(BindGroup::Image(image_id));
                        if bind_group != expected_bind_group {
                            bind_group = expected_bind_group;
                            let cmd = RenderPassCommand::SetBindGroup {
                                bind_group: &self.bind_groups[&image_id],
                            };
                            commands.push(cmd);
                        }
                        let cmd = RenderPassCommand::Draw {
                            vertex_range: vertex_range.start as u32..vertex_range.end as u32,
                        };
                        commands.push(cmd);
                    }
                }
            }
        }

        Render {
            pipeline: &self.render_pipeline,
            vertex_buffer,
            commands,
        }
    }
}

impl<'a> GlyphCacheCommand<'a> {
    /// Creates a buffer on the GPU loaded with the updated pixel data.
    ///
    /// Created with `BufferUsage::COPY_SRC`, ready to be copied to the texture.
    ///
    /// TODO: In the future, we should consider re-using the same buffer and writing to it via
    /// `Buffer::map_write_async`. When asking about how to ensure that the write completes before
    /// the following `copy_buffer_to_texture` command, I was advised to just create a new buffer
    /// each time instead for now.
    /// EDIT:
    /// > if you try to map an existing buffer, it will give it to you only after all the gpu use
    /// > of the buffer is over. So you can't do it every frame reasonably
    pub fn create_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let len = self.glyph_cache_pixel_buffer.len();
        device
            .create_buffer_mapped(len, wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&self.glyph_cache_pixel_buffer)
    }

    /// Create the copy view ready for copying the pixel data to the texture.
    pub fn buffer_copy_view<'b>(&self, buffer: &'b wgpu::Buffer) -> wgpu::BufferCopyView<'b> {
        wgpu::BufferCopyView {
            buffer,
            offset: 0,
            row_pitch: self.width,
            image_height: self.height,
        }
    }

    /// Create the texture copy view ready for receiving the pixel data from the buffer.
    pub fn texture_copy_view(&self) -> wgpu::TextureCopyView {
        wgpu::TextureCopyView {
            texture: &self.glyph_cache_texture,
            mip_level: 0,
            array_layer: 0,
            origin: wgpu::Origin3d::ZERO,
        }
    }

    /// Encode the command for copying the buffer's pixel data to the glyph cache texture.
    pub fn encode(&self, buffer: &wgpu::Buffer, encoder: &mut wgpu::CommandEncoder) {
        let buffer_copy_view = self.buffer_copy_view(&buffer);
        let texture_copy_view = self.texture_copy_view();
        let extent = self.extent();
        encoder.copy_buffer_to_texture(buffer_copy_view, texture_copy_view, extent);
    }

    /// The extent required for the copy command.
    pub fn extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth: 1,
        }
    }

    /// Short-hand for `create_buffer` and `encode` in succession.
    pub fn load_buffer_and_encode(&self, device: &wgpu::Device, e: &mut wgpu::CommandEncoder) {
        let buffer = self.create_buffer(&device);
        self.encode(&buffer, e);
    }
}

fn glyph_cache_tex_desc([width, height]: [u32; 2]) -> wgpu::TextureDescriptor {
    let depth = 1;
    let texture_extent = wgpu::Extent3d { width, height, depth };
    wgpu::TextureDescriptor {
        size: texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: GLYPH_TEX_FORMAT,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    }
}

fn default_image_tex_desc() -> wgpu::TextureDescriptor {
    let width = 64;
    let height = 64;
    let depth = 1;
    let texture_extent = wgpu::Extent3d { width, height, depth };
    wgpu::TextureDescriptor {
        size: texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEFAULT_IMAGE_TEX_FORMAT,
        usage: wgpu::TextureUsage::SAMPLED,
    }
}

fn sampler_desc() -> wgpu::SamplerDescriptor {
    wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        lod_min_clamp: -100.0,
        lod_max_clamp: 100.0,
        compare_function: wgpu::CompareFunction::Always,
    }
}

fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let glyph_cache_texture_binding = wgpu::BindGroupLayoutBinding {
        binding: 0,
        visibility: wgpu::ShaderStage::FRAGMENT,
        ty: wgpu::BindingType::SampledTexture {
            multisampled: false,
            dimension: wgpu::TextureViewDimension::D2,
        },
    };
    let sampler_binding = wgpu::BindGroupLayoutBinding {
        binding: 1,
        visibility: wgpu::ShaderStage::FRAGMENT,
        ty: wgpu::BindingType::Sampler,
    };
    let image_texture_binding = wgpu::BindGroupLayoutBinding {
        binding: 2,
        visibility: wgpu::ShaderStage::FRAGMENT,
        ty: wgpu::BindingType::SampledTexture {
            multisampled: false,
            dimension: wgpu::TextureViewDimension::D2,
        },
    };
    let bindings = &[glyph_cache_texture_binding, sampler_binding, image_texture_binding];
    let desc = wgpu::BindGroupLayoutDescriptor { bindings };
    device.create_bind_group_layout(&desc)
}

fn bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    glyph_cache_tex: &wgpu::Texture,
    sampler: &wgpu::Sampler,
    image: &wgpu::Texture,
) -> wgpu::BindGroup {
    // Glyph cache texture view.
    let glyph_cache_tex_view = glyph_cache_tex.create_default_view();
    let glyph_cache_tex_binding = wgpu::Binding {
        binding: 0,
        resource: wgpu::BindingResource::TextureView(&glyph_cache_tex_view),
    };

    // Sampler binding.
    let sampler_binding = wgpu::Binding {
        binding: 1,
        resource: wgpu::BindingResource::Sampler(&sampler),
    };

    // Image texture view.
    let image_tex_view = image.create_default_view();
    let image_tex_binding = wgpu::Binding {
        binding: 2,
        resource: wgpu::BindingResource::TextureView(&image_tex_view),
    };

    let bindings = &[glyph_cache_tex_binding, sampler_binding, image_tex_binding];
    let desc = wgpu::BindGroupDescriptor { layout, bindings };
    device.create_bind_group(&desc)
}

fn pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    let desc = wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    };
    device.create_pipeline_layout(&desc)
}

fn vertex_attrs() -> [wgpu::VertexAttributeDescriptor; 4] {
    let position_offset = 0;
    let position_size = std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress;
    let tex_coords_offset = position_offset + position_size;
    let tex_coords_size = position_size;
    let rgba_offset = tex_coords_offset + tex_coords_size;
    let rgba_size = std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress;
    let mode_offset = rgba_offset + rgba_size;
    [
        // position
        wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float2,
            offset: position_offset,
            shader_location: 0,
        },
        // tex_coords
        wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float2,
            offset: tex_coords_offset,
            shader_location: 1,
        },
        // rgba
        wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Float4,
            offset: rgba_offset,
            shader_location: 2,
        },
        // mode
        wgpu::VertexAttributeDescriptor {
            format: wgpu::VertexFormat::Uint,
            offset: mode_offset,
            shader_location: 3,
        },
    ]
}

fn render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    vs_mod: &wgpu::ShaderModule,
    fs_mod: &wgpu::ShaderModule,
    msaa_samples: u32,
) -> wgpu::RenderPipeline {
    let vs_desc = wgpu::ProgrammableStageDescriptor {
        module: &vs_mod,
        entry_point: "main",
    };
    let fs_desc = wgpu::ProgrammableStageDescriptor {
        module: &fs_mod,
        entry_point: "main",
    };
    let raster_desc = wgpu::RasterizationStateDescriptor {
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: wgpu::CullMode::None,
        depth_bias: 0,
        depth_bias_slope_scale: 0.0,
        depth_bias_clamp: 0.0,
    };
    let color_state_desc = wgpu::ColorStateDescriptor {
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        color_blend: wgpu::BlendDescriptor {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        alpha_blend: wgpu::BlendDescriptor {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        write_mask: wgpu::ColorWrite::ALL,
    };
    let vertex_attrs = vertex_attrs();
    let vertex_buffer_desc = wgpu::VertexBufferDescriptor {
        stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::InputStepMode::Vertex,
        attributes: &vertex_attrs[..],
    };
    let desc = wgpu::RenderPipelineDescriptor {
        layout,
        vertex_stage: vs_desc,
        fragment_stage: Some(fs_desc),
        rasterization_state: Some(raster_desc),
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[color_state_desc],
        depth_stencil_state: None,
        index_format: wgpu::IndexFormat::Uint16,
        vertex_buffers: &[vertex_buffer_desc],
        sample_count: msaa_samples,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    };
    device.create_render_pipeline(&desc)
}

fn conv_vertex_buffer(buffer: &[mesh::Vertex]) -> &[Vertex] {
    unsafe { std::mem::transmute(buffer) }
}
