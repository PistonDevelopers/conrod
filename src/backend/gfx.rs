use gfx::{self,Resources,Factory, texture,PipelineState};
use gfx::handle::{RenderTargetView};
use gfx::traits::FactoryExt;
use render;
use text::{rt,GlyphCache};
use std;

const FRAGMENT_SHADER: &'static [u8] = b"
    #version 140

    uniform sampler2D t_Color;

    in vec2 v_Uv;
    in vec4 v_Color;

    out vec4 f_Color;

    void main() {
        vec4 tex = texture(t_Color, v_Uv);
        f_Color = v_Color * tex;
    }
";

const VERTEX_SHADER: &'static [u8] = b"
    #version 140

    in vec2 a_Pos;
    in vec2 a_Uv;
    in vec4 a_Color;

    out vec2 v_Uv;
    out vec4 v_Color;

    void main() {
        v_Uv = a_Uv;
        v_Color = a_Color;
        gl_Position = vec4(a_Pos, 0.0, 1.0);
    }
";

/// Possible errors that may occur during a call to `Renderer::new`.
#[derive(Debug)]
pub enum RendererCreationError {
    /// Errors that might occur when creating the pipeline.
    PipelineState(gfx::PipelineStateError<String>),
}

// Format definitions (must be pub for  gfx_defines to use them)
pub type ColorFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;
type SurfaceFormat = gfx::format::R8_G8_B8_A8;
type FullFormat = (SurfaceFormat, gfx::format::Unorm);

//this is it's own module to allow_unsafe within it
mod defines{
    //it appears gfx_defines generates unsafe code
    #![allow(unsafe_code)]
    use gfx;
    use super::ColorFormat;
    // Vertex and pipeline declarations
    gfx_defines! {
        vertex Vertex {
            pos: [f32; 2] = "a_Pos",
            uv: [f32; 2] = "a_Uv",
            color: [f32; 4] = "a_Color",
        }

        pipeline pipe {
            vbuf: gfx::VertexBuffer<Vertex> = (),
            color: gfx::TextureSampler<[f32; 4]> = "t_Color",
            out: gfx::BlendTarget<ColorFormat> = ("f_Color", ::gfx::state::MASK_ALL, ::gfx::preset::blend::ALPHA),
        }
    }
}

use self::defines::*;

// Convenience constructor
impl Vertex {
    fn new(pos: [f32; 2], uv: [f32; 2], color: [f32; 4]) -> Vertex {
        Vertex {
            pos: pos,
            uv: uv,
            color: color,
        }
    }
}

pub struct Renderer<R: Resources>{
    pipeline: PipelineState<R, pipe::Meta>,
    glyph_cache: GlyphCache,
    cache_tex: gfx::handle::Texture<R, SurfaceFormat>,
    cache_tex_view: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    blank_texture: gfx::handle::ShaderResourceView<R, [f32; 4]>,
    data: pipe::Data<R>,
    dpi_factor: f32,
}

impl<R: Resources> Renderer<R>{
    pub fn new<F: Factory<R>>(factory: &mut F, rtv: &RenderTargetView<R, ColorFormat>, dpi_factor: f32) -> Result<Self,RendererCreationError>
    {
        let sampler_info = texture::SamplerInfo::new(
            texture::FilterMethod::Bilinear,
            texture::WrapMode::Clamp
        );
        let sampler = factory.create_sampler(sampler_info);

        let vbuf = factory.create_vertex_buffer(&[]);
        let (_, fake_texture) = create_texture(factory, 2, 2, &[0;4]);
        let (_, blank_texture) = create_texture(factory, 2, 2, &[255;4]);

        let mut data = pipe::Data {
            vbuf,
            color: (fake_texture.clone(), sampler),
            out: rtv.clone(),
        };

        let pipeline = factory.create_pipeline_simple(VERTEX_SHADER, FRAGMENT_SHADER, pipe::new())?;

        let (glyph_cache, cache_tex, cache_tex_view) = {
            let (width,height,_depth,_samples) = rtv.get_dimensions();

            let width = (width as f32 * dpi_factor) as u32;
            let height = (height as f32 * dpi_factor) as u32;

            const SCALE_TOLERANCE: f32 = 0.1;
            const POSITION_TOLERANCE: f32 = 0.1;

            let cache = GlyphCache::new(width, height,
                                                      SCALE_TOLERANCE,
                                                      POSITION_TOLERANCE);

            let data = vec![0; (width * height * 4) as usize];

            let (texture, texture_view) = create_texture(factory, width, height, &data);

            (cache, texture, texture_view)
        };
        Ok(Renderer{
            pipeline,
            glyph_cache,
            cache_tex,
            cache_tex_view,
            blank_texture,
            data,
            dpi_factor,
        })
    }

