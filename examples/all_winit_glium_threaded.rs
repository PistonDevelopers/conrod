//! This example behaves the same as the `all_winit_glium` example while demonstrating how to run
//! the `conrod` loop on a separate thread.

#[cfg(all(feature="winit", feature="glium"))] #[macro_use] extern crate conrod;
#[cfg(all(feature="winit", feature="glium"))] mod support;

fn main() {
    feature::main();
}

#[cfg(all(feature="winit", feature="glium"))]
mod feature {
    extern crate find_folder;
    extern crate image;
    use conrod;
    use conrod::backend::glium::glium;
    use conrod::backend::glium::glium::{DisplayBuild, Surface};
    use support;
    use std;

    // The initial width and height in "points".
    const WIN_W: u32 = support::WIN_W;
    const WIN_H: u32 = support::WIN_H;

    pub fn main() {

        // Build the window.
        let display = glium::glutin::WindowBuilder::new()
            .with_vsync()
            .with_dimensions(WIN_W, WIN_H)
            .with_title("Conrod with glium!")
            .build_glium()
            .unwrap();

        // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
        // for drawing to the glium `Surface`.
        //
        // Internally, the `Renderer` maintains:
        // - a `backend::glium::GlyphCache` for caching text onto a `glium::texture::Texture2d`.
        // - a `glium::Program` to use as the shader program when drawing to the `glium::Surface`.
        // - a `Vec` for collecting `backend::glium::Vertex`s generated when translating the
        // `conrod::render::Primitive`s.
        // - a `Vec` of commands that describe how to draw the vertices.
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

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

        let mut image_map = conrod::image::Map::new();
        let rust_logo = image_map.insert(load_rust_logo(&display));

        // A channel to send events from the main `winit` thread to the conrod thread.
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        // A channel to send `render::Primitive`s from the conrod thread to the `winit thread.
        let (render_tx, render_rx) = std::sync::mpsc::channel();
        // This window proxy will allow conrod to wake up the `winit::Window` for rendering.
        let window_proxy = display.get_window().unwrap().create_window_proxy();

        // A function that runs the conrod loop.
        fn run_conrod(rust_logo: conrod::image::Id,
                      event_rx: std::sync::mpsc::Receiver<conrod::event::Input>,
                      render_tx: std::sync::mpsc::Sender<conrod::render::OwnedPrimitives>,
                      window_proxy: glium::glutin::WindowProxy)
        {
            // Construct our `Ui`.
            let mut ui = conrod::UiBuilder::new([WIN_W as f64, WIN_H as f64]).theme(support::theme()).build();

            // Add a `Font` to the `Ui`'s `font::Map` from file.
            let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
            let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
            ui.fonts.insert_from_file(font_path).unwrap();

            // A demonstration of some app state that we want to control with the conrod GUI.
            let mut app = support::DemoApp::new(rust_logo);

            // The `widget::Id` of each widget instantiated in `support::gui`.
            let ids = support::Ids::new(ui.widget_id_generator());

            // Many widgets require another frame to finish drawing after clicks or hovers, so we
            // insert an update into the conrod loop using this `bool` after each event.
            let mut needs_update = true;
            'conrod: loop {

                // Collect any pending events.
                let mut events = Vec::new();
                while let Ok(event) = event_rx.try_recv() {
                    events.push(event);
                }

                // If there are no events pending, wait for them.
                if events.is_empty() || !needs_update {
                    match event_rx.recv() {
                        Ok(event) => events.push(event),
                        Err(_) => break 'conrod,
                    };
                }

                needs_update = false;

                // Input each event into the `Ui`.
                for event in events {
                    ui.handle_event(event);
                    needs_update = true;
                }

                // Instantiate a GUI demonstrating every widget type provided by conrod.
                support::gui(&mut ui.set_widgets(), &ids, &mut app);

                // Render the `Ui` to a list of primitives that we can send to the main thread for
                // display.
                if let Some(primitives) = ui.draw_if_changed() {
                    if render_tx.send(primitives.owned()).is_err() {
                        break 'conrod;
                    }
                    // Wakeup `winit` for rendering.
                    window_proxy.wakeup_event_loop();
                }
            }
        }

        // Spawn the conrod loop on its own thread.
        std::thread::spawn(move || run_conrod(rust_logo, event_rx, render_tx, window_proxy));

        // Run the `winit` loop.
        let mut last_update = std::time::Instant::now();
        'main: loop {

            // We don't want to loop any faster than 60 FPS, so wait until it has been at least
            // 16ms since the last yield.
            let sixteen_ms = std::time::Duration::from_millis(16);
            let now = std::time::Instant::now();
            let duration_since_last_update = now.duration_since(last_update);
            if duration_since_last_update < sixteen_ms {
                std::thread::sleep(sixteen_ms - duration_since_last_update);
            }

            // Collect all pending events.
            let mut events: Vec<_> = display.poll_events().collect();

            // If there are no events, wait for the next event.
            if events.is_empty() {
                events.extend(display.wait_events().next());
            }

            // Send any relevant events to the conrod thread.
            for event in events {

                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = conrod::backend::winit::convert(event.clone(), &display) {
                    event_tx.send(event).unwrap();
                }

                match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::Escape)) |
                    glium::glutin::Event::Closed =>
                        break 'main,
                    _ => {},
                }
            }

            // Draw the most recently received `conrod::render::Primitives` sent from the `Ui`.
            if let Ok(mut primitives) = render_rx.try_recv() {
                while let Ok(newest) = render_rx.try_recv() {
                    primitives = newest;
                }

                renderer.fill(&display, primitives.walk(), &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }

            last_update = std::time::Instant::now();
        }
    }

}

#[cfg(not(all(feature="winit", feature="glium")))]
mod feature {
    pub fn main() {
        println!("This example requires the `winit` and `glium` features. \
                 Try running `cargo run --release --features=\"winit glium\" --example <example_name>`");
    }
}
