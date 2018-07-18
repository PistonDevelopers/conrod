#[cfg(all(feature="winit", feature="glium"))] #[macro_use] extern crate conrod;
#[cfg(all(feature="winit", feature="glium"))] mod support;

fn main() {
    feature::main();
}

#[cfg(all(feature="winit", feature="glium"))]
mod feature {
    extern crate find_folder;
    use conrod;
    use conrod::backend::glium::glium;
    use conrod::backend::glium::glium::Surface;
    use std;
    use support;

    widget_ids! {
        struct Ids { canvas, grid, plot }
    }

    pub fn main() {
        const WIDTH: u32 = 720;
        const HEIGHT: u32 = 360;

        // Build the window.
        let mut events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_title("PlotPath Demo")
            .with_dimensions((WIDTH, HEIGHT).into());
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        // Construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

        // A unique identifier for each widget.
        let ids = Ids::new(ui.widget_id_generator());

        // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
        // for drawing to the glium `Surface`.
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

        // The image map describing each of our widget->image mappings (in our case, none).
        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        // Poll events from the window.
        let mut event_loop = support::EventLoop::new();
        'main: loop {

            // Handle all events.
            for event in event_loop.next(&mut events_loop) {

                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                    ui.handle_event(event);
                }

                match event {
                    glium::glutin::Event::WindowEvent { event, .. } => match event {
                        // Break from the loop upon `Escape`.
                        glium::glutin::WindowEvent::CloseRequested |
                        glium::glutin::WindowEvent::KeyboardInput {
                            input: glium::glutin::KeyboardInput {
                                virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => break 'main,
                        _ => (),
                    },
                    _ => (),
                }
            }

            // Instantiate the widgets.
            {
                use conrod::{color, widget, Colorable, Positionable, Sizeable, Widget};

                let ui = &mut ui.set_widgets();

                widget::Canvas::new().color(color::DARK_CHARCOAL).set(ids.canvas, ui);

                let min_x = 0.0;
                let max_x = std::f64::consts::PI * 2.0;
                let min_y = -1.0;
                let max_y = 1.0;

                let quarter_lines = widget::grid::Lines::step(0.5_f64).thickness(2.0);
                let sixteenth_lines = widget::grid::Lines::step(0.125_f64).thickness(1.0);
                let lines = &[
                    quarter_lines.x(),
                    quarter_lines.y(),
                    sixteenth_lines.x(),
                    sixteenth_lines.y(),
                ];

                widget::Grid::new(min_x, max_x, min_y, max_y, lines.iter().cloned())
                    .color(color::rgb(0.1, 0.12, 0.15))
                    .wh_of(ids.canvas)
                    .middle_of(ids.canvas)
                    .set(ids.grid, ui);
                widget::PlotPath::new(min_x, max_x, min_y, max_y, f64::sin)
                    .color(color::LIGHT_BLUE)
                    .thickness(2.0)
                    .wh_of(ids.canvas)
                    .middle_of(ids.canvas)
                    .set(ids.plot, ui);
            }

            // Render the `Ui` and then display it on the screen.
            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
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
