//! A demonstration using glutin to provide events and glium for drawing the Ui.

#[cfg(feature="glutin")] #[cfg(feature="glium")] #[macro_use] extern crate conrod;
#[cfg(feature="glutin")] #[cfg(feature="glium")] #[macro_use] extern crate glium;
#[cfg(feature="glutin")] #[cfg(feature="glium")] #[macro_use] extern crate graphics;

#[cfg(feature="glutin")]
mod support;

fn main() {
    feature::main();
}

#[cfg(feature="glutin")]
#[cfg(feature="glium")]
mod feature {
    extern crate find_folder;
    extern crate image;

    use std::path::Path;

    use conrod;
    use glium;
    use support;
    use std;


    use glium::{DisplayBuild, Surface};
    use glium::glutin;

    use std::borrow::Cow;

    // The width and height in "points".
    const WIN_W: u32 = support::WIN_W;
    const WIN_H: u32 = support::WIN_H;


    #[derive(Copy, Clone, PartialEq)]
    struct Vertex {
        position: [f32; 2],
        tex_coords: [f32; 2],
        colour: [f32; 4]
    }

    enum GliumQueuedEvent  {
        QueuedVertexes { vertices: Vec<Vertex> },
        LazySetGLMode { gl_primitive_mode: glium::index::PrimitiveType },

        // TODO: Figure out lifetimes so that we can put uniform refs in here.
        LazySetUniform {  gl_uniform_id : String }

    }

    fn mode_as_str(gl_primitive_mode : glium::index::PrimitiveType ) -> String  {
        match gl_primitive_mode {
            glium::index::PrimitiveType::TrianglesList => "trilist",
            glium::index::PrimitiveType::LineStrip => "linestrip",
            _ => "other",
        }.to_string()
    }

    fn xy_to_glcoord(x : f32, y : f32, w : f32, h : f32) -> [f32;2] {
        // TODO: Write unit test.
        let new_x = (x / w) * 2.0;
        let new_y = (y / h) * 2.0;
        [new_x, new_y]
    }

    fn printed_xy_to_glcoord(x : f32, y : f32, w : f32, h : f32) -> [f32;2] {
        // TODO: Figure out non-println debugging.
        let toreturn = xy_to_glcoord(x,y,w,h);
        println!("xy_to_glcoord(x={}, y={}, w={}, h={}) -> [{},{}]", x, y, w, h, toreturn[0], toreturn[1]);
        toreturn
    }


    pub fn main() {

        // Build the window.
        let display = glutin::WindowBuilder::new()
            .with_vsync()
            .with_dimensions(WIN_W, WIN_H)
            .with_title("Conrod with glutin & glium!")
            .build_glium()
            .unwrap();


        // Create the `GL` program.
        let program = program!(
            &display,
            140 => {
                vertex: "
                    #version 140

                    in vec2 position;
                    in vec2 tex_coords;
                    in vec4 colour;

                    out vec2 v_tex_coords;
                    out vec4 v_colour;

                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                        v_tex_coords = tex_coords;
                        v_colour = colour;
                    }
                ",

                fragment: "
                    #version 140
                    uniform sampler2D tex;
                    in vec2 v_tex_coords;
                    in vec4 v_colour;
                    out vec4 f_colour;

                    void main() {
                        vec3 tex_part = texture(tex, v_tex_coords).rgb;
                        vec3 vrt_part = v_colour.rgb;

                        vec3 mixed_colour = mix(tex_part, vrt_part, 1.0);
                        f_colour = vec4(mixed_colour, 1.0);
                    }
                "
            }).unwrap();
            // f_colour = v_colour * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);

        // Construct our `Ui`.
        let mut ui = conrod::UiBuilder::new().theme(support::theme()).build();

        // A demonstration of some app state that we want to control with the conrod GUI.
        let mut app = support::DemoApp::new();

        // The `widget::Id` of each widget instantiated in `gui`.
        let ids = support::Ids::new(ui.widget_id_generator());


