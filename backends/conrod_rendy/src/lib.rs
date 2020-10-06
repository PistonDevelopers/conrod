use conrod_core::image::{Id as ImageId, Map as ImageMap};
use conrod_core::mesh::{self, Mesh};
use conrod_core::{Rect, Ui};
use rendy::command::{QueueId, RenderPassEncoder};
use rendy::core::{hal, hal::pso::CreationError, types::Layout};
use rendy::factory::{Factory, ImageState};
use rendy::graph::{render::*, GraphContext, NodeBuffer, NodeImage};
use rendy::hal::{
    device::Device,
    format::{Aspects, Format},
    image::{Kind, Offset, ViewKind},
    pso::{DepthStencilDesc, Element, VertexInputRate},
    Backend,
};
use rendy::memory::Dynamic;
use rendy::mesh::AsVertex;
use rendy::resource::{
    Buffer, BufferInfo, DescriptorSet, DescriptorSetLayout, Escape, Extent, Handle,
};
use rendy::shader::{ShaderSet, ShaderSetBuilder, SpirvShader};
use rendy::texture::{Texture, TextureBuilder};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use vertex::Vertex;

mod vertex;

/// Draw text from the text cache texture `tex` in the fragment shader.
pub const MODE_TEXT: u32 = 0;
/// Draw an image from the texture at `tex` in the fragment shader.
pub const MODE_IMAGE: u32 = 1;
/// Ignore `tex` and draw simple, colored 2D geometry.
pub const MODE_GEOMETRY: u32 = 2;

/// Requirements for the auxiliary type within rendy graphs containing a `UiPipeline` node.
pub trait UiAux {
    type Backend: Backend;

    /// The user interface to be drawn.
    fn ui(&self) -> &Ui;

    /// Access to the user's images.
    fn image_map(&self) -> &ImageMap<UiTexture<Self::Backend>>;

    /// The DPI factor for translating from conrod's pixel-agnostic coordinates to pixel
    /// coordinates for the underlying surface.
    fn dpi_factor(&self) -> f64;
}

lazy_static::lazy_static! {
    static ref VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("shaders/vert.spv"),
        hal::pso::ShaderStageFlags::VERTEX,
        "main",
    ).expect("failed to construct `SpirvShader` from bytes");

    static ref FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("shaders/frag.spv"),
        hal::pso::ShaderStageFlags::FRAGMENT,
        "main",
    ).expect("failed to construct `SpirvShader` from bytes");

    static ref SHADERS: ShaderSetBuilder = ShaderSetBuilder::default()
        .with_vertex(&*VERTEX).unwrap()
        .with_fragment(&*FRAGMENT).unwrap();
}

/// A simple type that wraps a rendy `Texture` and provides a `conrod_core::mesh::ImageDimensions`
/// implementation.
pub struct UiTexture<B>
where
    B: Backend,
{
    texture: Texture<B>,
}

#[derive(Debug)]
pub struct UiPipelineDesc {
    /// The dimensions with which the glyph cache should be initialised.
    pub glyph_cache_dimensions: [u32; 2],
}

#[derive(Debug)]
pub struct UiPipeline<B: Backend> {
    mesh: Mesh,
    // The default descriptor set to be bound.
    default_descriptor_set: Escape<DescriptorSet<B>>,
    // One descriptor set per image as each is drawn in a separate pass.
    // TODO: This is quite inefficient - we should use a dynamically packed texture atlas or
    // something instead of this.
    descriptor_sets: HashMap<ImageId, Escape<DescriptorSet<B>>>,
    buffer: Option<Escape<Buffer<B>>>,
    glyph_cache_texture: Texture<B>,
}

/// A simple, provided implementation of the `UiAux` trait.
///
/// Useful as a suitable auxiliary type when the `UiPipeline` is the only pipeline within the
/// rendy graph.
pub struct SimpleUiAux<B>
where
    B: Backend,
{
    pub ui: Ui,
    pub image_map: ImageMap<UiTexture<B>>,
    pub dpi_factor: f64,
}

