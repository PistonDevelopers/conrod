//! A demonstration using glutin to provide events and glium for drawing the Ui.

#[cfg(feature="glutin")] #[cfg(feature="glium")] #[macro_use] extern crate conrod;
#[cfg(feature="glutin")] #[cfg(feature="glium")] #[macro_use] extern crate glium;

#[cfg(feature="glutin")]
mod support;

fn main() {
    feature::main();
}

#[cfg(feature="glutin")]
#[cfg(feature="glium")]
mod feature {
    extern crate find_folder;
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

    pub fn main() {

        // Build the window.
        let display = glutin::WindowBuilder::new()
            .with_vsync()
            .with_dimensions(WIN_W, WIN_H)
            .with_title("Conrod with glutin & glium!")
            .build_glium()
            .unwrap();

        // Create the `GL` program used for drawing textured stuff (i.e. `Image`s or `Text` from
        // the cache).
        let program_textured = program!(
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
                        f_colour = v_colour * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);
                    }
                "
            }).unwrap();

        // Create the `GL` program used for drawing basic coloured geometry (i.e. `Rectangle`s,
        // `Line`s or `Polygon`s).
        let program = program!(
            &display,
            140 => {
                vertex: "
                    #version 140

                    in vec2 position;
                    in vec4 colour;

                    out vec4 v_colour;

                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                        v_colour = colour;
                    }
                ",

                fragment: "
                    #version 140
                    in vec4 v_colour;
                    out vec4 f_colour;

                    void main() {
                        f_colour = v_colour;
                    }
                "
            }).unwrap();


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

        // Start the loop:
        //
        // - Render the current state of the `Ui`.
        // - Update the widgets via the `support::gui` fn.
        // - Poll the window for available events.
        // - Repeat.
        'main: loop {

            let ((win_w, win_h), dpi_factor) = {
                let window = display.get_window().unwrap();
                (window.get_inner_size_pixels().unwrap(), window.hidpi_factor())
            };

            // Construct a render event for conrod at the beginning of rendering.
            let dt_secs = 0.0;
            ui.handle_event(conrod::event::render(dt_secs, win_w, win_h, dpi_factor as conrod::Scalar));

            // Draw the `Ui`.
            if let Some(mut primitives) = ui.draw_if_changed() {
                use conrod::render;
                use conrod::text::rt;

                #[derive(Copy, Clone)]
                struct TexturedVertex {
                    position: [f32; 2],
                    tex_coords: [f32; 2],
                    colour: [f32; 4]
                }

                #[derive(Copy, Clone)]
                struct PlainVertex {
                    position: [f32; 2],
                    colour: [f32; 4],
                }

                implement_vertex!(TexturedVertex, position, tex_coords, colour);
                implement_vertex!(PlainVertex, position, colour);

                let (screen_width, screen_height) = {
                    let (w, h) = display.get_framebuffer_dimensions();
                    (w as f32, h as f32)
                };

                let half_win_w = win_w as conrod::Scalar / 2.0;
                let half_win_h = win_h as conrod::Scalar / 2.0;

                pub enum Command {
                    /// A range of vertices representing triangulated text.
                    Text(std::ops::Range<usize>),
                    /// A range of vertices representing triangulated rectangles.
                    Rectangles(std::ops::Range<usize>),
                }

                pub enum State {
                    Text { start: usize },
                    Rectangles { start: usize },
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

                let mut textured_vertices: Vec<TexturedVertex> = Vec::new();
                let mut plain_vertices: Vec<PlainVertex> = Vec::new();
                let mut commands: Vec<Command> = Vec::new();
                let mut current_state = State::Rectangles { start: 0 };

                // Draw each primitive in order of depth.
                while let Some(render::Primitive { id, kind, scizzor, rect }) = primitives.next() {
                    match kind {

                        render::PrimitiveKind::Rectangle { color } => {
                            // Ensure we're in the `Rectangle` state.
                            match current_state {
                                State::Rectangles { .. } => (),
                                State::Text { start } => {
                                    commands.push(Command::Text(start..textured_vertices.len()));
                                    current_state = State::Rectangles { start: plain_vertices.len() };
                                },
                            }

                            let color = gamma_srgb_to_linear(color.to_fsa());
                            let (l, r, b, t) = rect.l_r_b_t();

                            let v = |x, y| {
                                // Convert from conrod Scalar range to GL range -1.0 to 1.0.
                                let x = (x * dpi_factor as conrod::Scalar / half_win_w) as f32;
                                let y = (y * dpi_factor as conrod::Scalar / half_win_h) as f32;
                                PlainVertex {
                                    position: [x, y],
                                    colour: color,
                                }
                            };

                            let mut push_v = |x, y| plain_vertices.push(v(x, y));

                            // Bottom left triangle.
                            push_v(l, t);
                            push_v(r, b);
                            push_v(l, b);

                            // Top right triangle.
                            push_v(l, t);
                            push_v(r, b);
                            push_v(r, t);
                        },

                        render::PrimitiveKind::Polygon { color, points } => {

                            let color = gamma_srgb_to_linear(color.to_fsa());

                            // TODO
                        },

                        render::PrimitiveKind::Lines { color, cap, thickness, points } => {
                            // TODO
                        },

                        render::PrimitiveKind::Text { color, text, font_id } => {
                            // Switch to the `Text` state if we're not in it already.
                            match current_state {
                                State::Text { .. } => (),
                                State::Rectangles { start } => {
                                    commands.push(Command::Rectangles(start..plain_vertices.len()));
                                    current_state = State::Text { start: textured_vertices.len() };
                                },
                            }

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

                            let color = gamma_srgb_to_linear(color.to_fsa());

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

                            let extension = positioned_glyphs.into_iter()
                                .filter_map(|g| glyph_cache.rect_for(cache_id, g).ok().unwrap_or(None))
                                .flat_map(|(uv_rect, screen_rect)| {
                                    use std::iter::once;
                                    let gl_rect = to_gl_rect(screen_rect);
                                    let v = |p, t| once(TexturedVertex {
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

                            textured_vertices.extend(extension);
                        },

                        render::PrimitiveKind::Image { color, source_rect } => {
                            // TODO
                        },

                        // We have no special case widgets to handle.
                        render::PrimitiveKind::Other(_) => (),
                    }

                }

                // Enter the final command.
                match current_state {
                    State::Rectangles { start } => commands.push(Command::Rectangles(start..plain_vertices.len())),
                    State::Text { start } => commands.push(Command::Text(start..textured_vertices.len())),
                }

                let text_uniforms = uniform! {
                    tex: text_texture_cache.sampled()
                        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                };
                let rect_uniforms = glium::uniforms::EmptyUniforms;

                let blend = glium::Blend::alpha_blending();
                let draw_params = glium::DrawParameters { blend: blend, ..Default::default() };

                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);

                println!("draw");
                for command in commands {
                    match command {

                        Command::Text(range) => {
                            println!("\ttext: {:?}", &range);
                            let slice = &textured_vertices[range];
                            let vertex_buffer = glium::VertexBuffer::new(&display, slice).unwrap();
                            let no_indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
                            target.draw(&vertex_buffer, no_indices, &program_textured, &text_uniforms, &draw_params).unwrap();
                        },

                        Command::Rectangles(range) => {
                            println!("\trectangles: {:?}", &range);
                            let slice = &plain_vertices[range];
                            let vertex_buffer = glium::VertexBuffer::new(&display, slice).unwrap();
                            let no_indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
                            target.draw(&vertex_buffer, no_indices, &program, &rect_uniforms, &draw_params).unwrap();
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

}

#[cfg(not(feature="glutin"))]
#[cfg(not(feature="glium"))]
mod feature {
    pub fn main() {
        println!("This example requires the `glutin` and `glium` features. \
                 Try running `cargo run --release --no-default-features --features=\"glutin glium\" --example <example_name>`");
    }
}
