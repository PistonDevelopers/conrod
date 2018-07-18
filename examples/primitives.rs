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
    use support;

    // Generate a type that will produce a unique `widget::Id` for each widget.
    widget_ids! {
        struct Ids {
            canvas,
            line,
            point_path,
            rectangle_fill,
            rectangle_outline,
            trapezoid,
            oval_fill,
            oval_outline,
            circle,
        }
    }

    pub fn main() {
        const WIDTH: u32 = 400;
        const HEIGHT: u32 = 720;

        // Build the window.
        let mut events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_title("Primitive Widgets Demo")
            .with_dimensions((WIDTH, HEIGHT).into());
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        // construct our `Ui`.
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
                    event_loop.needs_update();
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

            set_ui(ui.set_widgets(), &ids);

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


    fn set_ui(ref mut ui: conrod::UiCell, ids: &Ids) {
        use conrod::{Positionable, Widget};
        use conrod::widget::{Canvas, Circle, Line, Oval, PointPath, Polygon, Rectangle};
        use std::iter::once;

        // The background canvas upon which we'll place our widgets.
        Canvas::new().pad(80.0).set(ids.canvas, ui);

        Line::centred([-40.0, -40.0], [40.0, 40.0]).top_left_of(ids.canvas).set(ids.line, ui);

        let left = [-40.0, -40.0];
        let top = [0.0, 40.0];
        let right = [40.0, -40.0];
        let points = once(left).chain(once(top)).chain(once(right));
        PointPath::centred(points).down(80.0).set(ids.point_path, ui);

        Rectangle::fill([80.0, 80.0]).down(80.0).set(ids.rectangle_fill, ui);

        Rectangle::outline([80.0, 80.0]).down(80.0).set(ids.rectangle_outline, ui);

        let bl = [-40.0, -40.0];
        let tl = [-20.0, 40.0];
        let tr = [20.0, 40.0];
        let br = [40.0, -40.0];
        let points = once(bl).chain(once(tl)).chain(once(tr)).chain(once(br));
        Polygon::centred_fill(points).right_from(ids.line, 80.0).set(ids.trapezoid, ui);

        Oval::fill([40.0, 80.0]).down(80.0).align_middle_x().set(ids.oval_fill, ui);

        Oval::outline([80.0, 40.0]).down(100.0).align_middle_x().set(ids.oval_outline, ui);

        Circle::fill(40.0).down(100.0).align_middle_x().set(ids.circle, ui);
    }
}

#[cfg(not(all(feature="winit", feature="glium")))]
mod feature {
    pub fn main() {
        println!("This example requires the `winit` and `glium` features. \
                 Try running `cargo run --release --features=\"winit glium\" --example <example_name>`");
    }
}
