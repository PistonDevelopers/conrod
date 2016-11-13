//! A demonstration of using glutin to provide events and GFX to draw the UI

#![allow(unused_variables)]

#[cfg(feature="glutin")]
#[macro_use]
extern crate conrod;
#[cfg(feature="glutin")]
extern crate glutin;
#[macro_use]
extern crate gfx;

#[cfg(feature="glutin")]
mod support;


fn main() {
    feature::main();
}

#[cfg(feature="glutin")]
mod feature {
    extern crate gfx_window_glutin;
    extern crate find_folder;

    use conrod;
    use glutin;
    use gfx;
    use support;

    use gfx::{Factory, Device, tex};
    use gfx::traits::FactoryExt;
    use conrod::render;
    use conrod::text::rt;

    const FRAGMENT_SHADER: &'static [u8] = b"
        #version 140

        uniform sampler2D t_Color;

        in vec2 v_Uv;
        in vec4 v_Color;

        out vec4 f_Color;

        void main() {
            vec4 tex = texture(t_Color, v_Uv);
            f_Color = v_Color * tex;
        }
    ";

    const VERTEX_SHADER: &'static [u8] = b"
        #version 140

        in vec2 a_Pos;
        in vec2 a_Uv;
        in vec4 a_Color;

        out vec2 v_Uv;
        out vec4 v_Color;

        void main() {
            v_Uv = a_Uv;
            v_Color = a_Color;
            gl_Position = vec4(a_Pos, 0.0, 1.0);
        }
    ";

    // Format definitions (must be pub for  gfx_defines to use them)
    pub type ColorFormat = gfx::format::Srgba8;
    type DepthFormat = gfx::format::DepthStencil;
    type SurfaceFormat = gfx::format::R8_G8_B8_A8;
    type FullFormat = (SurfaceFormat, gfx::format::Unorm);

    // Vertex and pipeline declarations
    gfx_defines! {
        vertex Vertex {
            pos: [f32; 2] = "a_Pos",
            uv: [f32; 2] = "a_Uv",
            color: [f32; 4] = "a_Color",
        }

        pipeline pipe {
            vbuf: ::gfx::VertexBuffer<Vertex> = (),
            color: ::gfx::TextureSampler<[f32; 4]> = "t_Color",
            out: ::gfx::BlendTarget<ColorFormat> = ("f_Color", ::gfx::state::MASK_ALL, ::gfx::preset::blend::ALPHA),
        }
    }

    // Convenience constructor
    impl Vertex {
        fn new(pos: [f32; 2], uv: [f32; 2], color: [f32; 4]) -> Vertex {
            Vertex {
                pos: pos,
                uv: uv,
                color: color,
            }
        }
    }

    const WIN_W: u32 = support::WIN_W;
    const WIN_H: u32 = support::WIN_H;
    const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

    // Creates a gfx texture with the given data
    fn create_texture<F, R>(factory: &mut F, width: u32, height: u32, data: &[u8])
        -> (gfx::handle::Texture<R, SurfaceFormat>, gfx::handle::ShaderResourceView<R, [f32; 4]>)

        where R: gfx::Resources, F: gfx::Factory<R>
    {
        let kind = tex::Kind::D2(width as tex::Size, height as tex::Size, tex::AaMode::Single);
        factory.create_texture_const_u8::<ColorFormat>(kind, &[data]).unwrap()
    }

    // Updates a texture with the given data (used for updating the GlyphCache texture)
    fn update_texture<R, C>(encoder: &mut gfx::Encoder<R, C>,
                            texture: &gfx::handle::Texture<R, SurfaceFormat>,
                            offset: [u16; 2],
                            size: [u16; 2],
                            data: &[[u8; 4]])

        where R: gfx::Resources, C: gfx::CommandBuffer<R>
    {
        let info = tex::ImageInfoCommon {
                xoffset: offset[0],
                yoffset: offset[1],
                zoffset: 0,
                width: size[0],
                height: size[1],
                depth: 0,
                format: (),
                mipmap: 0,
        };

        encoder.update_texture::<SurfaceFormat, FullFormat>(texture, None, info, data).unwrap();
    }

