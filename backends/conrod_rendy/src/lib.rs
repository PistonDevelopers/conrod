use conrod_core::render::{PrimitiveKind, PrimitiveWalker};
use conrod_core::{color, Scalar, Ui};
use rendy::command::{QueueId, RenderPassEncoder};
use rendy::core::{hal, hal::pso::CreationError, types::Layout};
use rendy::factory::Factory;
use rendy::graph::{render::*, GraphContext, NodeBuffer, NodeImage};
use rendy::hal::{
    format::Format,
    pso::{DepthStencilDesc, Element, VertexInputRate},
    Backend,
};
use rendy::memory::Dynamic;
use rendy::resource::{Buffer, BufferInfo, DescriptorSetLayout, Escape, Handle};
use rendy::shader::{
    ShaderKind, ShaderSet, ShaderSetBuilder, SourceLanguage, SourceShaderInfo, SpirvReflection,
    SpirvShader,
};

pub mod winit;

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

layout(location = 0) in vec2 v_Uv;
layout(location = 1) in vec4 v_Color;
layout(location = 2) flat in uint v_Mode;

layout(location = 0) out vec4 Target0;

void main() {
    // Text
    if (v_Mode == uint(0)) {
        Target0 = v_Color;

    // Image
    } else if (v_Mode == uint(1)) {
        Target0 = v_Color;

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

#[derive(Debug, Default)]
pub struct ConrodPipelineDesc;

#[derive(Debug)]
pub struct ConrodPipeline<B: Backend> {
    buffer: Escape<Buffer<B>>,
    vertice_count: u32,
}

impl<B> SimpleGraphicsPipelineDesc<B, Ui> for ConrodPipelineDesc
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

    fn load_shader_set(&self, factory: &mut Factory<B>, _aux: &Ui) -> ShaderSet<B> {
        SHADERS.build(factory, Default::default()).unwrap()
    }

    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        ui: &Ui,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<Self::Pipeline, CreationError> {
        assert!(buffers.is_empty());
        assert!(images.is_empty());
        assert!(set_layouts.is_empty());

        // Functions for converting for conrod scalar coords to GL vertex coords (-1.0 to 1.0).
        const WIN_W: f32 = 600.0;
        const WIN_H: f32 = 420.0;

        let vx = |x: Scalar| (x as f32 / (WIN_W / 2.0));
        let vy = |y: Scalar| -1.0 * (y as f32 / (WIN_H / 2.0));

        let mut vertices = vec![];

        if let Some(mut primitives) = ui.draw_if_changed() {
            while let Some(primitive) = primitives.next_primitive() {
                match primitive.kind {
                    PrimitiveKind::Rectangle { color }
                    | PrimitiveKind::Text { color, .. }
                    | PrimitiveKind::Image {
                        color: Some(color), ..
                    } => {
                        let mut color = color.to_fsa();
                        match primitive.kind {
                            PrimitiveKind::Text { .. } => {
                                color = [1.0, 0.0, 0.0, 1.0];
                            }
                            PrimitiveKind::Image { .. } => {
                                color = [0.0, 1.0, 0.0, 1.0];
                            }
                            _ => {}
                        }
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
                    _ => {}
                }
            }
        }

        let buffer_size =
            SHADER_REFLECTION.attributes_range(..).unwrap().stride as u64 * vertices.len() as u64;

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

        Ok(ConrodPipeline {
            buffer,
            vertice_count: vertices.len() as u32,
        })
    }
}

impl<B> SimpleGraphicsPipeline<B, Ui> for ConrodPipeline<B>
where
    B: Backend,
{
    type Desc = ConrodPipelineDesc;

    fn prepare(
        &mut self,
        _factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[Handle<DescriptorSetLayout<B>>],
        _index: usize,
        _aux: &Ui,
    ) -> PrepareResult {
        PrepareResult::DrawReuse
    }

    fn draw(
        &mut self,
        _layout: &<B as Backend>::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        _index: usize,
        _aux: &Ui,
    ) {
        unsafe {
            encoder.bind_vertex_buffers(0, Some((self.buffer.raw(), 0)));
            encoder.draw(0..self.vertice_count, 0..1);
        }
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &Ui) {}
}
