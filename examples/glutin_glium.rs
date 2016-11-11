//! A demonstration using glutin to provide events and glium for drawing the Ui.
//!
//! Note that the `glium` crate is re-exported via the `conrod::backend::glium` module.

#[cfg(feature="glutin")] #[cfg(feature="glium")] #[macro_use] extern crate conrod;

#[cfg(feature="glutin")] #[cfg(feature="glium")] mod support;

fn main() {
    feature::main();
}

#[cfg(feature="glutin")]
#[cfg(feature="glium")]
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

        // A demonstration of some app state that we want to control with the conrod GUI.
        let mut app = support::DemoApp::new();

        // Construct our `Ui`.
        let mut ui = conrod::UiBuilder::new().theme(support::theme()).build();

        // The `widget::Id` of each widget instantiated in `support::gui`.
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

        // Start the loop:
        //
        // - Render the current state of the `Ui`.
        // - Update the widgets via the `support::gui` fn.
        // - Poll the window for available events.
        // - Repeat.
        'main: loop {

            // Construct a render event for conrod at the beginning of rendering.
            // NOTE: This will be removed in a future version of conrod as Render events shouldn't
            // be necessary.
            let window = display.get_window().unwrap();
            ui.handle_event(conrod::backend::glutin::render_event(&window).unwrap());

            // Poll for events.
            for event in display.poll_events() {

                // Use the `glutin` backend feature to convert the glutin event to a conrod one.
                if let Some(event) = conrod::backend::glutin::convert(event.clone(), &window) {
                    ui.handle_event(event);
                }

                match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::Escape)) |
                    glium::glutin::Event::Closed =>
                        break 'main,

                    _ => {},
                }
            }

            // We must manually track the window width and height as it is currently not possible to
            // receive `Resize` events from glium on Mac OS any other way.
            //
            // TODO: Once the following PR lands, we should stop tracking size like this and use the
            // `window_resize_callback`. https://github.com/tomaka/winit/pull/88
            if let Some(win_rect) = ui.rect_of(ui.window) {
                let (win_w, win_h) = (win_rect.w() as u32, win_rect.h() as u32);
                let (w, h) = window.get_inner_size_points().unwrap();
                if w != win_w || h != win_h {
                    let event: conrod::event::Raw = conrod::event::Input::Resize(w, h).into();
                    ui.handle_event(event);
                }
            }

            // If some input event has been received, update the GUI.
            if ui.global_input.events().next().is_some() {
                // Instantiate a GUI demonstrating every widget type provided by conrod.
                let mut ui = ui.set_widgets();
                support::gui(&mut ui, &ids, &mut app);
            }

            // Draw the `Ui`.
            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
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