    pub fn draw<'a,F: Factory<R>,C: gfx::CommandBuffer<R>,D: gfx::Device<Resources=R,CommandBuffer=C>>(&mut self, factory: &mut F, encoder: &mut gfx::Encoder<R,C>, device: &mut D, mut primitives: render::Primitives<'a>, dims: (f32,f32)/*, image_map: &image::Map<gfx::handle::ShaderResourceView<R, [f32; 4]>>*/)
    {
        let Renderer{ ref pipeline, ref mut glyph_cache, ref cache_tex, ref cache_tex_view, ref blank_texture, ref mut data, dpi_factor} = *self;
        let (screen_width, screen_height) = dims;
        let mut vertices = Vec::new();
        let mut text_vertices = Vec::new();

        // Create vertices
        while let Some(render::Primitive { id, kind, scizzor, rect }) = primitives.next() {
            match kind {
                render::PrimitiveKind::Rectangle { color } => {
                    let color = gamma_srgb_to_linear(color.to_fsa());

                    let mut v = |x,y| vertices.push(Vertex::new([2.0 * x as f32 / screen_width, 2.0 * y as f32 / screen_height], [0.0, 0.0], color));
                    
                    v(rect.x.start, rect.y.end  );
                    v(rect.x.start, rect.y.start);
                    v(rect.x.end  , rect.y.start);
                    v(rect.x.end  , rect.y.start);
                    v(rect.x.end  , rect.y.end  );
                    v(rect.x.start, rect.y.end  );
                },
                render::PrimitiveKind::TrianglesSingleColor { color, triangles } => {
                    let color = gamma_srgb_to_linear(color.into());
                    use position::Point;
                    let mut v = |pos: Point| vertices.push(Vertex::new([2.0 * pos[0] as f32 / screen_width, 2.0 * pos[1] as f32 / screen_height], [0.0, 0.0], color));

                    for triangle in triangles{
                        v(triangle[0]);
                        v(triangle[1]);
                        v(triangle[2]);
                    }
                },
                render::PrimitiveKind::TrianglesMultiColor { triangles } => {
                    use widget::primitive::shape::triangles::ColoredPoint;
                    let mut v = |(pos,rgba): ColoredPoint| vertices.push(Vertex::new([2.0 * pos[0] as f32 / screen_width, 2.0 * pos[1] as f32 / screen_height], [0.0, 0.0], gamma_srgb_to_linear(rgba.into())));

                    for triangle in triangles{
                        v(triangle[0]);
                        v(triangle[1]);
                        v(triangle[2]);
                    }
                },
                render::PrimitiveKind::Image { .. } => {
                    //TODO
                },
                render::PrimitiveKind::Text { color, text, font_id } => {
                    let positioned_glyphs = text.positioned_glyphs(dpi_factor);

                    // Queue the glyphs to be cached
                    for glyph in positioned_glyphs {
                        glyph_cache.queue_glyph(font_id.index(), glyph.clone());
                    }

                    glyph_cache.cache_queued(|rect, data| {
                        let offset = [rect.min.x as u16, rect.min.y as u16];
                        let size = [rect.width() as u16, rect.height() as u16];

                        let new_data = data.iter().map(|x| [255, 255, 255, *x]).collect::<Vec<_>>();

                        update_texture(encoder, &cache_tex, offset, size, &new_data);
                    }).unwrap();

                    let color = gamma_srgb_to_linear(color.to_fsa());
                    let cache_id = font_id.index();
                    let origin = rt::point(0.0, 0.0);

                    // A closure to convert RustType rects to GL rects
                    let to_gl_rect = |screen_rect: rt::Rect<i32>| rt::Rect {
                        min: origin
                            + (rt::vector(screen_rect.min.x as f32 / screen_width - 0.5,
                                            1.0 - screen_rect.min.y as f32 / screen_height - 0.5)) * 2.0,
                        max: origin
                            + (rt::vector(screen_rect.max.x as f32 / screen_width - 0.5,
                                            1.0 - screen_rect.max.y as f32 / screen_height - 0.5)) * 2.0,
                    };

                    // Create new vertices
                    let extension = positioned_glyphs.into_iter()
                        .filter_map(|g| glyph_cache.rect_for(cache_id, g).ok().unwrap_or(None))
                        .flat_map(|(uv_rect, screen_rect)| {
                            use std::iter::once;

                            let gl_rect = to_gl_rect(screen_rect);
                            let v = |pos, uv| once(Vertex::new(pos, uv, color));

                            v([gl_rect.min.x, gl_rect.max.y], [uv_rect.min.x, uv_rect.max.y])
                                .chain(v([gl_rect.min.x, gl_rect.min.y], [uv_rect.min.x, uv_rect.min.y]))
                                .chain(v([gl_rect.max.x, gl_rect.min.y], [uv_rect.max.x, uv_rect.min.y]))
                                .chain(v([gl_rect.max.x, gl_rect.min.y], [uv_rect.max.x, uv_rect.min.y]))
                                .chain(v([gl_rect.max.x, gl_rect.max.y], [uv_rect.max.x, uv_rect.max.y]))
                                .chain(v([gl_rect.min.x, gl_rect.max.y], [uv_rect.min.x, uv_rect.max.y]))
                        });

                    text_vertices.extend(extension);
                },
                render::PrimitiveKind::Other(_) => {},
            }
        }

        // Draw the vertices
        data.color.0 = blank_texture.clone();
        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertices, ());
        data.vbuf = vbuf;
        encoder.draw(&slice, &pipeline, data);

        // Draw the text vertices
        data.color.0 = cache_tex_view.clone();
        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&text_vertices, ());
        data.vbuf = vbuf;
        encoder.draw(&slice, &pipeline, data);
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