    pub fn main() {
        // Builder for window
        let builder = glutin::WindowBuilder::new()
            .with_title("Conrod with GFX and Glutin")
            .with_dimensions(WIN_W, WIN_H);

        // Initialize gfx things
        let (window, mut device, mut factory, main_color, _) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
        let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();


        // Create texture sampler
        let sampler_info = tex::SamplerInfo::new(tex::FilterMethod::Bilinear, tex::WrapMode::Clamp);
        let sampler = factory.create_sampler(sampler_info);

        // Dummy values for initialization
        let vbuf = factory.create_vertex_buffer(&[]);
        let (_, fake_texture) = create_texture(&mut factory, 2, 2, &[0; 4]);

        let mut data = pipe::Data {
            vbuf: vbuf,
            color: (fake_texture.clone(), sampler),
            out: main_color.clone(),
        };

        // Compile GL program
        let pso = factory.create_pipeline_simple(VERTEX_SHADER, FRAGMENT_SHADER, pipe::new()).unwrap();

        // Demonstration app state that we'll control with our conrod GUI.
        let mut app = support::DemoApp::new();

        // Create Ui and Ids of widgets to instantiate
        let mut ui = conrod::UiBuilder::new([WIN_W as f64, WIN_H as f64]).theme(support::theme()).build();
        let ids = support::Ids::new(ui.widget_id_generator());

        // Load font from file
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        // Create glyph cache and its texture
        let (mut glyph_cache, cache_tex, cache_tex_view) = {
            let dpi = window.hidpi_factor();
            let width = (WIN_W as f32 * dpi) as u32;
            let height = (WIN_H as f32 * dpi) as u32;

            const SCALE_TOLERANCE: f32 = 0.1;
            const POSITION_TOLERANCE: f32 = 0.1;

            let cache = conrod::text::GlyphCache::new(width, height,
                                                      SCALE_TOLERANCE,
                                                      POSITION_TOLERANCE);

            let data = vec![0; (width * height * 4) as usize];

            let (texture, texture_view) = create_texture(&mut factory, width, height, &data);
            
            (cache, texture, texture_view)
        };

        // Event loop
        let mut events = window.poll_events();

        'main: loop {
            // If the window is closed, this will be None for one tick, so to avoid panicking with
            // unwrap, instead break the loop
            let (win_w, win_h) = match window.get_inner_size() {
                Some(s) => s,
                None => break 'main,
            };

            let dpi_factor = window.hidpi_factor();

            if let Some(mut primitives) = ui.draw_if_changed() {
                let (screen_width, screen_height) = (win_w as f32 * dpi_factor, win_h as f32 * dpi_factor);
                let mut vertices = Vec::new();

                // Create vertices
                while let Some(render::Primitive { id, kind, scizzor, rect }) = primitives.next() {
                    match kind {
                        render::PrimitiveKind::Rectangle { color } => {
                        },
                        render::PrimitiveKind::Polygon { color, points } => {
                        },
                        render::PrimitiveKind::Lines { color, cap, thickness, points } => {
                        },
                        render::PrimitiveKind::Image { color, source_rect } => {
                        },
                        render::PrimitiveKind::Text { color, text, font_id } => {
                            let positioned_glyphs = text.positioned_glyphs(dpi_factor);

                            // Queue the glyphs to be cached
                            for glyph in positioned_glyphs {
                                glyph_cache.queue_glyph(font_id.index(), glyph.clone());
                            }

                            glyph_cache.cache_queued(|rect, data| {
                                let offset = [rect.min.x as u16, rect.min.y as u16];
                                let size = [rect.width() as u16, rect.height() as u16];

                                let new_data = data.iter().map(|x| [0, 0, 0, *x]).collect::<Vec<_>>();

                                update_texture(&mut encoder, &cache_tex, offset, size, &new_data);
                            }).unwrap();

                            let color = color.to_fsa();
                            let cache_id = font_id.index();
                            let origin = rt::point(0.0, 0.0);

                            // A closure to convert RustType rects to GL rects
                            let to_gl_rect = |screen_rect: rt::Rect<i32>| rt::Rect {
                                min: origin
                                    + (rt::vector(screen_rect.min.x as f32 / screen_width - 0.5,
                                                  1.0 - screen_rect.min.y as f32 / screen_height - 0.5)) * 2.0,
                                max: origin
                                    + (rt::vector(screen_rect.max.x as f32 / screen_width - 0.5,
                                                  1.0 - screen_rect.max.y as f32 / screen_height - 0.5)) * 2.0,
                            };

                            // Create new vertices
                            let extension = positioned_glyphs.into_iter()
                                .filter_map(|g| glyph_cache.rect_for(cache_id, g).ok().unwrap_or(None))
                                .flat_map(|(uv_rect, screen_rect)| {
                                    use std::iter::once;

                                    let gl_rect = to_gl_rect(screen_rect);
                                    let v = |pos, uv| once(Vertex::new(pos, uv, color));

                                    v([gl_rect.min.x, gl_rect.max.y], [uv_rect.min.x, uv_rect.max.y])
                                        .chain(v([gl_rect.min.x, gl_rect.min.y], [uv_rect.min.x, uv_rect.min.y]))
                                        .chain(v([gl_rect.max.x, gl_rect.min.y], [uv_rect.max.x, uv_rect.min.y]))
                                        .chain(v([gl_rect.max.x, gl_rect.min.y], [uv_rect.max.x, uv_rect.min.y]))
                                        .chain(v([gl_rect.max.x, gl_rect.max.y], [uv_rect.max.x, uv_rect.max.y]))
                                        .chain(v([gl_rect.min.x, gl_rect.max.y], [uv_rect.min.x, uv_rect.max.y]))
                                });

                            vertices.extend(extension);
                        },
                        render::PrimitiveKind::Other(_) => {},
                    }
                }

                // Clear the window
                encoder.clear(&main_color, CLEAR_COLOR);

                // Draw the vertices
                data.color.0 = cache_tex_view.clone();
                let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertices, ());
                data.vbuf = vbuf;
                encoder.draw(&slice, &pso, &data);

                // Display the results
                encoder.flush(&mut device);
                window.swap_buffers().unwrap();
                device.cleanup();
            }

            if let Some(event) = events.next() {
                let (w, h) = (win_w as conrod::Scalar, win_h as conrod::Scalar);
                let dpi_factor = dpi_factor as conrod::Scalar;

                // Convert glutin event to conrod event, requires conrod to be built with the `glutin` feature
                if let Some(event) = conrod::backend::glutin::convert(event.clone(), &window) {
                    ui.handle_event(event);
                }

                // Close window if the escape key or the exit button is pressed
                match event {
                    glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                    glutin::Event::Closed =>
                        break 'main,

                    _ => {},
                }
            }

            // Update widgets if any event has happened
            if ui.global_input.events().next().is_some() {
                let mut ui = ui.set_widgets();
                support::gui(&mut ui, &ids, &mut app);
            }
        }
    }
}

#[cfg(not(feature="glutin"))]
mod feature {
    pub fn main() {
        println!("This example requires the `glutin` feature. \
                 Try running `cargo run --release --no-default-features --features=\"glutin\" --example <example_name>`");
    }
}