        // Add a `Font` to the `Ui`'s `font::Map` from file.
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        // Build the glyph cache and a texture for caching glyphs on the GPU.
        let (mut glyph_cache, text_texture_cache) = {
            let dpi = display.get_window().unwrap().hidpi_factor();
            let cache_width = (WIN_W as f32 * dpi) as u32;
            let cache_height = (WIN_H as f32 * dpi) as u32;

            // First, the rusttype `Cache`, used for caching glyphs onto the GPU.
            const SCALE_TOLERANCE: f32 = 0.1;
            const POSITION_TOLERANCE: f32 = 0.1;
            let cache = conrod::text::GlyphCache::new(cache_width, cache_height,
                                                      SCALE_TOLERANCE,
                                                      POSITION_TOLERANCE);

            // Now the texture.
            let grey_image = glium::texture::RawImage2d {
                data: Cow::Owned(vec![128u8; cache_width as usize * cache_height as usize]),
                width: cache_width,
                height: cache_height,
                format: glium::texture::ClientFormat::U8
            };
            let texture = glium::texture::Texture2d::with_format(
                &display,
                grey_image,
                glium::texture::UncompressedFloatFormat::U8,
                glium::texture::MipmapsOption::NoMipmap
            ).unwrap();

            (cache, texture)
        };

        let window = display.get_window().unwrap();

        // Create our `conrod::image::Map` which describes each of our widget->image mappings.
        // In our case we only have one image, however the macro may be used to list multiple.
        let image_map = image_map! {
            (ids.rust_logo, load_rust_logo(&display)),
        };


        // Start the loop:
        //
        // - Render the current state of the `Ui`.
        // - Update the widgets via `Ui::set_widgets`.
        // - Poll the window for available events.
        // - Repeat.
        'main: loop {


            let ((win_w, win_h), dpi_factor) = {
                (window.get_inner_size_pixels().unwrap(), window.hidpi_factor())
            };

            // Construct a render event for conrod at the beginning of rendering.
            let dt_secs = 0.0;
            ui.handle_event(conrod::event::render(dt_secs, win_w, win_h, dpi_factor as conrod::Scalar));

            // Draw the `Ui`.
            if let Some(mut primitives) = ui.draw_if_changed() {
                use conrod::render;
                use conrod::text::rt;



                implement_vertex!(Vertex, position, tex_coords, colour);

                let (screen_width, screen_height) = {
                    let (w, h) = display.get_framebuffer_dimensions();
                    (w as f32, h as f32)
                };




                // Get render surface + color it.
                // TODO : Figure out why window BG and canvas fail when the window
                // is resized to non-square shapes (i.e aspect ratio != 1:1).
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 0.0);

                let mut queued_events: Vec<GliumQueuedEvent> = Vec::new();



