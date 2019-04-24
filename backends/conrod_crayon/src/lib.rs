extern crate crayon;
extern crate conrod_core;
extern crate serde_json;
extern crate cgmath;
use crayon::impl_vertex;
use crayon::prelude::*;
use crayon::errors::Result;
use crayon::video::assets::texture::*;

pub mod events;
use conrod_core::{
    Rect,
    Scalar,
    color,
    image,
    render,
    text,
};

impl_vertex! {
    Vertex {
        pos => [Position; Float; 2; false],
        uv =>[Texcoord0; Float; 2; false],
        color =>[Color0; Float; 4; true],
        mode =>[Weight; Short; 1; false],
    }
}

/// Draw text from the text cache texture `tex` in the fragment shader.
pub const MODE_TEXT: i32 = 0;
/// Draw an image from the texture at `tex` in the fragment shader.
pub const MODE_IMAGE: i32 = 1;
/// Ignore `tex` and draw simple, colored 2D geometry.
pub const MODE_GEOMETRY: i32 = 2;
/// A `Command` describing a step in the drawing process.
#[derive(Clone, Debug)]
pub enum Command<'a> {
    /// Draw to the target.
    Draw(DrawE<'a>),
    /// Update the scizzor within the `glium::DrawParameters`.
    Scizzor(SurfaceScissor),
}
enum PreparedCommand {
    Image(image::Id, std::ops::Range<usize>),
    Plain(std::ops::Range<usize>),
    Scizzor(SurfaceScissor),
}
/// A rusttype `GlyphCache` along with a `glium::texture::Texture2d` for caching text on the `GPU`.
pub struct GlyphCache {
    cache: text::GlyphCache<'static>,
    texture: TextureHandle,
}

impl GlyphCache {
    /// Construct a **GlyphCache** with the given texture dimensions.
    ///
    /// When calling `GlyphCache::new`, the `get_framebuffer_dimensions` method is used to produce
    /// the width and height. However, often creating a texture the size of the screen might not be
    /// large enough to cache the necessary text for an application. The following constant
    /// multiplier is used to ensure plenty of room in the cache.
    pub fn with_dimensions(
        width: u32,
        height: u32,
    ) -> Result<Self>
    {
        // First, the rusttype `Cache` which performs the logic for rendering and laying out glyphs
        // in the cache.
        let cache = rusttype_glyph_cache(width, height);

        // Now the texture to which glyphs will be rendered.
        let texture = glyph_cache_texture(width, height)?;

        Ok(GlyphCache {
            cache: cache,
            texture: texture,
        })
    }

    /// Construct a `GlyphCache` with a size equal to the given `Display`'s current framebuffer
    /// dimensions.
    pub fn new(dim:(f64,f64)) -> Result<Self>
    {
        println!("inii {:?}",dim);
        Self::with_dimensions(dim.0 as u32, dim.1 as u32)
    }

    /// The texture used to cache the glyphs on the GPU.
    pub fn texture(&self) -> &TextureHandle {
        &self.texture
    }
}

