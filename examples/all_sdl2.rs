//! An example demonstrating all widgets in a long, vertically scrollable window.

#[cfg(feature="sdl2")] #[macro_use] extern crate conrod;
#[cfg(feature="sdl2")] mod support;

fn main() {
    feature::main();
}

#[cfg(feature="sdl2")]
mod feature {
    extern crate find_folder;
    extern crate sdl2;
    extern crate image;
    use conrod;
    use support;

    use self::sdl2::video::{Window, WindowBuilder};
    use self::sdl2::render::{BlendMode, Canvas, Texture};
    use self::sdl2::event::Event;
    use self::sdl2::surface::Surface;
    use self::sdl2::pixels::PixelFormatEnum;


    pub fn main() {
        const WIDTH: u32 = support::WIN_W;
        const HEIGHT: u32 = support::WIN_H;

        // Initialize the SDL2 subsystems we need
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        // Construct the window and canvas.
        let mut canvas: Canvas<Window> = WindowBuilder::new(&video, "All Widgets - SDL2 Backend", WIDTH, HEIGHT)
            .build()
            .unwrap()
            .into_canvas()
            .present_vsync()
            .build()
            .unwrap();

        let texture_creator = canvas.texture_creator();

        // construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64])
            .theme(support::theme())
            .build();

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        // Create a texture to use for efficiently caching text on the GPU.
        let mut text_cache_pixels = Vec::new();
        let (mut glyph_cache, mut text_texture_cache) = {
            const SCALE_TOLERANCE: f32 = 0.1;
            const POSITION_TOLERANCE: f32 = 0.1;
            const TEXTURE_WIDTH: u32 = 512;
            const TEXTURE_HEIGHT: u32 = 512;

            let cache = conrod::text::GlyphCache::new(TEXTURE_WIDTH, TEXTURE_HEIGHT, SCALE_TOLERANCE, POSITION_TOLERANCE);

            // The format uses the reversed channel order since we're assuming little endian
            let mut texture = texture_creator.create_texture_static(Some(PixelFormatEnum::ABGR8888), TEXTURE_WIDTH, TEXTURE_HEIGHT).unwrap();
            texture.set_blend_mode(BlendMode::Blend);   // FIXME not sure if add or blend is right
            // FIXME: Also, SDL claims that the blend mode is set automatically
            (cache, texture)
        };

        // Instantiate the generated list of widget identifiers.
        let ids = support::Ids::new(ui.widget_id_generator());

        // Load the rust logo from file to a piston_window texture.
        let rust_logo: Texture = {
            let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
            let path = assets.join("images/rust.png");

            // Use the `image` crate to load the file. Could also use `sdl2::image` if enabled
            // (which uses the `SDL_image` library internally).

            let rgba_image = image::open(&path).unwrap().to_rgba();
            let (w, h) = rgba_image.dimensions();
            let mut raw_data = rgba_image.into_raw();

            // The format uses the reversed channel order since we're assuming little endian
            let surface = Surface::from_data(&mut raw_data, w, h, w * 4, PixelFormatEnum::ABGR8888).unwrap();
            texture_creator.create_texture_from_surface(surface).unwrap()
        };

        // Create our `conrod::image::Map` which describes each of our widget->image mappings.
        // In our case we only have one image, however the macro may be used to list multiple.
        let mut image_map = conrod::image::Map::new();
        let rust_logo = image_map.insert(rust_logo);

        // A demonstration of some state that we'd like to control with the App.
        let mut app = support::DemoApp::new(rust_logo);

        // Poll events from SDL's event pump
        let mut event_pump = sdl.event_pump().unwrap();
        for event in event_pump.wait_iter() {
            if let Event::Quit { .. } = event {
                println!("Received quit event, exiting!");
                break;
            }

            // Convert the SDL2 event to a conrod event.
            let viewport = canvas.viewport();
            let (win_w, win_h) = (viewport.width() as conrod::Scalar, viewport.height() as conrod::Scalar);
            if let Some(e) = conrod::backend::sdl2::event::convert(event.clone(), win_w, win_h) {
                ui.handle_event(e);
            }

            {
                let mut ui = ui.set_widgets();
                support::gui(&mut ui, &ids, &mut app);
            }

            if let Some(primitives) = ui.draw_if_changed() {

                // A function used for caching glyphs to the texture cache.
                let cache_queued_glyphs = |_: &mut Canvas<_>,
                                           cache: &mut Texture,
                                           rect: conrod::text::rt::Rect<u32>,
                                           data: &[u8]|
                    {
                        text_cache_pixels.clear();
                        text_cache_pixels.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));

                        let sdl_rect = (rect.min.x as i32, rect.min.y as i32, rect.width(), rect.height()).into();
                        let pitch = 4 * rect.width() as usize;   // Bytes per row
                        cache.update(Some(sdl_rect), &text_cache_pixels, pitch).unwrap();
                    };

                // Draw the conrod `render::Primitives`.
                canvas.clear();
                conrod::backend::sdl2::draw::primitives(primitives,
                                                        &mut canvas,
                                                        &mut text_texture_cache,
                                                        &mut glyph_cache,
                                                        cache_queued_glyphs,
                                                        &mut image_map);
                canvas.present();
            }
        }
    }
}

#[cfg(not(feature="sdl2"))]
mod feature {
    pub fn main() {
        println!("This example requires the `sdl2` feature. \
                 Try running `cargo run --release --features=\"sdl2\" --example <example_name>`");
    }
}