                // Draw each primitive in order of depth.
                while let Some(render::Primitive { id, kind, scizzor, rect }) = primitives.next() {
                    match kind {

                        render::PrimitiveKind::Rectangle { color } => {
                            queued_events.push(GliumQueuedEvent::LazySetGLMode { gl_primitive_mode : glium::index::PrimitiveType::TrianglesList });
                            queued_events.push(GliumQueuedEvent::LazySetUniform { gl_uniform_id : "flat".to_string() });

                            let (l, b, w, h) = rect.l_b_w_h();

                            let color = color.to_fsa();

                            let pos_bl = printed_xy_to_glcoord(l as f32, b as f32, screen_width , screen_height);
                            let pos_br = printed_xy_to_glcoord(l as f32, (b+w) as f32, screen_width , screen_height);
                            let pos_tl = printed_xy_to_glcoord((l+h) as f32, b as f32, screen_width , screen_height);
                            let pos_tr = printed_xy_to_glcoord((l+h) as f32, (b+w) as f32, screen_width , screen_height);

                            let btmlft = Vertex {  position: pos_bl, tex_coords: [0.0, 0.0], colour: color };
                            let btmrgt = Vertex {  position: pos_br, tex_coords: [0.2, 0.0], colour: color };
                            let toplft = Vertex {  position: pos_tl, tex_coords: [0.0, 0.2], colour: color };
                            let toprgt = Vertex {  position: pos_tr, tex_coords: [0.2, 0.2], colour: color };

                            let mut vertices: Vec<Vertex> = Vec::new();

                            vertices.push(btmlft);
                            vertices.push(toplft);
                            vertices.push(toprgt);

                            vertices.push(toprgt);
                            vertices.push(btmrgt);
                            vertices.push(btmlft);

                            queued_events.push(GliumQueuedEvent::QueuedVertexes { vertices : vertices });

                        },

                        render::PrimitiveKind::Polygon { color, points } => {
                            // TODO
                            println!("Got polygon with {} points", points.len());
                            let mut vertices: Vec<Vertex> = Vec::new();
                            let color = color.to_fsa();
                            for point in points.iter() {
                                // Translate points to OGL coords.
                                let pos = xy_to_glcoord(point[0] as f32, point[1] as f32, screen_width , screen_height);

                                let v = Vertex {  position: pos,  tex_coords: [0.2, 0.2], colour: color };
                                vertices.push(v);
                            };

                            // TODO: Triangulate vertices.
                            let triangulated_vertices = triangulate_vertices(vertices);

                            queued_events.push(GliumQueuedEvent::LazySetGLMode { gl_primitive_mode : glium::index::PrimitiveType::TrianglesList });
                            queued_events.push(GliumQueuedEvent::QueuedVertexes { vertices : triangulated_vertices });
                        },

                        render::PrimitiveKind::Lines { color, cap, thickness, points } => {
                            // TODO
                            println!("PK::Lines")
                        },

                        render::PrimitiveKind::Text { color, text, font_id } => {
                            let positioned_glyphs = text.positioned_glyphs(dpi_factor);

                            // Queue the glyphs to be cached.
                            for glyph in positioned_glyphs.iter() {
                                glyph_cache.queue_glyph(font_id.index(), glyph.clone());
                            }

                            // Cache the glyphs on the GPU.
                            glyph_cache.cache_queued(|rect, data| {
                                let glium_rect = glium::Rect {
                                    left: rect.min.x,
                                    bottom: rect.min.y,
                                    width: rect.width(),
                                    height: rect.height()
                                };
                                let image = glium::texture::RawImage2d {
                                    data: Cow::Borrowed(data),
                                    width: rect.width(),
                                    height: rect.height(),
                                    format: glium::texture::ClientFormat::U8
                                };
                                text_texture_cache.main_level().write(glium_rect, image);

                            }).unwrap();

                            let color = color.to_fsa();

                            let cache_id = font_id.index();

                            let origin = rt::point(0.0, 0.0);
                            let to_gl_rect = |screen_rect: conrod::text::rt::Rect<i32>| conrod::text::rt::Rect {
                                min: origin
                                    + (rt::vector(screen_rect.min.x as f32 / screen_width - 0.5,
                                            1.0 - screen_rect.min.y as f32 / screen_height - 0.5)) * 2.0,
                                max: origin
                                    + (rt::vector(screen_rect.max.x as f32 / screen_width - 0.5,
                                            1.0 - screen_rect.max.y as f32 / screen_height - 0.5)) * 2.0
                            };

                            let mut vertices: Vec<Vertex> = Vec::new();
                            let extension = positioned_glyphs.into_iter()
                                .filter_map(|g| glyph_cache.rect_for(cache_id, g).ok().unwrap_or(None))
                                .flat_map(|(uv_rect, screen_rect)| {
                                    use std::iter::once;
                                    let gl_rect = to_gl_rect(screen_rect);
                                    let v = |p, t| once(Vertex {
                                        position: p,
                                        tex_coords: t,
                                        colour: color,
                                    });
                                    v([gl_rect.min.x, gl_rect.max.y], [uv_rect.min.x, uv_rect.max.y])
                                        .chain(v([gl_rect.min.x, gl_rect.min.y], [uv_rect.min.x, uv_rect.min.y]))
                                        .chain(v([gl_rect.max.x, gl_rect.min.y], [uv_rect.max.x, uv_rect.min.y]))
                                        .chain(v([gl_rect.max.x, gl_rect.min.y], [uv_rect.max.x, uv_rect.min.y]))
                                        .chain(v([gl_rect.max.x, gl_rect.max.y], [uv_rect.max.x, uv_rect.max.y]))
                                        .chain(v([gl_rect.min.x, gl_rect.max.y], [uv_rect.min.x, uv_rect.max.y]))
                                });


                            vertices.extend(extension);
                            queued_events.push( GliumQueuedEvent::QueuedVertexes { vertices : vertices } );

                        },

                        render::PrimitiveKind::Image { color, source_rect } => {
                            // id, kind, scizzor, rect
                            println!("PK::Image");
                            queued_events.push(GliumQueuedEvent::LazySetGLMode { gl_primitive_mode : glium::index::PrimitiveType::TrianglesList });
                            queued_events.push(GliumQueuedEvent::LazySetUniform { gl_uniform_id : "textured".to_string() });
                            let (l, b, w, h) = rect.l_b_w_h();
                            let lbwh = [l, b, w, h];

                            let color = match color {
                                Some(c) => c,
                                _ => conrod::color::PURPLE,
                            };

                            let color = color.to_fsa();

                            let pos_bl = printed_xy_to_glcoord(l as f32, b as f32, screen_width , screen_height);
                            let pos_br = printed_xy_to_glcoord(l as f32, (b+w) as f32, screen_width , screen_height);
                            let pos_tl = printed_xy_to_glcoord((l+h) as f32, b as f32, screen_width , screen_height);
                            let pos_tr = printed_xy_to_glcoord((l+h) as f32, (b+w) as f32, screen_width , screen_height);

                            let btmlft = Vertex {  position: pos_bl, tex_coords: [0.0, 0.0], colour: color };
                            let btmrgt = Vertex {  position: pos_br, tex_coords: [0.0, 1.0], colour: color };
                            let toplft = Vertex {  position: pos_tl, tex_coords: [1.0, 0.0], colour: color };
                            let toprgt = Vertex {  position: pos_tr, tex_coords: [1.0, 1.0], colour: color };

                            let mut vertices: Vec<Vertex> = Vec::new();

                            vertices.push(btmlft);
                            vertices.push(toplft);
                            vertices.push(toprgt);

                            vertices.push(toprgt);
                            vertices.push(btmrgt);
                            vertices.push(btmlft);

                            queued_events.push(GliumQueuedEvent::QueuedVertexes { vertices : vertices });

                        },

                        // We have no special case widgets to handle.
                        render::PrimitiveKind::Other(_) => ( println!("PK::Other.") ),
                    }

                }

