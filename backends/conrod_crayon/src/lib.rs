extern crate crayon;
extern crate conrod_core;
extern crate serde_json;
use crayon::impl_vertex;
use crayon::prelude::*;
pub mod events;
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
/// A `Command` describing a step in the drawing process.
#[derive(Clone, Debug)]
pub enum Command<'a> {
    /// Draw to the target.
    Draw(Draw_e<'a>),
    /// Update the scizzor within the `glium::DrawParameters`.
    Scizzor(SurfaceScissor),
}
enum PreparedCommand {
    Image(image::Id, std::ops::Range<usize>),
    Plain(std::ops::Range<usize>),
    Scizzor(SurfaceScissor),
}
pub struct Renderer {
    vertices: Vec<Vertex>,
    shader: ShaderHandle,
    surface: SurfaceHandle,
    rendered_texture:RenderTextureHandle,
    commands: Vec<PreparedCommand>,
}
/// A `Command` for drawing to the target.
///
/// Each variant describes how to draw the contents of the vertex buffer.
#[derive(Clone, Debug)]
pub enum Draw_e<'a> {
    /// A range of vertices representing triangles textured with the image in the
    /// image_map at the given `widget::Id`.
    Image(image::Id, &'a [Vertex]),
    /// A range of vertices representing plain triangles.
    Plain(&'a [Vertex]),
}

pub struct Commands<'a> {
    commands: std::slice::Iter<'a, PreparedCommand>,
    vertices: &'a [Vertex],
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
        let vert:Vec<Vertex> = Vec::new();
        let commands:Vec<PreparedCommand> = Vec::new();
        let surface = video::create_surface(params).unwrap();
        Renderer{
          vertices: vert,
          shader:shader,
          surface:surface,
          rendered_texture:rendered_texture,
          commands: commands
        }
    }
    pub fn commands(&self) -> Commands {
        let Renderer { ref commands, ref vertices, .. } = *self;
        Commands {
            commands: commands.iter(),
            vertices: vertices,
        }
    }
    pub fn fill<P>(&mut self,dims: (f64, f64),dpi_factor: f64,mut primitives: P, image_map:&conrod_core::image::Map<TextureHandle> )where P: render::PrimitiveWalker{
        let (screen_w, screen_h) = dims;
        let half_win_w = screen_w / 2.0;
        let half_win_h = screen_h / 2.0;
        let Renderer { ref mut vertices,shader,surface,rendered_texture,ref mut commands} = *self;
        commands.clear();
        vertices.clear();
        let mut current_scissor =SurfaceScissor::Enable{
            position: Vector2{x:0,y:0},
            size: Vector2{x:screen_w as u32,y:screen_h as u32}
        };
        enum State {
            Image { image_id: image::Id, start: usize },
            Plain { start: usize },
        }
        let mut current_state = State::Plain { start: 0 };
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
        let mut c =0;
        while let Some(primitive) = primitives.next_primitive() {
            let render::Primitive { kind, scizzor, rect, .. } = primitive;
            let new_scissor = rect_to_crayon_rect(scizzor);
            if new_scissor != current_scissor {
                // Finish the current command.
                match current_state {
                    State::Plain { start } =>
                        commands.push(PreparedCommand::Plain(start..vertices.len())),
                    State::Image { image_id, start } =>
                        commands.push(PreparedCommand::Image(image_id, start..vertices.len())),
                }

                // Update the scizzor and produce a command.
                current_scissor = new_scissor;
                commands.push(PreparedCommand::Scizzor(current_scissor));

                // Set the state back to plain drawing.
                current_state = State::Plain { start: vertices.len() };
            }
            match kind {
                render::PrimitiveKind::Rectangle { color } => {
                    switch_to_plain_state!();
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
        // Enter the final command.
        match current_state {
            State::Plain { start } =>
                commands.push(PreparedCommand::Plain(start..vertices.len())),
            State::Image { image_id, start } =>
                commands.push(PreparedCommand::Image(image_id, start..vertices.len())),
        }
    }
    pub fn draw(&self,batch: &mut CommandBuffer){
        const NUM_VERTICES_IN_TRIANGLE: usize = 3;
        
        
        for command in self.commands() {
            match command {
                Command::Scizzor(scizzor) => batch.update_scissor(scizzor),

                // Draw to the target with the given `draw` command.
                Command::Draw(draw) => match draw {
                    Draw_e::Plain(slice) => if slice.len() >= NUM_VERTICES_IN_TRIANGLE {
                        //let mut idxes = vec![];
                        let mut idxes:Vec<u8> = vec![];
                        for i in 0..slice.len(){
                            idxes.push(i as u8);
                        }
                       // let idxes = [0,1,2];
                        println!("vertices {:?}",slice);
                        let mut params = MeshParams::default();
                        params.num_verts = slice.len();
                        params.num_idxes = slice.len();
                        params.primitive = MeshPrimitive::Triangles;
                        params.layout = Vertex::layout();
                        let data = MeshData {
                            //vptr: Vertex::encode(&slice[..]).into(),
                            vptr: Vertex::encode(&slice[..]).into(),
                            iptr: IndexFormat::encode(&idxes).into(),
                        };
                        dbg!("before_create_mesh");
                        let mesh = video::create_mesh(params.clone(), Some(data)).unwrap();
                        dbg!("after_create_mesh");
                        let dc = Draw::new(self.shader, mesh);
                        dbg!("dc");
                        batch.draw(dc);
                        dbg!("draw dc");
                        batch.submit(self.surface).unwrap();
                        dbg!("submit dc");
                    },
                    _=>{

                    }
                }
            }
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

impl<'a> Iterator for Commands<'a> {
    type Item = Command<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let Commands { ref mut commands, ref vertices } = *self;
        commands.next().map(|command| match *command {
            PreparedCommand::Scizzor(scizzor) => Command::Scizzor(scizzor),
            PreparedCommand::Plain(ref range) =>
                Command::Draw(Draw_e::Plain(&vertices[range.clone()])),
            PreparedCommand::Image(id, ref range) =>
                Command::Draw(Draw_e::Image(id, &vertices[range.clone()])),
        })
    }
}