// Creates a gfx texture with the given data
fn create_texture<F, R>(factory: &mut F, width: u32, height: u32, data: &[u8])
    -> (gfx::handle::Texture<R, SurfaceFormat>, gfx::handle::ShaderResourceView<R, [f32; 4]>)

    where R: gfx::Resources, F: gfx::Factory<R>
{
    // Modified `Factory::create_texture_immutable_u8` for dynamic texture.
    fn create_texture<T, F, R>(
        factory: &mut F,
        kind: gfx::texture::Kind,
        data: &[&[u8]]
    ) -> Result<(
        gfx::handle::Texture<R, T::Surface>,
        gfx::handle::ShaderResourceView<R, T::View>
    ), gfx::CombinedError>
        where F: gfx::Factory<R>,
                R: gfx::Resources,
                T: gfx::format::TextureFormat
    {
        use gfx::{format, texture};
        use gfx::memory::{Usage, SHADER_RESOURCE};
        use gfx_core::memory::Typed;

        let surface = <T::Surface as format::SurfaceTyped>::get_surface_type();
        let num_slices = kind.get_num_slices().unwrap_or(1) as usize;
        let num_faces = if kind.is_cube() {6} else {1};
        let desc = texture::Info {
            kind: kind,
            levels: (data.len() / (num_slices * num_faces)) as texture::Level,
            format: surface,
            bind: SHADER_RESOURCE,
            usage: Usage::Dynamic,
        };
        let cty = <T::Channel as format::ChannelTyped>::get_channel_type();
        let raw = try!(factory.create_texture_raw(desc, Some(cty), Some(data)));
        let levels = (0, raw.get_info().levels - 1);
        let tex = Typed::new(raw);
        let view = try!(factory.view_texture_as_shader_resource::<T>(
            &tex, levels, format::Swizzle::new()
        ));
        Ok((tex, view))
    }

    let kind = texture::Kind::D2(
        width as texture::Size,
        height as texture::Size,
        texture::AaMode::Single
    );
    create_texture::<ColorFormat, F, R>(factory, kind, &[data]).unwrap()
}

// Updates a texture with the given data (used for updating the GlyphCache texture)
fn update_texture<R, C>(encoder: &mut gfx::Encoder<R, C>,
                        texture: &gfx::handle::Texture<R, SurfaceFormat>,
                        offset: [u16; 2],
                        size: [u16; 2],
                        data: &[[u8; 4]])

    where R: gfx::Resources, C: gfx::CommandBuffer<R>
{
    let info = texture::ImageInfoCommon {
            xoffset: offset[0],
            yoffset: offset[1],
            zoffset: 0,
            width: size[0],
            height: size[1],
            depth: 0,
            format: (),
            mipmap: 0,
    };

    encoder.update_texture::<SurfaceFormat, FullFormat>(texture, None, info, data).unwrap();
}

impl From<gfx::PipelineStateError<String>> for RendererCreationError {
    fn from(err: gfx::PipelineStateError<String>) -> Self {
        RendererCreationError::PipelineState(err)
    }
}

impl std::error::Error for RendererCreationError {
    fn description(&self) -> &str {
        match *self {
            RendererCreationError::PipelineState(ref e) => std::error::Error::description(e),
        }
    }
}

impl std::fmt::Display for RendererCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            RendererCreationError::PipelineState(ref e) => std::fmt::Display::fmt(e, f),
        }
    }
}