pub struct Renderer {
    glyph_cache: GlyphCache,
    vertices: Vec<Vertex>,
    shader: ShaderHandle,
    surface: SurfaceHandle,
    //rendered_texture:RenderTextureHandle,
    commands: Vec<PreparedCommand>,
}
/// A `Command` for drawing to the target.
///
/// Each variant describes how to draw the contents of the vertex buffer.
#[derive(Clone, Debug)]
pub enum DrawE<'a> {
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

        let attributes = AttributeLayoutBuilder::new()
            .with(Attribute::Position, 2)
            .with(Attribute::Texcoord0, 2)
            .with(Attribute::Color0, 4)
            .with(Attribute::Weight,1)
            .finish();
        let uniforms = UniformVariableLayout::build()
            .with("tex", UniformVariableType::Texture)
            .finish();
        let mut params = ShaderParams::default();
         params.state.color_blend = Some((crayon::video::assets::shader::Equation::Add,
        crayon::video::assets::shader::BlendFactor::Value(crayon::video::assets::shader::BlendValue::SourceAlpha),
        crayon::video::assets::shader::BlendFactor::OneMinusValue(crayon::video::assets::shader::BlendValue::SourceAlpha)));
        
        params.attributes = attributes;
        params.uniforms = uniforms;
        //looking for Position
        let vs = include_str!("shaders/conrod.vs").to_owned();
        let fs = include_str!("shaders/conrod.fs").to_owned();
        let shader = video::create_shader(params.clone(), vs, fs).unwrap();

        let mut params = SurfaceParams::default();
        params.set_clear(Color::gray(), None, None);
        let vert:Vec<Vertex> = Vec::new();
        let commands:Vec<PreparedCommand> = Vec::new();
        let surface = video::create_surface(params).unwrap();
        
        Renderer{
          glyph_cache:GlyphCache::new((dim.0 * dpi_factor,dim.1 * dpi_factor)).unwrap(),
          vertices: vert,
          shader:shader,
          surface:surface,
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
        let Renderer { ref mut vertices,shader,surface,ref mut commands, ref mut glyph_cache,..} = *self;
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
                    let color = gamma_srgb_to_linear(color.to_fsa());

                    let (l, r, b, t) = rect.l_r_b_t();
                    let v = |x, y| {
                        Vertex::new([vx(x),vy(y)],[0.0,0.0],color,MODE_GEOMETRY)
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

                render::PrimitiveKind::TrianglesSingleColor { color, triangles } => {
                    if triangles.is_empty() {
                        continue;
                    }

                    switch_to_plain_state!();

                    let color = gamma_srgb_to_linear(color.into());

                    let v = |p: [Scalar; 2]| {
                        Vertex::new([vx(p[0]), vy(p[1])],[0.0,0.0],color,MODE_GEOMETRY)
                    };

                    for triangle in triangles {
                        vertices.push(v(triangle[0]));
                        vertices.push(v(triangle[1]));
                        vertices.push(v(triangle[2]));
                    }
                },

                render::PrimitiveKind::TrianglesMultiColor { triangles } => {
                    if triangles.is_empty() {
                        continue;
                    }

                    switch_to_plain_state!();

                    let v = |(p, c): ([Scalar; 2], color::Rgba)| {
                        Vertex::new([vx(p[0]), vy(p[1])],[0.0,0.0],gamma_srgb_to_linear(c.into()),MODE_GEOMETRY)
                    };

                    for triangle in triangles {
                        vertices.push(v(triangle[0]));
                        vertices.push(v(triangle[1]));
                        vertices.push(v(triangle[2]));
                    }
                },
                render::PrimitiveKind::Text { color, text, font_id } => {
                    switch_to_plain_state!();

                    let positioned_glyphs = text.positioned_glyphs(dpi_factor as f32);

                    let GlyphCache { ref mut cache, ref mut texture } = *glyph_cache;

                    // Queue the glyphs to be cached.
                    for glyph in positioned_glyphs.iter() {
                        cache.queue_glyph(font_id.index(), glyph.clone());
                    }

                    // Cache the glyphs on the GPU.
                    cache.cache_queued(|rect, data| {
                        let width = rect.width();
                        let height = rect.height();
                        let lbwh = [rect.min.x,rect.min.y,rect.max.x,rect.max.y];
                        let p1 = cgmath::Point2::new(lbwh[0],lbwh[1]);
                        let p2 = cgmath::Point2::new(lbwh[2],lbwh[3]);
                        let rect = crayon::math::aabb::Aabb2::new(p1,p2);
                        video::update_texture(*texture,rect,data).unwrap();
        
                    }).unwrap();

                    let color = gamma_srgb_to_linear(color.to_fsa());
                    let cache_id = font_id.index();

                    let origin = text::rt::point(0.0, 0.0);
                    let to_gl_rect = |screen_rect: text::rt::Rect<i32>| text::rt::Rect {
                        min: origin
                            + (text::rt::vector(screen_rect.min.x as f32 / screen_w as f32 - 0.5,
                                          1.0 - screen_rect.min.y as f32 / screen_h as f32 - 0.5)) * 2.0,
                        max: origin
                            + (text::rt::vector(screen_rect.max.x as f32 / screen_w as f32 - 0.5,
                                          1.0 - screen_rect.max.y as f32 / screen_h as f32 - 0.5)) * 2.0
                    };
                    let mut l =0;
                    for g in positioned_glyphs {
                        
                        if let Ok(Some((uv_rect, screen_rect))) = cache.rect_for(cache_id, g) {
                            
                            let gl_rect = to_gl_rect(screen_rect);
                            let v = |p:[f32;2],t:[f32;2]| {Vertex::new(p,t,color,MODE_TEXT)};
                            let mut push_v = |p, t| vertices.push(v(p, t));
                            push_v([gl_rect.min.x, gl_rect.max.y], [uv_rect.min.x, uv_rect.max.y]);
                            push_v([gl_rect.min.x, gl_rect.min.y], [uv_rect.min.x, uv_rect.min.y]);
                            push_v([gl_rect.max.x, gl_rect.min.y], [uv_rect.max.x, uv_rect.min.y]);
                            push_v([gl_rect.max.x, gl_rect.min.y], [uv_rect.max.x, uv_rect.min.y]);
                            push_v([gl_rect.max.x, gl_rect.max.y], [uv_rect.max.x, uv_rect.max.y]);
                            push_v([gl_rect.min.x, gl_rect.max.y], [uv_rect.min.x, uv_rect.max.y]);
                        l = l+1;
                        
                        }
                    }
                    
                },
                render::PrimitiveKind::Image { image_id, color, source_rect } => {
                    // Switch to the `Image` state for this image if we're not in it already.
                    let new_image_id = image_id;
                    match current_state {

                        // If we're already in the drawing mode for this image, we're done.
                        State::Image { image_id, .. } if image_id == new_image_id => (),

                        // If we were in the `Plain` drawing state, switch to Image drawing state.
                        State::Plain { start } => {
                            commands.push(PreparedCommand::Plain(start..vertices.len()));
                            current_state = State::Image {
                                image_id: new_image_id,
                                start: vertices.len(),
                            };
                        },

                        // If we were drawing a different image, switch state to draw *this* image.
                        State::Image { image_id, start } => {
                            commands.push(PreparedCommand::Image(image_id, start..vertices.len()));
                            current_state = State::Image {
                                image_id: new_image_id,
                                start: vertices.len(),
                            };
                        },
                    }
                    
                    let color = color.unwrap_or(color::WHITE).to_fsa();

                    if let Some(&image) = image_map.get(&image_id) {
                        let mut image_w:f64 = 100.0;
                        let mut image_h:f64 = 100.0;
                        if let Some(image_param) = video::texture(image){
                            image_w = image_param.dimensions.x as f64;
                            image_h = image_param.dimensions.y as f64;
                        } 

                        // Get the sides of the source rectangle as uv coordinates.
                        //
                        // Texture coordinates range:
                        // - left to right: 0.0 to 1.0
                        // - bottom to top: 0.0 to 1.0
                        let (uv_l, uv_r, uv_b, uv_t) = match source_rect {
                            Some(src_rect) => {
                                let (l, r, b, t) = src_rect.l_r_b_t();
                                ((l / image_w) as f32,
                                 (r / image_w) as f32,
                                 (b / image_h) as f32,
                                 (t / image_h) as f32)
                            },
 
                            None => (0.0, 1.0 , 0.0, 1.0),
                        };

                        let v = |x, y, t| {
                            // Convert from conrod Scalar range to GL range -1.0 to 1.0.
                            let x = (x * dpi_factor as Scalar / half_win_w) as f32;
                            let y = (y * dpi_factor as Scalar / half_win_h) as f32;
                            Vertex::new([x, y],t,color,MODE_IMAGE)
                        };

                        let mut push_v = |x, y, t| vertices.push(v(x, y, t));

                        let (l, r, b, t) = rect.l_r_b_t();

                        // Bottom left triangle.
                        push_v(l, t, [uv_l, uv_t]);
                        push_v(r, b, [uv_r, uv_b]);
                        push_v(l, b, [uv_l, uv_b]);

                        // Top right triangle.
                        push_v(l, t, [uv_l, uv_t]);
                        push_v(r, b, [uv_r, uv_b]);
                        push_v(r, t, [uv_r, uv_t]);
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
    pub fn draw(&self,batch: &mut CommandBuffer,image_map:&conrod_core::image::Map<TextureHandle>){
        const NUM_VERTICES_IN_TRIANGLE: usize = 3;
        let uniform =self.glyph_cache.texture(); //can use rendertexture

        for command in self.commands() {
            match command {
                Command::Scizzor(scizzor) => {
                    batch.update_scissor(scizzor)
                },

                // Draw to the target with the given `draw` command.
                Command::Draw(draw) => match draw {
                    DrawE::Plain(slice) => if slice.len() >= NUM_VERTICES_IN_TRIANGLE {
                        let mut idxes:Vec<u16> = vec![];
                        for i in 0..slice.len(){
                            idxes.push(i as u16);
                        }

                        let mut params = MeshParams::default();
                        params.num_verts = slice.len();
                        params.num_idxes = slice.len();
                        params.primitive = MeshPrimitive::Triangles;
                        params.layout = Vertex::layout();
                        let data = MeshData {
                            vptr: Vertex::encode(&slice[..]).into(),
                            iptr: IndexFormat::encode(&idxes).into(),
                        };
                        let mesh = video::create_mesh(params.clone(), Some(data)).unwrap();
                        let mut dc = Draw::new(self.shader, mesh);
                        dc.set_uniform_variable("tex", *uniform);
                        batch.draw(dc);
                        batch.submit(self.surface).unwrap();
                       
                    },
                    // Draw an image whose texture data lies within the `image_map` at the
                    // given `id`.
                    //
                    // Only submit the vertices if there is enough for at least one triangle.
                    DrawE::Image(image_id, slice) => if slice.len() >= NUM_VERTICES_IN_TRIANGLE {
                        if let Some(&image) = image_map.get(&image_id) {
                            
                            let mut idxes:Vec<u16> = vec![];
                            for i in 0..slice.len(){
                                idxes.push(i as u16);
                            }

                            let mut params = MeshParams::default();
                            params.num_verts = slice.len();
                            params.num_idxes = slice.len();
                            params.primitive = MeshPrimitive::Triangles;
                            params.layout = Vertex::layout();
                            let data = MeshData {
                                vptr: Vertex::encode(&slice[..]).into(),
                                iptr: IndexFormat::encode(&idxes).into(),
                            };
                            
                            let mesh = video::create_mesh(params, Some(data)).unwrap();
                            let mut dc = Draw::new(self.shader, mesh);
                            dc.set_uniform_variable("tex", image);
                            batch.draw(dc);
                            batch.submit(self.surface).unwrap()
                        }
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
                Command::Draw(DrawE::Plain(&vertices[range.clone()])),
            PreparedCommand::Image(id, ref range) =>
                Command::Draw(DrawE::Image(id, &vertices[range.clone()])),
        })
    }
}
// Creating the rusttype glyph cache used within a `GlyphCache`.
fn rusttype_glyph_cache(w: u32, h: u32) -> text::GlyphCache<'static> {
    const SCALE_TOLERANCE: f32 = 0.1;
    const POSITION_TOLERANCE: f32 = 0.1;
    text::GlyphCache::builder()
        .dimensions(w, h)
        .scale_tolerance(SCALE_TOLERANCE)
        .position_tolerance(POSITION_TOLERANCE)
        .build()
}

// Create the texture used within a `GlyphCache` of the given size.
fn glyph_cache_texture(
    width: u32,
    height: u32,
) -> Result<TextureHandle>
{
    // Determine the optimal texture format to use given the opengl version.
    let mut params = TextureParams::default();
    params.format = TextureFormat::R8;
    //params.hint = TextureHint::Stream;
    params.hint = TextureHint::Stream;
    params.dimensions = (width, height).into();
    let data_size = params.format.size(params.dimensions) as usize;
    let bytes = vec![];
    let data = TextureData{
       bytes:bytes
    };
    video::create_texture(params,data)
}