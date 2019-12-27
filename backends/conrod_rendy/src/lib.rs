pub mod winit_convert;

use conrod_core::image::Map;
use image;
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
use rendy::resource::{Buffer, BufferInfo, DescriptorSet, DescriptorSetLayout, Escape, Extent, Handle};
use rendy::shader::{
    ShaderKind, ShaderSet, ShaderSetBuilder, SourceLanguage, SourceShaderInfo, SpirvReflection,
    SpirvShader,
};
use rendy::texture::{Texture, TextureBuilder};
use std::path::PathBuf;

/// Draw text from the text cache texture `tex` in the fragment shader.
pub const MODE_TEXT: u32 = 0;
/// Draw an image from the texture at `tex` in the fragment shader.
pub const MODE_IMAGE: u32 = 1;
/// Ignore `tex` and draw simple, colored 2D geometry.
pub const MODE_GEOMETRY: u32 = 2;

/// Requirements for the auxiliary type within rendy graphs containing a `UiPipeline` node.
pub trait UiAux {
    /// The user interface to be drawn.
    fn ui(&self) -> &Ui;

    /// Access to the user's images.
    fn image_map(&self) -> &Map<UiImage>;

    /// The DPI factor for translating from conrod's pixel-agnostic coordinates to pixel
    /// coordinates for the underlying surface.
    fn dpi_factor(&self) -> f64;
}

