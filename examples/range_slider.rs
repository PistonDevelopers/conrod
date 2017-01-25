//! A simple example demonstrating the `RangeSlider` widget.

#[cfg(all(feature="glutin", feature="glium"))] #[macro_use] extern crate conrod;
#[cfg(all(feature="glutin", feature="glium"))] mod support;

fn main() {
    feature::main();
}

#[cfg(all(feature="glutin", feature="glium"))]
mod feature {
    extern crate find_folder;
    use conrod;
    use conrod::backend::glium::glium;
    use conrod::backend::glium::glium::{DisplayBuild, Surface};
    use support;

    widget_ids! {
        struct Ids { canvas, oval, range_slider }
    }

    pub fn main() {
        const WIDTH: u32 = 360;
        const HEIGHT: u32 = 360;

        // Build the window.
        let display = glium::glutin::WindowBuilder::new()
            .with_vsync()
            .with_dimensions(WIDTH, HEIGHT)
            .with_title("RangeSlider Demo")
            .build_glium()
            .unwrap();

        // Construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

        // A unique identifier for each widget.
        let ids = Ids::new(ui.widget_id_generator());

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
        // for drawing to the glium `Surface`.
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

        // The image map describing each of our widget->image mappings (in our case, none).
        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        let mut oval_range = (0.25, 0.75);

        // Poll events from the window.
        let mut event_loop = support::EventLoop::new();
        'main: loop {

            // Handle all events.
            for event in event_loop.next(&display) {

                // Use the `glutin` backend feature to convert the glutin event to a conrod one.
                let window = display.get_window().unwrap();
                if let Some(event) = conrod::backend::glutin::convert(event.clone(), window) {
                    ui.handle_event(event);
                    event_loop.needs_update();
                }

                match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::Event::KeyboardInput(_, _, Some(glium::glutin::VirtualKeyCode::Escape)) |
                    glium::glutin::Event::Closed =>
                        break 'main,
                    _ => {},
                }
            }

            // TODO: Remove this once the following PR lands and is published
            // https://github.com/tomaka/winit/pull/118
            if let Some(resize) = support::check_for_window_resize(&ui, &display) {
                ui.handle_event(resize);
            }

            set_ui(ui.set_widgets(), &ids, &mut oval_range);

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

    // Declare the `WidgetId`s and instantiate the widgets.
    fn set_ui(ref mut ui: conrod::UiCell, ids: &Ids, oval_range: &mut (conrod::Scalar, conrod::Scalar)) {
        use conrod::{color, widget, Colorable, Positionable, Sizeable, Widget};

        widget::Canvas::new().color(color::DARK_CHARCOAL).set(ids.canvas, ui);

        const PAD: conrod::Scalar = 20.0;
        let (ref mut start, ref mut end) = *oval_range;
        let min = 0.0;
        let max = 1.0;
        for (edge, value) in widget::RangeSlider::new(*start, *end, min, max)
            .color(color::LIGHT_BLUE)
            .padded_w_of(ids.canvas, PAD)
            .h(30.0)
            .mid_top_with_margin_on(ids.canvas, PAD)
            .set(ids.range_slider, ui)
        {
            match edge {
                widget::range_slider::Edge::Start => *start = value,
                widget::range_slider::Edge::End => *end = value,
            }
        }

        let range_slider_w = ui.w_of(ids.range_slider).unwrap();
        let w = (*end - *start) * range_slider_w;
        let h = 200.0;
        widget::Oval::fill([w, h])
            .mid_left_with_margin_on(ids.canvas, PAD + *start * range_slider_w)
            .color(color::LIGHT_BLUE)
            .down(50.0)
            .set(ids.oval, ui);
    }
}

#[cfg(not(all(feature="glutin", feature="glium")))]
mod feature {
    pub fn main() {
        println!("This example requires the `glutin` and `glium` features. \
                 Try running `cargo run --release --features=\"glutin glium\" --example <example_name>`");
    }
}
