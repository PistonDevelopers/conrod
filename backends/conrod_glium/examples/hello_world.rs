//! A simple example that demonstrates using conrod within a basic `winit` window loop, using
//! `glium` to render the `conrod_core::render::Primitives` to screen.

#[cfg(all(feature = "conrod_winit_glue"))]
#[macro_use] extern crate conrod_core;
extern crate glium;
#[cfg(all(feature = "conrod_winit_glue"))] mod support;


fn main() {
    feature::main();
}

#[cfg(all(feature = "conrod_winit_glue"))]
mod feature {
    extern crate conrod_glium_backend;
    extern crate conrod_winit_backend;

    extern crate find_folder;

    use conrod_core::{self, widget, Colorable, Positionable, Widget};
    use glium::{self, Surface};
    use support;

    pub fn main() {
        const WIDTH: u32 = 400;
        const HEIGHT: u32 = 200;

        // Build the window.
        let mut events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_title("Hello Conrod!")
            .with_dimensions((WIDTH, HEIGHT).into());
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let display = glium::Display::new(window, context, &events_loop).unwrap();
        let display = support::GliumDisplayWinitWrapper(display);

        // construct our `Ui`.
        let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

        // Generate the widget identifiers.
        widget_ids!(struct Ids { text });
        let ids = Ids::new(ui.widget_id_generator());

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        // A type used for converting `conrod_core::render::Primitives` into `Command`s that can be used
        // for drawing to the glium `Surface`.
        let mut renderer = conrod_glium_backend::Renderer::new(&display.0).unwrap();;

        // The image map describing each of our widget->image mappings (in our case, none).
        let image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();

        let mut events = Vec::new();

        'render: loop {
            events.clear();

            // Get all the new events since the last frame.
            events_loop.poll_events(|event| { events.push(event); });

            // If there are no new events, wait for one.
            if events.is_empty() {
                events_loop.run_forever(|event| {
                    events.push(event);
                    glium::glutin::ControlFlow::Break
                });
            }

            // Process the events.
            for event in events.drain(..) {

                // Break from the loop upon `Escape` or closed window.
                match event.clone() {
                    glium::glutin::Event::WindowEvent { event, .. } => {
                        match event {
                            glium::glutin::WindowEvent::CloseRequested |
                            glium::glutin::WindowEvent::KeyboardInput {
                                input: glium::glutin::KeyboardInput {
                                    virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                    ..
                                },
                                ..
                            } => break 'render,
                            _ => (),
                        }
                    }
                    _ => (),
                };

                // Use the `winit` backend feature to convert the winit event to a conrod input.
                let input = match conrod_winit_backend::convert_event(event, &display) {
                    None => continue,
                    Some(input) => input,
                };

                // Handle the input with the `Ui`.
                ui.handle_event(input);

                // Set the widgets.
                let ui = &mut ui.set_widgets();

                // "Hello World!" in the middle of the screen.
                widget::Text::new("Hello World!")
                    .middle_of(ui.window)
                    .color(conrod_core::color::WHITE)
                    .font_size(32)
                    .set(ids.text, ui);
            }

            // Draw the `Ui` if it has changed.
            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display.0, primitives, &image_map);
                let mut target = display.0.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display.0, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    }
}

#[cfg(not(all(feature = "conrod_winit_glue")))]
mod feature {
    pub fn main() {
        println!("This example requires the `conrod_winit_glue` feature. \
                 Try running `cargo run --release --features=\"conrod_winit_glue\" --example <example_name>`");
    }
}
