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
/*
impl_vertex! {
    Vertex {
        pos => [Position; Float; 2; false],
        uv =>[Texcoord0; Float; 2; false],
        //color =>[Color0; Float; 4; true],
        color =>[Color0; Float; 4; false],
        mode => [Tangent; Float; 2;false],
    }
}
*/
impl_vertex! {
    Vertex {
        position => [Position; Float; 2; false],
    }
}
/// Draw text from the text cache texture `tex` in the fragment shader.
pub const MODE_TEXT: f32 = 0.0;
/// Draw an image from the texture at `tex` in the fragment shader.
pub const MODE_IMAGE: f32 = 1.0;
/// Ignore `tex` and draw simple, colored 2D geometry.
pub const MODE_GEOMETRY: f32 = 2.0;
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
    shader2: ShaderHandle,
    surface: SurfaceHandle,
    //rendered_texture:RenderTextureHandle,
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
    Test,
}

pub struct Commands<'a> {
    commands: std::slice::Iter<'a, PreparedCommand>,
    vertices: &'a [Vertex],
}
impl Renderer{
    pub fn new(dim:(f64,f64),dpi_factor: f64)->Self{
        /*let mut params = RenderTextureParams::default();
        params.format = RenderTextureFormat::RGBA8;
        params.dimensions = (dim.0 as u32,dim.1 as u32).into();
        let rendered_texture = video::create_render_texture(params).unwrap();
        */
        let attributes = AttributeLayoutBuilder::new()
            .with(Attribute::Position, 2)
            .finish();
        let uniforms = UniformVariableLayout::build()
            .with("tex", UniformVariableType::RenderTexture)
            .finish();
        let mut params = ShaderParams::default();
        params.attributes = attributes;
        //params.uniforms = uniforms;
        //looking for Position
        let vs = include_str!("shaders/conrod.1.vs").to_owned();
        let fs = include_str!("shaders/conrod.1.fs").to_owned();
        let shader = video::create_shader(params.clone(), vs, fs).unwrap();

        let mut params = ShaderParams::default();
        let attributes2 = AttributeLayoutBuilder::new()
            .with(Attribute::Position, 2)
            .finish();
        params.attributes = attributes2;
        let vs2 = include_str!("shaders/triangle.vs").to_owned();
        let fs2 = include_str!("shaders/triangle.fs").to_owned();
        let shader2 = video::create_shader(params, vs2, fs2).unwrap();
        let mut params = SurfaceParams::default();
        //params.set_attachments(&[rendered_texture], None).unwrap();
        params.set_clear(Color::gray(), None, None);
        let vert:Vec<Vertex> = Vec::new();
        let commands:Vec<PreparedCommand> = Vec::new();
        let surface = video::create_surface(params).unwrap();
        
        Renderer{
          vertices: vert,
          shader:shader,
          shader2:shader2,
          surface:surface,
          //rendered_texture:rendered_texture,
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
        let Renderer { ref mut vertices,shader,surface,ref mut commands,..} = *self;
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
        let mut c = 0;
        while let Some(primitive) = primitives.next_primitive() {
            let render::Primitive { kind, scizzor, rect, .. } = primitive;
            /*
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
            */
            match kind {
                render::PrimitiveKind::Rectangle { color } => {
    
                    switch_to_plain_state!();
                    let color = gamma_srgb_to_linear(color.to_fsa());
                    let (l, r, b, t) = rect.l_r_b_t();
                    let v = |x, y| {
                        // Convert from conrod Scalar range to GL range -1.0 to 1.0.
                        /*Vertex {
                            pos: [vx(x), vy(y)],
                            uv: [0.0, 0.0],
                            color: color,
                            mode: [MODE_GEOMETRY,0.0],
                        }*/
                        Vertex::new([vx(x),vy(y)])
                    };
                    if c ==0{
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
                    
                },
                render::PrimitiveKind::Other(_) => (),
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
        let mut c=0;
        for command in self.commands() {
            match command {
                Command::Scizzor(scizzor) => {
                    batch.update_scissor(scizzor)
                },

                // Draw to the target with the given `draw` command.
                Command::Draw(draw) => match draw {
                    Draw_e::Plain(slice) => if slice.len() >= NUM_VERTICES_IN_TRIANGLE {
                        //let mut idxes = vec![];
                        let mut idxes:Vec<u16> = vec![];
                        for i in 0..slice.len(){
                            idxes.push(i as u16);
                        }
                       // let idxes = [0,1,2];
                        println!("slice {:?}",slice.len());
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
                        
                        let mesh = video::create_mesh(params.clone(), Some(data)).unwrap();
                        let dc = Draw::new(self.shader, mesh);
                        batch.draw(dc);
                        batch.submit(self.surface).unwrap();
                       
                    },
                    Draw_e::Test =>{
                        let mut params = MeshParams::default();
                        let slice : [Vertex; 6] = [
                            Vertex::new([0.5, 0.0]),
                            Vertex::new([0.5, 0.5]),
                            Vertex::new([0.0, 0.0]),
                            Vertex::new([0.0, 0.0]),
                            Vertex::new([0.0, 0.5]),
                            Vertex::new([0.5, 0.5])
                        ];
                        let mut idxes:Vec<u16> = vec![];
                        for i in 0..slice.len(){
                            idxes.push(i as u16);
                        }
                        params.num_verts = slice.len();
                        params.num_idxes = slice.len();
                        params.primitive = MeshPrimitive::Triangles;
                        params.layout = Vertex::layout();
                        let data = MeshData {
                            //vptr: Vertex::encode(&slice[..]).into(),
                            vptr: Vertex::encode(&slice[..]).into(),
                            iptr: IndexFormat::encode(&idxes).into(),
                        };
                        
                        let mesh = video::create_mesh(params.clone(), Some(data)).unwrap();
                        let dc = Draw::new(self.shader2, mesh);
                        batch.draw(dc);
                        batch.submit(self.surface).unwrap();
                        println!("submitted");
                        
                    }
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
                // Command::Draw(Draw_e::Test),
            PreparedCommand::Image(id, ref range) =>
                Command::Draw(Draw_e::Image(id, &vertices[range.clone()])),
        })
    }
}