impl<B> UiTexture<B>
where
    B: Backend,
{
    /// An optional, simplified constructor for loading a `UiTexture` from a slice of sRGBA bytes.
    pub fn from_rgba_bytes(
        bytes: &[u8],
        dimensions: [u32; 2],
        factory: &mut Factory<B>,
        queue_id: QueueId,
    ) -> Result<Self, rendy::texture::BuildError> {
        let [width, height] = dimensions;
        let img_state = sampler_img_state(queue_id);
        let texture = TextureBuilder::new()
            .with_raw_data(bytes, Format::Rgba8Srgb)
            .with_data_width(width)
            .with_data_height(height)
            .with_kind(Kind::D2(width, height, 1, 1))
            .with_view_kind(ViewKind::D2)
            .build(img_state, factory)?;
        Ok(UiTexture { texture })
    }
}

impl Default for UiPipelineDesc {
    fn default() -> Self {
        let glyph_cache_dimensions = conrod_core::mesh::DEFAULT_GLYPH_CACHE_DIMS;
        UiPipelineDesc {
            glyph_cache_dimensions,
        }
    }
}

impl<B> Deref for UiTexture<B>
where
    B: Backend,
{
    type Target = Texture<B>;
    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

impl<B> DerefMut for UiTexture<B>
where
    B: Backend,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.texture
    }
}

impl<B> From<Texture<B>> for UiTexture<B>
where
    B: Backend,
{
    fn from(texture: Texture<B>) -> Self {
        UiTexture { texture }
    }
}

impl<B> Into<Texture<B>> for UiTexture<B>
where
    B: Backend,
{
    fn into(self) -> Texture<B> {
        self.texture
    }
}

impl<B> mesh::ImageDimensions for UiTexture<B>
where
    B: Backend,
{
    fn dimensions(&self) -> [u32; 2] {
        match self.image().kind() {
            Kind::D1(w, _) => [w, 1],
            Kind::D2(w, h, _, _) => [w, h],
            Kind::D3(w, h, _) => [w, h],
        }
    }
}

impl<B> UiAux for SimpleUiAux<B>
where
    B: Backend,
{
    type Backend = B;

    fn ui(&self) -> &Ui {
        &self.ui
    }

    fn image_map(&self) -> &ImageMap<UiTexture<Self::Backend>> {
        &self.image_map
    }

    fn dpi_factor(&self) -> f64 {
        self.dpi_factor
    }
}