                // Some orientation vertexes.
                let mut vertices: Vec<Vertex> = Vec::new();
                queued_events.push(GliumQueuedEvent::LazySetGLMode { gl_primitive_mode : glium::index::PrimitiveType::TrianglesList });
                queued_events.push(GliumQueuedEvent::LazySetUniform { gl_uniform_id : "flat".to_string() });

                // Left
                vertices.push(Vertex { position: [-0.9 as f32, 0.2 as f32 ], tex_coords: [0.0, 0.0], colour: conrod::color::BLACK.to_fsa() });
                vertices.push(Vertex { position: [-0.9 as f32, -0.2 as f32 ], tex_coords: [1.0, 0.0], colour: conrod::color::BLACK.to_fsa() });
                vertices.push(Vertex { position: [-0.8 as f32, 0.0 as f32 ], tex_coords: [0.5, 0.5], colour: conrod::color::BLACK.to_fsa() });

                // Right
                queued_events.push(GliumQueuedEvent::LazySetUniform { gl_uniform_id : "textured".to_string() });
                vertices.push(Vertex { position: [0.9 as f32, 0.2 as f32 ], tex_coords: [1.0, 0.0], colour: conrod::color::GREEN.to_fsa() });
                vertices.push(Vertex { position: [0.9 as f32, -0.2 as f32 ], tex_coords: [0.0, 1.0], colour: conrod::color::GREEN.to_fsa() });
                vertices.push(Vertex { position: [0.8 as f32, 0.0 as f32 ], tex_coords: [0.5, 0.5], colour: conrod::color::GREEN.to_fsa() });

                // Top
                vertices.push(Vertex { position: [-0.2 as f32, 0.9 as f32 ], tex_coords: [1.0, 0.0], colour: conrod::color::BLUE.to_fsa() });
                vertices.push(Vertex { position: [0.2 as f32, 0.9 as f32 ], tex_coords: [0.0, 1.0], colour: conrod::color::BLUE.to_fsa() });
                vertices.push(Vertex { position: [0.0 as f32, 0.8 as f32 ], tex_coords: [0.5, 0.5], colour: conrod::color::BLUE.to_fsa() });
                queued_events.push(GliumQueuedEvent::LazySetUniform { gl_uniform_id : "flat".to_string() });

