//! A demonstration using glutin to provide events and glium for drawing the Ui.

#[cfg(feature="glutin")] #[cfg(feature="glium")] #[macro_use] extern crate conrod;
#[cfg(feature="glutin")] #[cfg(feature="glium")] #[macro_use] extern crate glium;
#[cfg(feature="glutin")] #[cfg(feature="glium")] #[macro_use] extern crate graphics;


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
    use std;


    use glium::{DisplayBuild, Surface};
    use glium::glutin;

    use std::borrow::Cow;

    // The width and height in "points".
    const WIN_W: u32 = 512;
    const WIN_H: u32 = 512;


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


        let rust_logo_texture = load_rust_logo(&display);


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

                        vec3 mixed_colour = mix(tex_part, vrt_part, 0.5);
                        f_colour = vec4(mixed_colour, 1.0);
                    }
                "
            }).unwrap();
            // f_colour = v_colour * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);

        // Construct our `Ui`.
        let mut ui = conrod::UiBuilder::new().build();

        let ids = Ids::new(ui.widget_id_generator());


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
            (ids.rust_logo_texture, rust_logo_texture),
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
                target.clear_color(0.4, 0.4, 0.8, 0.4);

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

                let tex : &glium::texture::SrgbTexture2d = image_map.get(&ids.rust_logo_texture).unwrap();

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
                // Update all widgets within the `Ui`.
                set_widgets(ui.set_widgets(), &ids);
            }

            // Avoid hogging the CPU.
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }


    // Generate a type which may produce unique identifier for each widget.
    widget_ids! {
        struct Ids {
            canvas,
            fish,
            rust_logo,
            rust_logo_texture,
            circle,
            toggle,
            button,
            text,
        }
    }


    /// Instantiate the widgets.
    fn set_widgets(ref mut ui: conrod::UiCell, ids: &Ids) {
        use conrod::{widget, Colorable, Positionable, Sizeable, Widget};

        widget::Canvas::new().color(conrod::color::DARK_CHARCOAL).set(ids.canvas, ui);

        // Some starting text to edit.
        let demo_text = "Some fish.";

        //conrod::Text::new("Foo! Bar! Baz!\nFloozy Woozy\nQux Flux")
        widget::Text::new(demo_text)
            .middle_of(ids.canvas)
            .wh_of(ids.canvas)
            .font_size(20)
            .color(conrod::color::BLACK)
            .align_text_middle()
            .set(ids.text, ui);

        // Fish polygon.
        let mut fish_vertexes =  vec![[-0.698435, 0.327139], [-0.908714, 0.200846], [-1.000000, 0.045003], [-0.882569, -0.153804], [-0.716302, -0.243929], [-0.419474, -0.310119], [-0.086611, -0.607857], [0.091828, -0.619086], [-0.041731, -0.307105], [0.322430, -0.227440], [0.544011, -0.121772], [0.642794, -0.220526], [0.809389, -0.322116], [1.000000, -0.378260], [0.800696, -0.049199], [0.797215, 0.151499], [0.965221, 0.456860], [0.755156, 0.402490], [0.573567, 0.255689], [0.397194, 0.317920], [0.513037, 0.371109], [0.473372, 0.466375], [0.340141, 0.466375], [0.152681, 0.354088], [-0.125625, 0.399003], [0.053142, 0.551713], [-0.009126, 0.619086], [-0.206692, 0.619086], [-0.420234, 0.381569], [-0.698439, 0.324244], [-0.698435, 0.327139]];

        let fish_scale = ui.window_dim();
        for n in 0..fish_vertexes.len() {
            fish_vertexes[n][0] = fish_vertexes[n][0] * fish_scale[0] / 2.0;
            fish_vertexes[n][1] = fish_vertexes[n][1] * fish_scale[1] / 2.0;
        };

        widget::Polygon::fill(fish_vertexes)
            .top_left_of(ids.canvas) // TODO: Figure out why move & scale does nothing.
            .color(conrod::color::RED)
            .set(ids.fish, ui);


        // Logo
        widget::Image::new()
            .w_h(128.0 as f64, 128.0 as f64)
            .top_right_of(ids.canvas)
            //.down(0.01).left(0.01)  // TODO: Figure out why this row makes the logo disappear.
            .set(ids.rust_logo, ui);


        // Draw a circle at the app's circle_pos.
        widget::Circle::fill(64.0)
            .top_left_of(ids.canvas)
            .color(conrod::color::GREEN)
            .set(ids.circle, ui);

        let mut toggle_value = false;
        let mut toggle_label = "OFF".to_string();

        if let Some(value) = widget::Toggle::new(toggle_value)
            .w_h(75.0, 75.0)
            .bottom_left_of(ids.canvas)
            .down(20.0)
            .rgb(0.6, 0.25, 0.75)
            //.border(2.0)  // TODO: Figure out why these rows don't compile.
            //.label(&toggle_label)
            //.label_color(conrod::color::WHITE)
            .set(ids.toggle, ui)
            .last()
        {
            toggle_label = match toggle_value {
                true => "ON".to_string(),
                false => "OFF".to_string()
            }
        };


        // Button widget example button.
        if widget::Button::new()
            .w_h(200.0, 50.0)
            .bottom_right_of(ids.canvas)
            .rgb(0.4, 0.75, 0.6)
            //.border(2.0)
            //.label("PRESS")
            .set(ids.button, ui)
            .was_clicked()
        {
            println!("Click!");
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

