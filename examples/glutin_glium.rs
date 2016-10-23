//! A demonstration using glutin to provide events and glium for drawing the Ui.

#[cfg(feature="glutin")] #[cfg(feature="glium")] #[macro_use] extern crate conrod;
#[cfg(feature="glutin")] #[cfg(feature="glium")] #[macro_use] extern crate glium;

fn main() {
    feature::main();
}

#[cfg(feature="glutin")]
#[cfg(feature="glium")]
mod feature {
    extern crate find_folder;
    use conrod;
    use glium;
    use std;

    use glium::{DisplayBuild, Surface};
    use glium::glutin;

    use std::borrow::Cow;

    // The width and height in "points".
    const WIN_W: u32 = 512;
    const WIN_H: u32 = 512;

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
                        f_colour = v_colour * vec4(1.0, 1.0, 1.0, texture(tex, v_tex_coords).r);
                    }
                "
            }).unwrap();

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

        // Start the loop:
        //
        // - Render the current state of the `Ui`.
        // - Update the widgets via `Ui::set_widgets`.
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
                struct Vertex {
                    position: [f32; 2],
                    tex_coords: [f32; 2],
                    colour: [f32; 4]
                }

                implement_vertex!(Vertex, position, tex_coords, colour);

                let (screen_width, screen_height) = {
                    let (w, h) = display.get_framebuffer_dimensions();
                    (w as f32, h as f32)
                };

                let mut vertices: Vec<Vertex> = Vec::new();

                // Draw each primitive in order of depth.
                while let Some(render::Primitive { id, kind, scizzor, rect }) = primitives.next() {
                    match kind {

                        render::PrimitiveKind::Rectangle { color } => {
                            // TODO
                        },

                        render::PrimitiveKind::Polygon { color, points } => {
                            // TODO
                            let color = color.to_fsa();
                            for point in points.iter() {
                                let v = Vertex {  position: [point[0] as f32, point[1] as f32],  tex_coords: [0.2, 0.2], colour: color };
                                vertices.push(v);
                            };
                        },

                        render::PrimitiveKind::Lines { color, cap, thickness, points } => {
                            // TODO
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
                        },

                        render::PrimitiveKind::Image { color, source_rect } => {
                            // TODO
                        },

                        // We have no special case widgets to handle.
                        render::PrimitiveKind::Other(_) => (),
                    }

                }

                let uniforms = uniform! {
                    tex: text_texture_cache.sampled()
                        .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                };

                let mut target = display.draw();
                target.clear_color(1.0, 1.0, 1.0, 0.0);
                let vertex_buffer = glium::VertexBuffer::new(&display, &vertices).unwrap();
                let blend = glium::Blend::alpha_blending();
                let no_indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
                //let no_indices = glium::index::NoIndices(glium::index::PrimitiveType::LineStrip);
                let draw_params = glium::DrawParameters { blend: blend, ..Default::default() };
                target.draw(&vertex_buffer, no_indices, &program, &uniforms, &draw_params).unwrap();
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
            line,
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

        let vertexes =  vec![[-0.698435, 0.327139], [-0.908714, 0.200846], [-1.000000, 0.045003], [-0.882569, -0.153804], [-0.716302, -0.243929], [-0.419474, -0.310119], [-0.086611, -0.607857], [0.091828, -0.619086], [-0.041731, -0.307105], [0.322430, -0.227440], [0.544011, -0.121772], [0.642794, -0.220526], [0.809389, -0.322116], [1.000000, -0.378260], [0.800696, -0.049199], [0.797215, 0.151499], [0.965221, 0.456860], [0.755156, 0.402490], [0.573567, 0.255689], [0.397194, 0.317920], [0.513037, 0.371109], [0.473372, 0.466375], [0.340141, 0.466375], [0.152681, 0.354088], [-0.125625, 0.399003], [0.053142, 0.551713], [-0.009126, 0.619086], [-0.206692, 0.619086], [-0.420234, 0.381569], [-0.698439, 0.324244], [-0.698435, 0.327139]];

        //  [40.0, 40.0], [-15.0, -15.0],
        widget::Polygon::fill(vertexes).top_left_of(ids.canvas)
            .color(conrod::color::RED)
            .set(ids.line, ui);

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
