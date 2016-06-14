//! A demonstration using glutin to provide events and glium for drawing the Ui.

#[cfg(feature="event_glutin")] #[cfg(feature="draw_glium")] #[macro_use] extern crate conrod;
#[cfg(feature="event_glutin")] #[cfg(feature="draw_glium")] #[macro_use] extern crate glium;

fn main() {
    feature::main();
}

#[cfg(feature="event_glutin")]
#[cfg(feature="draw_glium")]
mod feature {
    extern crate find_folder;
    use conrod;
    use glium;

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
            .with_title("Conrod with glutin & glium!".into())
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

        // construct our `Ui`.
        let mut ui = conrod::Ui::new(conrod::Theme::default());

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        {
            let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
            let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
            ui.fonts.insert_from_file(font_path).unwrap();
        }

        // Build a texture the size of the number of pixels in the window to use as a cache for text
        // glyphs.
        let text_texture_cache = {
            let dpi = display.get_window().unwrap().hidpi_factor();
            let cache_width = (WIN_W as f32 * dpi) as u32;
            let cache_height = (WIN_H as f32 * dpi) as u32;
            let grey_image = glium::texture::RawImage2d {
                data: Cow::Owned(vec![128u8; cache_width as usize * cache_height as usize]),
                width: cache_width,
                height: cache_height,
                format: glium::texture::ClientFormat::U8
            };
            glium::texture::Texture2d::with_format(
                &display,
                grey_image,
                glium::texture::UncompressedFloatFormat::U8,
                glium::texture::MipmapsOption::NoMipmap
            ).unwrap()
        };

        let mut text: String = "FOO!".into();


        // Start the loop:
        //
        // - Render the current state of the `Ui`.
        // - Update the widgets via `Ui::set_widgets`.
        // - Poll the window for available events.
        // - Repeat.
        'main: loop {

            let ((win_w, win_h), dpi) = {
                let window = display.get_window().unwrap();
                (window.get_inner_size_pixels().unwrap(), window.hidpi_factor())
            };

            // Construct a render event for conrod at the beginning of rendering.
            let dt_secs = 0.0;
            ui.handle_event(conrod::event::render(dt_secs, win_w, win_h, dpi as conrod::Scalar));

            // Draw the `Ui`.
            if let Some(mut primitives) = ui.draw_if_changed() {
                use conrod::render;

                // Draw each primitive in order of depth.
                primitives.draw(|render::Primitive { kind, scizzor, rect }| {
                    match kind {

                        render::PrimitiveKind::Rectangle { color } => {
                            // TODO
                        },

                        render::PrimitiveKind::Polygon { color, points } => {
                            // TODO
                        },

                        render::PrimitiveKind::Lines { color, cap, thickness, points } => {
                            // TODO
                        },

                        render::PrimitiveKind::Text { color, glyph_cache, positioned_glyphs, font_id } => {

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

                            // Render glyphs via the texture cache.
                            let cache_id = font_id.index();
                            for g in positioned_glyphs {
                                if let Ok(Some((uv_rect, screen_rect))) = glyph_cache.rect_for(cache_id, g) {
                                    // TODO
                                }
                            }

                        },

                        render::PrimitiveKind::Image { maybe_color, texture_id, source_rect } => {
                            // TODO
                        },
                    }
                });

            }

            // let mut target = display.draw();
            // target.clear_color(1.0, 1.0, 1.0, 0.0);

            // let blend = glium::Blend::alpha_blending();
            // let draw_params = glium::DrawParameters { blend: blend, ..Default::default() };
            // let no_indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
            // target.draw(&vertex_buffer, no_indices, &program, &uniforms, &draw_params).unwrap();

            // target.finish().unwrap();

            // Update all widgets within the `Ui`.
            ui.set_widgets(|ui| set_widgets(ui));

            for event in display.poll_events() {

                // Use the `event_glutin` backend feature to convert the glutin event to a conrod one.
                let (w, h) = (win_w as conrod::Scalar, win_h as conrod::Scalar);
                let dpi = dpi as conrod::Scalar;
                if let Some(event) = conrod::backend::event_glutin::convert(event.clone(), w, h, dpi) {
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
        }
    }


    /// Instantiate the widgets.
    fn set_widgets(ref mut ui: conrod::UiCell) {
        use conrod::{Colorable, Widget};

        widget_ids!{
            CANVAS,
            BUTTON,
        };

        conrod::Canvas::new().color(conrod::color::DARK_CHARCOAL).set(CANVAS, ui);
    }
}

#[cfg(not(feature="event_glutin"))]
#[cfg(not(feature="draw_glium"))]
mod feature {
    pub fn main() {
        println!("This example requires the `event_glutin` and `draw_glium` features. \
                 Try running `cargo run --release --features=\"event_glutin draw_glium\" --example <example_name>`");
    }
}
