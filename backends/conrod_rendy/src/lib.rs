pub mod winit_convert;

use conrod_core::image::{Id, Map};
use conrod_core::render::{Primitive, PrimitiveKind, PrimitiveWalker};
use conrod_core::{color, Scalar, Ui};
use image;
use rendy::command::{QueueId, RenderPassEncoder};
use rendy::core::{hal, hal::pso::CreationError, types::Layout};
use rendy::factory::{Factory, ImageState};
use rendy::graph::{render::*, GraphContext, NodeBuffer, NodeImage};
use rendy::hal::{
    device::Device,
    format::Format,
    image::{Kind, ViewKind},
    pso::{DepthStencilDesc, Element, VertexInputRate},
    Backend,
};
use rendy::memory::Dynamic;
use rendy::resource::{Buffer, BufferInfo, DescriptorSet, DescriptorSetLayout, Escape, Handle};
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

pub struct ConrodImage {
    texture_builder: TextureBuilder<'static>,
    width: u32,
    height: u32,
}

impl ConrodImage {
    pub fn new(path: PathBuf) -> Result<ConrodImage, image::ImageError> {
        let image = image::open(path)?.to_rgba();
        let (width, height) = image.dimensions();

        let texture_builder = TextureBuilder::new()
            .with_raw_data(image.into_vec(), Format::Rgba8Srgb)
            .with_data_width(width)
            .with_data_height(height)
            .with_kind(Kind::D2(width, height, 1, 1))
            .with_view_kind(ViewKind::D2);

        Ok(ConrodImage {
            texture_builder,
            width,
            height,
        })
    }
}

pub struct ConrodAux {
    pub ui: Ui,
    pub image_map: Map<ConrodImage>,
    pub image_id: Id,
}

#[derive(Debug, Default)]
pub struct ConrodPipelineDesc;

#[derive(Debug)]
pub struct ConrodPipeline<B: Backend> {
    descriptor_set: Escape<DescriptorSet<B>>,
    buffer: Option<Escape<Buffer<B>>>,
    texture: Option<Texture<B>>,
    vertice_count: u32,
}

impl<B> SimpleGraphicsPipelineDesc<B, ConrodAux> for ConrodPipelineDesc
where
    B: Backend,
{
    type Pipeline = ConrodPipeline<B>;

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

    fn load_shader_set(&self, factory: &mut Factory<B>, _aux: &ConrodAux) -> ShaderSet<B> {
        SHADERS.build(factory, Default::default()).unwrap()
    }

    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
        aux: &ConrodAux,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<Self::Pipeline, CreationError> {
        let descriptor_set = factory
            .create_descriptor_set(set_layouts[0].clone())
            .unwrap();

        let image = aux.image_map.get(&aux.image_id);
        let texture = if let Some(image) = image {
            let texture = image
                .texture_builder
                .build(
                    ImageState {
                        queue,
                        stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                        access: hal::image::Access::SHADER_READ,
                        layout: hal::image::Layout::ShaderReadOnlyOptimal,
                    },
                    factory,
                )
                .unwrap();

            unsafe {
                factory
                    .device()
                    .write_descriptor_sets(vec![hal::pso::DescriptorSetWrite {
                        set: descriptor_set.raw(),
                        binding: 0,
                        array_offset: 0,
                        descriptors: vec![hal::pso::Descriptor::CombinedImageSampler(
                            texture.view().raw(),
                            hal::image::Layout::ShaderReadOnlyOptimal,
                            texture.sampler().raw(),
                        )],
                    }]);
            }
            Some(texture)
        } else {
            None
        };

        Ok(ConrodPipeline {
            descriptor_set,
            buffer: None,
            texture,
            vertice_count: 0,
        })
    }
}

impl<B> SimpleGraphicsPipeline<B, ConrodAux> for ConrodPipeline<B>
where
    B: Backend,
{
    type Desc = ConrodPipelineDesc;

    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[Handle<DescriptorSetLayout<B>>],
        _index: usize,
        aux: &ConrodAux,
    ) -> PrepareResult {
        let mut vertices = vec![];

        if let Some(mut primitives) = aux.ui.draw_if_changed() {
            while let Some(primitive) = primitives.next_primitive() {
                vertices.append(&mut self.fill(aux, primitive));
            }

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
        }
        PrepareResult::DrawRecord
    }

    fn draw(
        &mut self,
        layout: &<B as Backend>::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        _index: usize,
        _aux: &ConrodAux,
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

    fn dispose(self, _factory: &mut Factory<B>, _aux: &ConrodAux) {}
}

impl<B: Backend> ConrodPipeline<B> {
    fn fill(&self, aux: &ConrodAux, primitive: Primitive) -> Vec<Vertex> {
        // TODO: Get the dpi factor
        let dpi_factor = 1.0;
        let half_win_w = aux.ui.win_w / 2.0;
        let half_win_h = aux.ui.win_h / 2.0;

        // Functions for converting for conrod scalar coords to GL vertex coords (-1.0 to 1.0).
        let vx = |x: Scalar| (x * dpi_factor / half_win_w) as f32;
        let vy = |y: Scalar| -1.0 * (y * dpi_factor / half_win_h) as f32;

        let mut vertices = vec![];

        match primitive.kind {
            PrimitiveKind::Rectangle { color } => {
                let color = color.to_fsa();
                let (l, r, b, t) = primitive.rect.l_r_b_t();

                let v = |x, y| {
                    // Convert from conrod Scalar range to GL range -1.0 to 1.0.
                    Vertex {
                        pos: [vx(x), vy(y)],
                        uv: [0.0, 0.0],
                        color,
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
            PrimitiveKind::TrianglesSingleColor { color, triangles } => {
                let v = |p: [Scalar; 2]| Vertex {
                    pos: [vx(p[0]), vy(p[1])],
                    uv: [0.0, 0.0],
                    color: color.into(),
                    mode: MODE_GEOMETRY,
                };

                for triangle in triangles {
                    vertices.push(v(triangle[0]));
                    vertices.push(v(triangle[1]));
                    vertices.push(v(triangle[2]));
                }
            }
            PrimitiveKind::TrianglesMultiColor { triangles } => {
                let v = |(p, c): ([Scalar; 2], color::Rgba)| Vertex {
                    pos: [vx(p[0]), vy(p[1])],
                    uv: [0.0, 0.0],
                    color: c.into(),
                    mode: MODE_GEOMETRY,
                };

                for triangle in triangles {
                    vertices.push(v(triangle[0]));
                    vertices.push(v(triangle[1]));
                    vertices.push(v(triangle[2]));
                }
            }
            PrimitiveKind::Image {
                image_id,
                color,
                source_rect,
            } => {
                if let Some(image_ref) = aux.image_map.get(&image_id) {
                    let color = color.unwrap_or(color::WHITE).to_fsa();
                    let (image_w, image_h) = (image_ref.width, image_ref.height);
                    let (image_w, image_h) = (image_w as Scalar, image_h as Scalar);

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
                    let (l, r, b, t) = primitive.rect.l_r_b_t();

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
            PrimitiveKind::Text {
                color,
                text,
                font_id,
            } => {
                let positioned_glyphs = text.positioned_glyphs(dpi_factor as f32);
            }
            _ => {}
        }
        vertices
    }
}