                // Bottom
                vertices.push(Vertex { position: [-0.2 as f32, -0.9 as f32 ], tex_coords: [1.0, 0.0], colour: conrod::color::RED.to_fsa() });
                vertices.push(Vertex { position: [0.2 as f32,  -0.9 as f32 ], tex_coords: [0.0, 1.0], colour: conrod::color::RED.to_fsa() });
                vertices.push(Vertex { position: [-0.0 as f32, -0.8 as f32 ], tex_coords: [0.5, 0.5], colour: conrod::color::RED.to_fsa() });

                queued_events.push(GliumQueuedEvent::QueuedVertexes { vertices : vertices });


                // Hack: These switches will cause the queue to flush.
                queued_events.push(GliumQueuedEvent::LazySetGLMode { gl_primitive_mode : glium::index::PrimitiveType::LineStrip  });
                queued_events.push(GliumQueuedEvent::LazySetGLMode { gl_primitive_mode : glium::index::PrimitiveType::TrianglesList });


                let mut queued_vertices : Vec<Vertex> = Vec::new();
                let blend = glium::Blend::alpha_blending();
                let draw_params = glium::DrawParameters { blend: blend, ..Default::default() };
                // Uniforms are different for each texture.
                let uniform_flat = uniform! {
                    tex: text_texture_cache.sampled()
                        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                };

                let tex : &glium::texture::SrgbTexture2d = image_map.get(&ids.rust_logo).unwrap();

