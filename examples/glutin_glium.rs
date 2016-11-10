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
    extern crate image;
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
        let program = program!(
            &display,
            140 => {
                vertex: "
                    #version 140

                    in vec2 position;
                    in vec2 tex_coords;
                    in vec4 colour;
                    // Describes the mode of rendering:
                    //     0 -> text
                    //     1 -> image
                    //     2 -> geometry
                    in uint mode;

                    out vec2 v_tex_coords;
                    out vec4 v_colour;
                    flat out uint v_mode;

                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                        v_tex_coords = tex_coords;
                        v_colour = colour;
                        v_mode = mode;
                    }
                ",
                fragment: "
                    #version 140
                    uniform sampler2D tex;

                    in vec2 v_tex_coords;
                    in vec4 v_colour;
                    flat in uint v_mode;

                    out vec4 f_colour;

                    void main() {

                        // Text
                        if (v_mode == uint(0)) {
                            f_colour = v_colour * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);

                        // Image
                        } else if (v_mode == uint(1)) {
                            f_colour = texture(tex, v_tex_coords);

                        // 2D Geometry
                        } else if (v_mode == uint(2)) {
                            f_colour = v_colour;
                        }
                    }
                ",
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

        // Load the Rust logo from our assets folder to use as an example image.
        fn load_rust_logo(display: &glium::Display) -> glium::texture::Texture2d {
            let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
            let path = assets.join("images/rust.png");
            let rgba_image = image::open(&std::path::Path::new(&path)).unwrap().to_rgba();
            let image_dimensions = rgba_image.dimensions();
            let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(rgba_image.into_raw(), image_dimensions);
            let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();
            texture
        }

        let image_map = support::image_map(&ids, load_rust_logo(&display));

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

            // Poll for events.
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

            // Draw the `Ui`.
            if let Some(mut primitives) = ui.draw_if_changed() {
                use conrod::render;
                use conrod::text::rt;

                const MODE_TEXT: u32 = 0;
                const MODE_IMAGE: u32 = 1;
                const MODE_GEOMETRY: u32 = 2;

                #[derive(Copy, Clone)]
                struct Vertex {
                    position: [f32; 2],
                    tex_coords: [f32; 2],
                    colour: [f32; 4],
                    mode: u32,
                }

                implement_vertex!(Vertex, position, tex_coords, colour, mode);

                let (screen_width, screen_height) = {
                    let (w, h) = display.get_framebuffer_dimensions();
                    (w as f32, h as f32)
                };

                let half_win_w = win_w as conrod::Scalar / 2.0;
                let half_win_h = win_h as conrod::Scalar / 2.0;

                pub enum Command {
                    /// A range of vertices representing triangles textured with the image in the
                    /// image_map at the given `widget::Id`..
                    Image(conrod::widget::Id, std::ops::Range<usize>),
                    /// A range of vertices representing plain triangles.
                    Plain(std::ops::Range<usize>),
                }

                pub enum State {
                    Image { id: conrod::widget::Id, start: usize },
                    Plain { start: usize },
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

                let mut vertices: Vec<Vertex> = Vec::new();
                let mut commands: Vec<Command> = Vec::new();
                let mut current_state = State::Plain { start: 0 };

                // Functions for converting for conrod coords to GL vertex coords (-1.0 to 1.0).
                let vx = |x: conrod::Scalar| (x * dpi_factor as conrod::Scalar / half_win_w) as f32;
                let vy = |y: conrod::Scalar| (y * dpi_factor as conrod::Scalar / half_win_h) as f32;

                // Draw each primitive in order of depth.
                while let Some(render::Primitive { id, kind, scizzor, rect }) = primitives.next() {
                    match kind {

                        render::PrimitiveKind::Rectangle { color } => {
                            // Switch to `Plain` state if we're not in it already.
                            match current_state {
                                State::Plain { .. } => (),
                                State::Image { id, start } => {
                                    commands.push(Command::Image(id, start..vertices.len()));
                                    current_state = State::Plain { start: vertices.len() };
                                },
                            }

                            let color = gamma_srgb_to_linear(color.to_fsa());
                            let (l, r, b, t) = rect.l_r_b_t();

                            let v = |x, y| {
                                // Convert from conrod Scalar range to GL range -1.0 to 1.0.
                                Vertex {
                                    position: [vx(x), vy(y)],
                                    tex_coords: [0.0, 0.0],
                                    colour: color,
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
                        },

                        render::PrimitiveKind::Polygon { color, points } => {
                            // If we don't at least have a triangle, keep looping.
                            if points.len() < 3 {
                                continue;
                            }

                            // Switch to `Plain` state if we're not in it already.
                            match current_state {
                                State::Plain { .. } => (),
                                State::Image { id, start } => {
                                    commands.push(Command::Image(id, start..vertices.len()));
                                    current_state = State::Plain { start: vertices.len() };
                                },
                            }

                            let color = gamma_srgb_to_linear(color.to_fsa());

                            let v = |p: [conrod::Scalar; 2]| {
                                Vertex {
                                    position: [vx(p[0]), vy(p[1])],
                                    tex_coords: [0.0, 0.0],
                                    colour: color,
                                    mode: MODE_GEOMETRY,
                                }
                            };

                            // Triangulate the polygon.
                            //
                            // Make triangles between the first point and every following pair of
                            // points.
                            //
                            // For example, for a polygon with 6 points (a to f), this makes the
                            // following triangles: abc, acd, ade, aef.
                            let first = points[0];
                            let first_v = v(first);
                            let mut prev_v = v(points[1]);
                            let mut push_v = |v| vertices.push(v);
                            for &p in &points[2..] {
                                let v = v(p);
                                push_v(first_v);
                                push_v(prev_v);
                                push_v(v);
                                prev_v = v;
                            }
                        },

                        render::PrimitiveKind::Lines { color, cap, thickness, points } => {

                            // We need at least two points to draw any lines.
                            if points.len() < 2 {
                                continue;
                            }

                            // Switch to `Plain` state if we're not in it already.
                            match current_state {
                                State::Plain { .. } => (),
                                State::Image { id, start } => {
                                    commands.push(Command::Image(id, start..vertices.len()));
                                    current_state = State::Plain { start: vertices.len() };
                                },
                            }

                            let color = gamma_srgb_to_linear(color.to_fsa());

                            let v = |p: [conrod::Scalar; 2]| {
                                Vertex {
                                    position: [vx(p[0]), vy(p[1])],
                                    tex_coords: [0.0, 0.0],
                                    colour: color,
                                    mode: MODE_GEOMETRY,
                                }
                            };

                            // Convert each line to a rectangle for triangulation.
                            //
                            // TODO: handle `cap` and properly join consecutive lines considering
                            // the miter. Discussion here:
                            // https://forum.libcinder.org/topic/smooth-thick-lines-using-geometry-shader#23286000001269127
                            let mut a = points[0];
                            for &b in &points[1..] {

                                let direction = [b[0] - a[0], b[1] - a[1]];
                                let mag = (direction[0].powi(2) + direction[1].powi(2)).sqrt();
                                let unit = [direction[0] / mag, direction[1] / mag];
                                let normal = [-unit[1], unit[0]];
                                let half_thickness = thickness / 2.0;

                                // A perpendicular line with length half the thickness.
                                let n = [normal[0] * half_thickness, normal[1] * half_thickness];

                                // The corners of the rectangle as GL vertices.
                                let (r1, r2, r3, r4);
                                r1 = v([a[0] + n[0], a[1] + n[1]]);
                                r2 = v([a[0] - n[0], a[1] - n[1]]);
                                r3 = v([b[0] + n[0], b[1] + n[1]]);
                                r4 = v([b[0] - n[0], b[1] - n[1]]);

                                // Push the rectangle's vertices.
                                let mut push_v = |v| vertices.push(v);
                                push_v(r1);
                                push_v(r4);
                                push_v(r2);
                                push_v(r1);
                                push_v(r4);
                                push_v(r3);

                                a = b;
                            }
                        },

                        render::PrimitiveKind::Text { color, text, font_id } => {
                            // Switch to the `Textured` state if we're not in it already.
                            match current_state {
                                State::Plain { .. } => (),
                                State::Image { id, start } => {
                                    commands.push(Command::Image(id, start..vertices.len()));
                                    current_state = State::Plain { start: vertices.len() };
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

                            for g in positioned_glyphs {
                                if let Ok(Some((uv_rect, screen_rect))) = glyph_cache.rect_for(cache_id, g) {
                                    let gl_rect = to_gl_rect(screen_rect);
                                    let v = |p, t| Vertex {
                                        position: p,
                                        tex_coords: t,
                                        colour: color,
                                        mode: MODE_TEXT,
                                    };
                                    let mut push_v = |p, t| vertices.push(v(p, t));
                                    push_v([gl_rect.min.x, gl_rect.max.y], [uv_rect.min.x, uv_rect.max.y]);
                                    push_v([gl_rect.min.x, gl_rect.min.y], [uv_rect.min.x, uv_rect.min.y]);
                                    push_v([gl_rect.max.x, gl_rect.min.y], [uv_rect.max.x, uv_rect.min.y]);
                                    push_v([gl_rect.max.x, gl_rect.min.y], [uv_rect.max.x, uv_rect.min.y]);
                                    push_v([gl_rect.max.x, gl_rect.max.y], [uv_rect.max.x, uv_rect.max.y]);
                                    push_v([gl_rect.min.x, gl_rect.max.y], [uv_rect.min.x, uv_rect.max.y]);
                                }
                            }
                        },

                        render::PrimitiveKind::Image { color, source_rect } => {
                            // Switch to the `Textured` state if we're not in it already.
                            let widget_id = id;
                            match current_state {
                                State::Image { id, .. } if id == widget_id => (),
                                State::Plain { start } => {
                                    commands.push(Command::Plain(start..vertices.len()));
                                    current_state = State::Image { id: id, start: vertices.len() };
                                },
                                State::Image { id, start } => {
                                    commands.push(Command::Image(id, start..vertices.len()));
                                    current_state = State::Image { id: id, start: vertices.len() };
                                },
                            }

                            let color = gamma_srgb_to_linear(color.unwrap_or(conrod::color::WHITE).to_fsa());

                            let image = image_map.get(&id).unwrap();
                            let image_w = image.get_width() as conrod::Scalar;
                            let image_h = image.get_height().unwrap() as conrod::Scalar;

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
                                None => (0.0, 1.0, 0.0, 1.0),
                            };

                            let v = |x, y, t| {
                                // Convert from conrod Scalar range to GL range -1.0 to 1.0.
                                let x = (x * dpi_factor as conrod::Scalar / half_win_w) as f32;
                                let y = (y * dpi_factor as conrod::Scalar / half_win_h) as f32;
                                Vertex {
                                    position: [x, y],
                                    tex_coords: t,
                                    colour: color,
                                    mode: MODE_IMAGE,
                                }
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
                        },

                        // We have no special case widgets to handle.
                        render::PrimitiveKind::Other(_) => (),
                    }

                }

                // Enter the final command.
                match current_state {
                    State::Plain { start } => commands.push(Command::Plain(start..vertices.len())),
                    State::Image { id, start } => commands.push(Command::Image(id, start..vertices.len())),
                }

                let text_uniforms = uniform! {
                    tex: text_texture_cache.sampled()
                        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                };

                let blend = glium::Blend::alpha_blending();
                let draw_params = glium::DrawParameters { blend: blend, ..Default::default() };
                let no_indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 1.0, 1.0);

                for command in commands {
                    match command {

                        Command::Plain(range) => {
                            let slice = &vertices[range];
                            let vertex_buffer = glium::VertexBuffer::new(&display, slice).unwrap();
                            target.draw(&vertex_buffer, no_indices, &program, &text_uniforms, &draw_params).unwrap();
                        },

                        Command::Image(id, range) => {
                            let slice = &vertices[range];
                            let vertex_buffer = glium::VertexBuffer::new(&display, slice).unwrap();
                            let image = image_map.get(&id).unwrap();
                            let uniforms = uniform! {
                                tex: image.sampled()
                                    .wrap_function(glium::uniforms::SamplerWrapFunction::Clamp)
                                    .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest),
                            };
                            target.draw(&vertex_buffer, no_indices, &program, &uniforms, &draw_params).unwrap();
                        },

                    }
                }

                target.finish().unwrap();
            }

            // Avoid hogging the CPU.
            std::thread::sleep(std::time::Duration::from_millis(16));
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
