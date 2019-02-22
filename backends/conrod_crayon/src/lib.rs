extern crate crayon;
extern crate conrod_core;
use crayon::impl_vertex;
use crayon::prelude::*;
use conrod_core::{
    Rect,
    Scalar,
    color,
    image,
    render,
    text::{rt, GlyphCache},
};
impl_vertex! {
    Vertex {
        pos => [Position; Float; 2; false],
        uv =>[Texcoord0; Float; 2; false],
        color =>[Color0; Float; 4; true],
        mode => [Tangent; UShort; 2;false],
    }
}
/// Draw text from the text cache texture `tex` in the fragment shader.
pub const MODE_TEXT: u16 = 0;
/// Draw an image from the texture at `tex` in the fragment shader.
pub const MODE_IMAGE: u16 = 1;
/// Ignore `tex` and draw simple, colored 2D geometry.
pub const MODE_GEOMETRY: u16 = 2;
pub struct Renderer {
    mesh: Option<MeshHandle>,
    shader: ShaderHandle,
    surface: SurfaceHandle,
    rendered_texture:RenderTextureHandle
}
impl Renderer{
    pub fn new(dim:(f64,f64),dpi_factor: f64)->Self{
        let mut params = RenderTextureParams::default();
        params.format = RenderTextureFormat::RGBA8;
        params.dimensions = (dim.0 as u32,dim.1 as u32).into();
        let rendered_texture = video::create_render_texture(params).unwrap();
        let attributes = AttributeLayoutBuilder::new()
            .with(Attribute::Position, 2)
            .finish();
        let mut params = ShaderParams::default();
        params.attributes = attributes;
        //looking for Position
        let vs = include_str!("shaders/conrod.vs").to_owned();
        let fs = include_str!("shaders/conrod.fs").to_owned();
        let shader = video::create_shader(params, vs, fs).unwrap();
        let mut params = SurfaceParams::default();
        params.set_attachments(&[rendered_texture], None).unwrap();
        params.set_clear(Color::gray(), None, None);
        
        let surface = video::create_surface(params).unwrap();
        Renderer{
          mesh:None,
          shader:shader,
          surface:surface,
          rendered_texture:rendered_texture,
        }
    }
    pub fn fill<P>(&mut self,dims: (f64, f64),dpi_factor: f64,mut primitives: P, image_map:&conrod_core::image::Map<TextureHandle> )where P: render::PrimitiveWalker{
        let (screen_w, screen_h) = dims;
        let half_win_w = screen_w / 2.0;
        let half_win_h = screen_h / 2.0;
        let Renderer { mut mesh,shader,surface,rendered_texture} = *self;
        let mut vertices:Vec<Vertex> = Vec::new();
        let current_scissor =SurfaceScissor::Enable{
            position: Vector2{x:0,y:0},
            size: Vector2{x:screen_w as u32,y:screen_h as u32}
        };
        let rect_to_crayon_rect = |rect: Rect| {
            let (w, h) = rect.w_h();
            let left = (rect.left() * dpi_factor + half_win_w) as i32;
            let bottom = (rect.bottom() * dpi_factor + half_win_h) as i32;
            let width = (w * dpi_factor) as u32;
            let height = (h * dpi_factor) as u32;
            SurfaceScissor::Enable{
                position: Vector2{x:std::cmp::max(left, 0),y:std::cmp::max(bottom, 0)},
                size: Vector2{x:std::cmp::min(width, screen_w as u32),y: std::cmp::min(height, screen_h as u32)}
            }
        };
        
        let vx = |x: f64| (x * dpi_factor / half_win_w) as f32;
        let vy = |y: f64| (y * dpi_factor / half_win_h) as f32;
        while let Some(primitive) = primitives.next_primitive() {
            let render::Primitive { kind, scizzor, rect, .. } = primitive;
            let new_scizzor = rect_to_crayon_rect(scizzor);
            if new_scizzor != current_scissor {

            }
            match kind {
                render::PrimitiveKind::Rectangle { color } => {
                    dbg!("there is rect");
                    let color = gamma_srgb_to_linear(color.to_fsa());
                    let (l, r, b, t) = rect.l_r_b_t();
                    let v = |x, y| {
                        // Convert from conrod Scalar range to GL range -1.0 to 1.0.
                        Vertex {
                            pos: [vx(x), vy(y)],
                            uv: [0.0, 0.0],
                            color: color,
                            mode: [MODE_GEOMETRY,0],
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
                _=>{}
            }
        }
        let mut params = MeshParams::default();
        params.num_verts = 3;
        params.num_idxes = 3;
        let idxes: [u16; 3] = [0, 1, 2];
        let data = MeshData {
            vptr: Vertex::encode(&vertices[..]).into(),
            iptr: IndexFormat::encode(&idxes).into(),
        };

        mesh = Some(video::create_mesh(params, Some(data)).unwrap());
        
    }
    pub fn draw(&self,batch: &mut CommandBuffer){
        if let Some(m) = self.mesh{
            let surface = self.surface;
            let dc = Draw::new(self.shader, m);
            batch.draw(dc);
            batch.submit(surface).unwrap();
        }
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