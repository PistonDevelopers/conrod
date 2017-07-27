//! A demonstration of using `winit` to provide events and GFX to draw the UI.
//!
//! `winit` is used via the `glutin` crate which also provides an OpenGL context for drawing
//! `conrod::render::Primitives` to the screen.

#![allow(unused_variables)]

#[cfg(feature="winit")] #[macro_use] extern crate conrod;
#[cfg(feature="winit")] extern crate glutin;
#[cfg(feature="winit")] extern crate winit;
#[cfg(feature="gfx_rs")] extern crate gfx;
#[cfg(feature="gfx_rs")] extern crate gfx_core;

#[cfg(feature="winit")]
mod support;


fn main() {
    feature::main();
}

#[cfg(all(feature="winit",feature="gfx_rs"))]
mod feature {
    extern crate gfx_window_glutin;
    extern crate find_folder;

    use conrod;
    use glutin;
    use gfx;
    use support;
    use winit;

    use glutin::GlContext;
    use gfx::Device;


    const WIN_W: u32 = support::WIN_W;
    const WIN_H: u32 = support::WIN_H;
    const CLEAR_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];

    type DepthFormat = gfx::format::DepthStencil;

    pub fn main() {
        // Builder for window
        let builder = glutin::WindowBuilder::new()
            .with_title("Conrod with GFX and Glutin")
            .with_dimensions(WIN_W, WIN_H);

        let context = glutin::ContextBuilder::new()
            .with_multisampling(4);

        let mut events_loop = winit::EventsLoop::new();

        // Initialize gfx things
        let (window, mut device, mut factory, rtv, _) =
            gfx_window_glutin::init::<conrod::backend::gfx::ColorFormat, DepthFormat>(builder, context, &events_loop );
        let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

        let mut renderer = conrod::backend::gfx::Renderer::new(&mut factory, &rtv, window.hidpi_factor()).unwrap();

        // Create Ui and Ids of widgets to instantiate
        let mut ui = conrod::UiBuilder::new([WIN_W as f64, WIN_H as f64]).theme(support::theme()).build();
        let ids = support::Ids::new(ui.widget_id_generator());

        // Load font from file
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        // FIXME: We don't yet load the rust logo, so just insert nothing for now so we can get an
        // identifier used to construct the DemoApp. This should be changed to *actually* load a
        // gfx texture for the rust logo and insert it into the map.
        let mut image_map = conrod::image::Map::new();
        let rust_logo = image_map.insert(());

        // Demonstration app state that we'll control with our conrod GUI.
        let mut app = support::DemoApp::new(rust_logo);

        'main: loop {
            // If the window is closed, this will be None for one tick, so to avoid panicking with
            // unwrap, instead break the loop
            let (win_w, win_h) = match window.get_inner_size() {
                Some(s) => s,
                None => break 'main,
            };

            let dpi_factor = window.hidpi_factor();

            if let Some(primitives) = ui.draw_if_changed() {
                let dims = (win_w as f32 * dpi_factor, win_h as f32 * dpi_factor);

                //Clear the window
                encoder.clear(&rtv, CLEAR_COLOR);

                renderer.draw(&mut factory,&mut encoder,&mut device,primitives,dims);

                encoder.flush(&mut device);
                window.swap_buffers().unwrap();
                device.cleanup();
            }

            let mut should_quit = false;
            events_loop.poll_events(|event|{
                let (w, h) = (win_w as conrod::Scalar, win_h as conrod::Scalar);
                let dpi_factor = dpi_factor as conrod::Scalar;

                // Convert winit event to conrod event, requires conrod to be built with the `winit` feature
                if let Some(event) = conrod::backend::winit::convert_event(event.clone(), window.window()) {
                    ui.handle_event(event);
                }

                // Close window if the escape key or the exit button is pressed
                match event {
                    winit::Event::WindowEvent{event: winit::WindowEvent::KeyboardInput{input: winit::KeyboardInput{virtual_keycode: Some(winit::VirtualKeyCode::Escape),..}, ..}, .. } |
                    winit::Event::WindowEvent{event: winit::WindowEvent::Closed, ..} =>
                        should_quit = true,
                    _ => {},
                }
            });
            if should_quit {
                break 'main;
            }

            // Update widgets if any event has happened
            if ui.global_input().events().next().is_some() {
                let mut ui = ui.set_widgets();
                support::gui(&mut ui, &ids, &mut app);
            }
        }
    }
}

#[cfg(not(all(feature="winit",feature="gfx_rs")))]
mod feature {
    pub fn main() {
        println!("This example requires the `winit` feature and the `gfx_rs` feature. \
                 Try running `cargo run --release --no-default-features --features=\"winit gf_rs\" --example <example_name>`");
   }
}