lazy_static::lazy_static! {
    static ref VERTEX: SpirvShader = SourceShaderInfo::new(
    "
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
    ",
        "conrod.vert",
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref FRAGMENT: SpirvShader = SourceShaderInfo::new(
    "
#version 450
layout(set = 0, binding = 0) uniform sampler2D t_TextColor;
layout(set = 0, binding = 1) uniform sampler2D t_ImgColor;

layout(location = 0) in vec2 v_Uv;
layout(location = 1) in vec4 v_Color;
layout(location = 2) flat in uint v_Mode;

layout(location = 0) out vec4 Target0;

void main() {
    // Text
    if (v_Mode == uint(0)) {
        Target0 = v_Color * vec4(1.0, 1.0, 1.0, texture(t_TextColor, v_Uv).r);

    // Image
    } else if (v_Mode == uint(1)) {
        Target0 = texture(t_ImgColor, v_Uv);

    // 2D Geometry
    } else if (v_Mode == uint(2)) {
        Target0 = v_Color;
    }
}
    ",
        "conrod.frag",
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref SHADERS: ShaderSetBuilder = ShaderSetBuilder::default()
        .with_vertex(&*VERTEX).unwrap()
        .with_fragment(&*FRAGMENT).unwrap();

    static ref SHADER_REFLECTION: SpirvReflection = SHADERS.reflect().unwrap();
}

/// The `Vertex` type passed to the vertex shader.
#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub mode: u32,
}

pub struct UiImage {
    texture_builder: TextureBuilder<'static>,
    width: u32,
    height: u32,
}

impl UiImage {
    pub fn new(path: PathBuf) -> Result<UiImage, image::ImageError> {
        let image = image::open(path)?.to_rgba();
        let (width, height) = image.dimensions();

        let texture_builder = TextureBuilder::new()
            .with_raw_data(image.into_vec(), Format::Rgba8Srgb)
            .with_data_width(width)
            .with_data_height(height)
            .with_kind(Kind::D2(width, height, 1, 1))
            .with_view_kind(ViewKind::D2);

        Ok(UiImage {
            texture_builder,
            width,
            height,
        })
    }
}

#[derive(Debug, Default)]
pub struct UiPipelineDesc;

#[derive(Debug)]
pub struct UiPipeline<B: Backend> {
    mesh: Mesh,
    descriptor_set: Escape<DescriptorSet<B>>,
    buffer: Option<Escape<Buffer<B>>>,
    texture: Option<Texture<B>>,
    glyph_cache_texture: Texture<B>,
    vertice_count: u32,
}

/// A simple, provided implementation of the `UiAux` trait.
///
/// Useful as a suitable auxiliary type when the `UiPipeline` is the only pipeline within the
/// rendy graph.
pub struct SimpleUiAux {
    pub ui: Ui,
    pub image_map: Map<UiImage>,
    pub dpi_factor: f64,
}

impl UiAux for SimpleUiAux {
    fn ui(&self) -> &Ui {
        &self.ui
    }

    fn image_map(&self) -> &Map<UiImage> {
        &self.image_map
    }

    fn dpi_factor(&self) -> f64 {
        self.dpi_factor
    }
}

impl mesh::ImageDimensions for UiImage {
    fn dimensions(&self) -> [u32; 2] {
        [self.width, self.height]
    }
}

impl<B, T> SimpleGraphicsPipelineDesc<B, T> for UiPipelineDesc
where
    B: Backend,
    T: UiAux,
{
    type Pipeline = UiPipeline<B>;

    fn depth_stencil(&self) -> Option<DepthStencilDesc> {
        None
    }

    fn vertices(&self) -> Vec<(Vec<Element<Format>>, u32, VertexInputRate)> {
        vec![SHADER_REFLECTION
            .attributes_range(..)
            .unwrap()
            .gfx_vertex_input_desc(VertexInputRate::Vertex)]
    }

    fn layout(&self) -> Layout {
        SHADER_REFLECTION.layout().unwrap()
    }

    fn load_shader_set(&self, factory: &mut Factory<B>, _aux: &T) -> ShaderSet<B> {
        SHADERS.build(factory, Default::default()).unwrap()
    }

    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
        aux: &T,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<Self::Pipeline, CreationError> {
        // TODO: Consider using `Mesh::with_glyph_cache_dimensions` and allowing user to specify
        // glyph cache dimensions. Currently we just use the default size, but this is not always
        // enough for large GUIs with lots of text.
        let mesh = Mesh::new();

        let descriptor_set = factory
            .create_descriptor_set(set_layouts[0].clone())
            .unwrap();

        // Create the texture used for caching glyphs on the GPU.
        let sampler_img_state = sampler_img_state(queue);
        let (gc_width, gc_height) = mesh.glyph_cache().dimensions();
        let init_data = mesh.glyph_cache_pixel_buffer().to_vec();
        let glyph_cache_texture = TextureBuilder::new()
            .with_raw_data(init_data, Format::R8Unorm)
            .with_data_width(gc_width)
            .with_data_height(gc_height)
            .with_kind(Kind::D2(gc_width, gc_height, 1, 1))
            .with_view_kind(ViewKind::D2)
            .build(sampler_img_state, factory)
            .expect("failed to create glyph cache texture");

        // TODO: Move this into the `draw`?
        // Write the descriptor set.
        let glyph_cache_sampler_desc = hal::pso::Descriptor::CombinedImageSampler(
            glyph_cache_texture.view().raw(),
            hal::image::Layout::ShaderReadOnlyOptimal,
            glyph_cache_texture.sampler().raw(),
        );
        let mut descriptors = vec![glyph_cache_sampler_desc];

        let mut texture = None;
        if let Some(image) = aux.image_map().values().next() {
            texture = Some(
                image
                    .texture_builder
                    .build(sampler_img_state, factory)
                    .expect("failed to build texture for image")
            );
            let texture = texture.as_ref().unwrap();
            let sampler_desc = hal::pso::Descriptor::CombinedImageSampler(
                texture.view().raw(),
                hal::image::Layout::ShaderReadOnlyOptimal,
                texture.sampler().raw(),
            );
            descriptors.push(sampler_desc);
        }

        unsafe {
            factory
                .device()
                .write_descriptor_sets(vec![hal::pso::DescriptorSetWrite {
                    set: descriptor_set.raw(),
                    binding: 0,
                    array_offset: 0,
                    descriptors,
                }]);
        }

        Ok(UiPipeline {
            mesh,
            descriptor_set,
            buffer: None,
            glyph_cache_texture,
            texture,
            vertice_count: 0,
        })
    }
}

impl<B, T> SimpleGraphicsPipeline<B, T> for UiPipeline<B>
where
    B: Backend,
    T: UiAux,
{
    type Desc = UiPipelineDesc;

    fn prepare(
        &mut self,
        factory: &Factory<B>,
        queue: QueueId,
        _set_layouts: &[Handle<DescriptorSetLayout<B>>],
        _index: usize,
        aux: &T,
    ) -> PrepareResult {
        let ui = aux.ui();
        let dpi_factor = aux.dpi_factor();
        let image_map = aux.image_map();
        let primitives = match aux.ui().draw_if_changed() {
            None => return PrepareResult::DrawReuse,
            Some(prims) => prims,
        };

        let viewport = Rect::from_xy_dim([0.0; 2], [ui.win_w, ui.win_h]);
        let fill = self.mesh.fill(viewport, dpi_factor, image_map, primitives)
            .expect("failed to fill mesh");

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

        // TODO: Remove this in favour of `unsafe`ly casting the `&[conrod_core::mesh::Vertex]`
        // to `&[Vertex]` after ensuring layouts are the same.
        let vertices: Vec<Vertex> = self.mesh.vertices().iter().map(|v| {
            Vertex {
                pos: v.position,
                uv: v.tex_coords,
                color: v.rgba,
                mode: v.mode,
            }
        }).collect();

        let buffer_size = SHADER_REFLECTION.attributes_range(..).unwrap().stride as u64
            * vertices.len() as u64;

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
        self.vertice_count = vertices.len() as u32;
        PrepareResult::DrawRecord
    }

    fn draw(
        &mut self,
        layout: &<B as Backend>::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        _index: usize,
        _aux: &T,
    ) {
        if let Some(buffer) = &self.buffer {
            unsafe {
                encoder.bind_graphics_descriptor_sets(
                    layout,
                    0,
                    std::iter::once(self.descriptor_set.raw()),
                    std::iter::empty::<u32>(),
                );
                encoder.bind_vertex_buffers(0, Some((buffer.raw(), 0)));
                encoder.draw(0..self.vertice_count, 0..1);
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