impl<B, T> SimpleGraphicsPipelineDesc<B, T> for UiPipelineDesc
where
    B: Backend,
    T: UiAux<Backend = B>,
{
    type Pipeline = UiPipeline<B>;

    fn depth_stencil(&self) -> Option<DepthStencilDesc> {
        None
    }

    fn vertices(&self) -> Vec<(Vec<Element<Format>>, u32, VertexInputRate)> {
        vec![Vertex::vertex().gfx_vertex_input_desc(hal::pso::VertexInputRate::Vertex)]
    }

    fn layout(&self) -> Layout {
        Layout {
            sets: vec![SetLayout {
                bindings: vec![
                    hal::pso::DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: hal::pso::DescriptorType::CombinedImageSampler,
                        count: 1,
                        stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                    hal::pso::DescriptorSetLayoutBinding {
                        binding: 1,
                        ty: hal::pso::DescriptorType::CombinedImageSampler,
                        count: 1,
                        stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                ],
            }],
            push_constants: Vec::new(),
        }
    }

    fn load_shader_set(&self, factory: &mut Factory<B>, _aux: &T) -> ShaderSet<B> {
        SHADERS.build(factory, Default::default()).unwrap()
    }

    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
        _aux: &T,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<Self::Pipeline, CreationError> {
        // TODO: Consider using `Mesh::with_glyph_cache_dimensions` and allowing user to specify
        // glyph cache dimensions. Currently we just use the default size, but this is not always
        // enough for large GUIs with lots of text.
        let mesh = Mesh::with_glyph_cache_dimensions(self.glyph_cache_dimensions);

        // Create the texture used for caching glyphs on the GPU.
        let sampler_img_state = sampler_img_state(queue);
        let (gc_width, gc_height) = mesh.glyph_cache().dimensions();
        let glyph_cache_texture = TextureBuilder::new()
            .with_raw_data(mesh.glyph_cache_pixel_buffer(), Format::R8Unorm)
            .with_data_width(gc_width)
            .with_data_height(gc_height)
            .with_kind(Kind::D2(gc_width, gc_height, 1, 1))
            .with_view_kind(ViewKind::D2)
            .build(sampler_img_state, factory)
            .expect("failed to create glyph cache texture");

        // Write the default descriptor set.
        let layout = set_layouts[0].clone();
        let default_descriptor_set = factory.create_descriptor_set(layout).unwrap();
        let glyph_cache_desc = create_glyph_cache_descriptor(&glyph_cache_texture);
        let descriptors = vec![glyph_cache_desc];
        let write = hal::pso::DescriptorSetWrite {
            set: default_descriptor_set.raw(),
            binding: 0,
            array_offset: 0,
            descriptors,
        };
        let writes = vec![write];
        unsafe {
            factory.device().write_descriptor_sets(writes);
        }

        let descriptor_sets = HashMap::new();

        Ok(UiPipeline {
            mesh,
            default_descriptor_set,
            descriptor_sets,
            buffer: None,
            glyph_cache_texture,
        })
    }
}

impl<B, T> SimpleGraphicsPipeline<B, T> for UiPipeline<B>
where
    B: Backend,
    T: UiAux<Backend = B>,
{
    type Desc = UiPipelineDesc;

    fn prepare(
        &mut self,
        factory: &Factory<B>,
        queue: QueueId,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
        _index: usize,
        aux: &T,
    ) -> PrepareResult {
        let ui = aux.ui();
        let dpi_factor = aux.dpi_factor();
        let image_map = aux.image_map();
        let viewport = Rect::from_xy_dim([0.0; 2], [ui.win_w, ui.win_h]);

        // Only continue preparation if something within the UI has visibly changed.
        let primitives = match aux.ui().draw_if_changed() {
            None => return PrepareResult::DrawReuse,
            Some(prims) => prims,
        };

        // Fill the mesh from the given primitives.
        let fill = self
            .mesh
            .fill(viewport, dpi_factor, image_map, primitives)
            .expect("failed to fill mesh");

        // If fill indicates the glyph cache needs updating, do so.
        if fill.glyph_cache_requires_upload {
            let (gc_width, gc_height) = self.mesh.glyph_cache().dimensions();
            let img_layers = rendy::resource::SubresourceLayers {
                aspects: Aspects::COLOR,
                level: 0,
                layers: 0..1,
            };
            let img_offset = Offset::ZERO;
            let img_extent = Extent {
                width: gc_width,
                height: gc_height,
                depth: 1,
            };
            let img_state = sampler_img_state(queue);
            let (last, next) = (img_state, img_state);
            unsafe {
                factory
                    .upload_image(
                        self.glyph_cache_texture.image().clone(),
                        gc_width,
                        gc_height,
                        img_layers,
                        img_offset,
                        img_extent,
                        self.mesh.glyph_cache_pixel_buffer(),
                        last,
                        next,
                    )
                    .expect("failed to update glyph cache texture");
            }
        }

        // Create new descriptor sets for new images.
        let image_map = aux.image_map();
        let glyph_cache_texture = &self.glyph_cache_texture;
        let descriptor_sets = &mut self.descriptor_sets;
        let new_textures: HashMap<_, _> = image_map
            .iter()
            .filter(|&(img_id, _)| !descriptor_sets.contains_key(img_id))
            .collect();
        let new_descriptors: HashMap<_, _> = new_textures
            .iter()
            .map(|(img_id, texture)| {
                let descriptors = create_descriptors(glyph_cache_texture, texture);
                let descriptor_set = factory
                    .create_descriptor_set(set_layouts[0].clone())
                    .unwrap();
                descriptor_sets.insert(**img_id, descriptor_set);
                (img_id, descriptors)
            })
            .collect();
        let new_writes: Vec<_> = new_descriptors
            .into_iter()
            .map(|(img_id, descriptors)| hal::pso::DescriptorSetWrite {
                set: descriptor_sets[img_id].raw(),
                binding: 0,
                array_offset: 0,
                descriptors,
            })
            .collect();
        if !new_writes.is_empty() {
            unsafe {
                factory.device().write_descriptor_sets(new_writes);
            }
        }

        // TODO: Remove this in favour of `unsafe`ly casting the `&[conrod_core::mesh::Vertex]`
        // to `&[Vertex]` after ensuring layouts are the same.
        let vertices: Vec<Vertex> = self
            .mesh
            .vertices()
            .iter()
            .map(|v| Vertex {
                pos: v.position.into(),
                uv: v.tex_coords.into(),
                color: v.rgba.into(),
                mode: v.mode.into(),
            })
            .collect();
        let buffer_size = Vertex::vertex().stride as u64 * vertices.len() as u64;
        let mut buffer = factory
            .create_buffer(
                BufferInfo {
                    size: buffer_size,
                    usage: hal::buffer::Usage::VERTEX,
                },
                Dynamic,
            )
            .unwrap();
        unsafe {
            // Fresh buffer.
            factory
                .upload_visible_buffer(&mut buffer, 0, &vertices)
                .unwrap();
        }
        self.buffer = Some(buffer);
        PrepareResult::DrawRecord
    }

    fn draw(
        &mut self,
        layout: &<B as Backend>::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        _index: usize,
        _aux: &T,
    ) {
        let buffer = match self.buffer {
            None => return,
            Some(ref b) => b,
        };

        unsafe {
            // Bind the default descriptor set.
            encoder.bind_graphics_descriptor_sets(
                layout,
                0,
                std::iter::once(self.default_descriptor_set.raw()),
                std::iter::empty::<u32>(),
            );
        }

        for cmd in self.mesh.commands() {
            match cmd {
                mesh::Command::Draw(draw) => match draw {
                    mesh::Draw::Image(img_id, v_range) => {
                        if v_range.len() == 0 {
                            continue;
                        }
                        let vertices_range = v_range.start as u32..v_range.end as u32;
                        let instances_range = 0..1;
                        unsafe {
                            // Bind the descriptor set associated with this image.
                            encoder.bind_graphics_descriptor_sets(
                                layout,
                                0,
                                std::iter::once(self.descriptor_sets[&img_id].raw()),
                                std::iter::empty::<u32>(),
                            );
                            encoder.bind_vertex_buffers(0, Some((buffer.raw(), 0)));
                            encoder.draw(vertices_range, instances_range);
                        }
                    }

                    mesh::Draw::Plain(v_range) => {
                        if v_range.len() == 0 {
                            continue;
                        }
                        let vertices_range = v_range.start as u32..v_range.end as u32;
                        let instances_range = 0..1;
                        unsafe {
                            encoder.bind_vertex_buffers(0, Some((buffer.raw(), 0)));
                            encoder.draw(vertices_range, instances_range);
                        }
                    }
                },

                mesh::Command::Scizzor(scizzor) => {
                    let mesh::Scizzor {
                        top_left,
                        dimensions,
                    } = scizzor;
                    let [x, y] = top_left;
                    let [w, h] = dimensions;
                    let rect = hal::pso::Rect {
                        x: x as i16,
                        y: y as i16,
                        w: w as i16,
                        h: h as i16,
                    };
                    let first_scissor = 0; // TODO: Clarify what this means, I'm just guessing.
                    unsafe {
                        encoder.set_scissors(first_scissor, Some(&rect));
                    }
                }
            }
        }
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &T) {}
}

fn sampler_img_state(queue_id: QueueId) -> ImageState {
    ImageState {
        queue: queue_id,
        stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
        access: hal::image::Access::SHADER_READ,
        layout: hal::image::Layout::ShaderReadOnlyOptimal,
    }
}

// Create the glyph cache texture sampler descriptor.
fn create_glyph_cache_descriptor<B>(glyph_cache_texture: &Texture<B>) -> hal::pso::Descriptor<B>
where
    B: Backend,
{
    hal::pso::Descriptor::CombinedImageSampler(
        glyph_cache_texture.view().raw(),
        hal::image::Layout::ShaderReadOnlyOptimal,
        glyph_cache_texture.sampler().raw(),
    )
}

// Create the glyph cache and image descriptors.
fn create_descriptors<'a, B>(
    glyph_cache_texture: &'a Texture<B>,
    image_texture: &'a Texture<B>,
) -> Vec<hal::pso::Descriptor<'a, B>>
where
    B: Backend,
{
    let glyph_cache_sampler_desc = create_glyph_cache_descriptor(glyph_cache_texture);
    let image_sampler_desc = hal::pso::Descriptor::CombinedImageSampler(
        image_texture.view().raw(),
        hal::image::Layout::ShaderReadOnlyOptimal,
        image_texture.sampler().raw(),
    );

    vec![glyph_cache_sampler_desc, image_sampler_desc]
}