                let uniform_textured = uniform! {
                    matrix: [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0f32],
                    ],
                    tex: glium::uniforms::Sampler::new(tex).magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
                };


                // Select primitive type (tristrip, lines etc).
                //let no_indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
                //let no_indices = glium::index::NoIndices(glium::index::PrimitiveType::LineStrip);
                let mut previous_gl_primitive_mode = glium::index::PrimitiveType::TrianglesList;
                let mut no_indices = glium::index::NoIndices(previous_gl_primitive_mode);

                let mut previous_uniform_id = "flat".to_string();

                queued_events.push(GliumQueuedEvent::LazySetGLMode { gl_primitive_mode : glium::index::PrimitiveType::LineStrip  });

                // Process glium queue here.
                // Above this point :
                //  - all primitives shall be constructed & triangulated
                //  - all textures loaded
                //
                // Below this point :
                //  - all render calls

                for queued_event in queued_events {

                    match queued_event {
                        GliumQueuedEvent::QueuedVertexes { vertices : vertices } => {
                            queued_vertices.extend(vertices);
                        },

                        GliumQueuedEvent::LazySetGLMode { gl_primitive_mode : gl_primitive_mode } => {

                            if gl_primitive_mode != previous_gl_primitive_mode {
                                // Only push for a render if we need to switch GL mode.
                                // If not, just let the vertice queue build up.

                                println!("Flushing {} vertices using mode {} and uniform_id {}.", queued_vertices.len(), mode_as_str(previous_gl_primitive_mode), previous_uniform_id);
                                no_indices = glium::index::NoIndices(previous_gl_primitive_mode);

                                if queued_vertices.len() < 10 {
                                    for &ver in &queued_vertices {
                                        let Vertex { position, tex_coords, colour} = ver;
                                        println!("Vertex at coord {}, {}", position[0], position[1])

                                    }
                                };

                                // Fetch vertices.
                                let vertex_buffer = glium::VertexBuffer::new(&display, &queued_vertices).unwrap();
                                queued_vertices.truncate(0); // Throw away all vertices... Assume this is more efficient than creating a new Vec.

                                // Actual draw call. The number of calls should be minimized, but unknown how.
                                // I.e { colored_square, textured_square, colored_square } will cause
                                // three seperate calls, but the colored calls should be grouped for performance.
                                // However this won't work if we want one square to draw over the other.
                                // Perhaps in the future we can sort all 2d draws by z-buffer.
                                //target.draw(&vertex_buffer, no_indices, &program, previous_uniform, &draw_params).unwrap();
                                if previous_uniform_id == "textured" {
                                    target.draw(&vertex_buffer, no_indices, &program, &uniform_textured, &draw_params).unwrap();
                                } else {
                                    target.draw(&vertex_buffer, no_indices, &program, &uniform_flat, &draw_params).unwrap();
                                };

                                println!("Switching GL mode {} -> {}", mode_as_str(gl_primitive_mode), mode_as_str(previous_gl_primitive_mode));
                                previous_gl_primitive_mode = gl_primitive_mode;

                            }
                        },

                        GliumQueuedEvent::LazySetUniform { gl_uniform_id } => {
                            if gl_uniform_id == previous_uniform_id { continue }

                            println!("Flushing {} vertices using mode {} and uniform_id {} due to uni change.", queued_vertices.len(), mode_as_str(previous_gl_primitive_mode), previous_uniform_id);
                            no_indices = glium::index::NoIndices(previous_gl_primitive_mode);

                            if queued_vertices.len() < 10 {
                                for &ver in &queued_vertices {
                                    let Vertex { position, tex_coords, colour} = ver;
                                    println!("Vertex at coord {}, {}", position[0], position[1])

                                }
                            };

                            // Fetch vertices.
                            let vertex_buffer = glium::VertexBuffer::new(&display, &queued_vertices).unwrap();
                            queued_vertices.truncate(0); // Throw away all vertices... Assume this is more efficient than creating a new Vec.

                            // TODO:
                            if previous_uniform_id == "textured" {
                                target.draw(&vertex_buffer, no_indices, &program, &uniform_textured, &draw_params).unwrap();
                            } else {
                                target.draw(&vertex_buffer, no_indices, &program, &uniform_flat, &draw_params).unwrap();
                            };

                            previous_uniform_id = gl_uniform_id;

                        },
                    }
                }

                target.finish().unwrap();
            }

            for event in display.poll_events() {

                // Use the `glutin` backend feature to convert the glutin event to a conrod one.
                let (w, h) = (win_w as conrod::Scalar, win_h as conrod::Scalar);
                let dpi_factor = dpi_factor as conrod::Scalar;
                if let Some(event) = conrod::backend::glutin::convert(event.clone(), w, h, dpi_factor) {
                    ui.handle_event(event);
                }

                match event {
                    // Break from the loop upon `Escape`.
                    glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                    glutin::Event::Closed =>
                        break 'main,

                    _ => {},
                }
            }

            if ui.global_input.events().next().is_some() {
                // Instantiate a GUI demonstrating every widget type provided by conrod.
                let mut ui = ui.set_widgets();
                support::gui(&mut ui, &ids, &mut app);
            }

            // Avoid hogging the CPU.
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    // Load the Rust logo from our assets folder.
    // TODO: Rename this method or something.
    // TODO: Figure out if we can replace image opening. Do we really need image in the .toml file?
    fn load_rust_logo(display : &glium::Display) -> glium::texture::SrgbTexture2d {
        let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
        let path = assets.join("images/eyesore.png");
        let rgba_image = image::open(&Path::new(&path)).unwrap().to_rgba();
        let image_dimensions = rgba_image.dimensions();
        let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(rgba_image.into_raw(), image_dimensions);
        let rust_logo_texture = glium::texture::SrgbTexture2d::new(display, raw_image).unwrap();
        rust_logo_texture
    }


    fn triangulate_vertices(vertices: Vec<Vertex>) -> Vec<Vertex> {
        // Very basic triangulation algo.
        // Will fail for any concave polygon.
        // Texture coords ignored.
        let mut toreturn : Vec<Vertex> = Vec::new();
        let mut average_x = 0.0;
        let mut average_y = 0.0;

        for vert in &vertices {
            let Vertex { position, tex_coords, colour } = *vert;
            average_x = average_x + position[0];
            average_y = average_y + position[1];
        };

        average_x = average_x / (vertices.len() as f32);
        average_y = average_y / (vertices.len() as f32);

        let mut last_vert = vertices[0];

        for vert in vertices {

            if vert == last_vert { continue }
            let Vertex { position, tex_coords, colour } = vert;
            let center = Vertex { position : [average_x, average_y] , tex_coords : tex_coords, colour : colour };

            // Every segment in the poly outline is triangulated
            // to a (center, last_vertex, current_vertex) triangle.
            toreturn.push(center);
            toreturn.push(last_vert);
            toreturn.push(vert);

            last_vert = vert;

        };

        toreturn
    }
}

#[cfg(not(feature="glutin"))]
#[cfg(not(feature="glium"))]
mod feature {
    pub fn main() {
        println!("This example requires the `glutin` and `glium` features. \
                 Try running `cargo run --release --no-default-features --features=\"glutin glium\" --example <example_name>`");
    }
}
