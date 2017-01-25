//! A simple example demonstrating the `List` widget.

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
        struct Ids { canvas, list }
    }

    pub fn main() {
        const WIDTH: u32 = 150;
        const HEIGHT: u32 = 600;

        // Build the window.
        let display = glium::glutin::WindowBuilder::new()
            .with_vsync()
            .with_dimensions(WIDTH, HEIGHT)
            .with_title("List Demo")
            .build_glium()
            .unwrap();

        // Construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

        // Unique identifier for each widget.
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

        let mut list = vec![true; 16];

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

            set_ui(ui.set_widgets(), &mut list, &ids);

            // TODO: Remove this once the following PR lands and is published
            // https://github.com/tomaka/winit/pull/118
            if let Some(resize) = support::check_for_window_resize(&ui, &display) {
                ui.handle_event(resize);
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

    // Declare the `WidgetId`s and instantiate the widgets.
    fn set_ui(ref mut ui: conrod::UiCell, list: &mut [bool], ids: &Ids) {
        use conrod::{widget, Colorable, Labelable, Positionable, Sizeable, Widget};

        widget::Canvas::new().color(conrod::color::DARK_CHARCOAL).set(ids.canvas, ui);

        const ITEM_HEIGHT: conrod::Scalar = 50.0;

        let (mut items, scrollbar) = widget::List::new(list.len(), ITEM_HEIGHT)
            .scrollbar_on_top()
            .middle_of(ids.canvas)
            .wh_of(ids.canvas)
            .set(ids.list, ui);

        while let Some(item) = items.next(ui) {
            let i = item.i;
            let label = format!("item {}: {}", i, list[i]);
            let toggle = widget::Toggle::new(list[i])
                .label(&label)
                .label_color(conrod::color::WHITE)
                .color(conrod::color::LIGHT_BLUE);
            for v in item.set(toggle, ui) {
                list[i] = v;
            }
        }

        if let Some(s) = scrollbar { s.set(ui) }
    }
}

#[cfg(not(all(feature="glutin", feature="glium")))]
mod feature {
    pub fn main() {
        println!("This example requires the `glutin` and `glium` features. \
                 Try running `cargo run --release --features=\"glutin glium\" --example <example_name>`");
    }
}
