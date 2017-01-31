extern crate find_folder;
#[cfg(all(feature="winit", feature="glium"))] #[macro_use] extern crate conrod;
#[cfg(all(feature="winit", feature="glium"))] mod support;


#[cfg(all(feature="winit", feature="glium"))]
fn main() {
    use conrod::backend::glium::glium;
    use conrod::backend::glium::glium::{DisplayBuild, Surface};

    const WIDTH: u32 = 600;
    const HEIGHT: u32 = 300;

    // Build the window.
    let display = glium::glutin::WindowBuilder::new()
        .with_vsync()
        .with_dimensions(WIDTH, HEIGHT)
        .with_title("FileNavigator Demo")
        .build_glium()
        .unwrap();

    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // A unique identifier for each widget.
    widget_ids!(struct Ids { canvas, file_navigator });
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

    let directory = find_folder::Search::KidsThenParents(3, 5).for_folder("conrod").unwrap();

    // Poll events from the window.
    let mut event_loop = support::EventLoop::new();
    'main: loop {

        // Handle all events.
        for event in event_loop.next(&display) {

            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = conrod::backend::winit::convert(event.clone(), &display) {
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

        // Instantiate the conrod widgets.
        {
            use conrod::{widget, Colorable, Positionable, Sizeable, Widget};
            let ui = &mut ui.set_widgets();

            widget::Canvas::new().color(conrod::color::DARK_CHARCOAL).set(ids.canvas, ui);

            // Navigate the conrod directory only showing `.rs` and `.toml` files.
            for event in widget::FileNavigator::with_extension(&directory, &["rs", "toml"])
                .color(conrod::color::LIGHT_BLUE)
                .font_size(16)
                .wh_of(ids.canvas)
                .middle_of(ids.canvas)
                //.show_hidden_files(true)  // Use this to show hidden files
                .set(ids.file_navigator, ui)
            {
                println!("{:?}", event);
            }
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

#[cfg(not(all(feature="winit", feature="glium")))]
fn main() {
    println!("This example requires the `winit` and `glium` features. \
             Try running `cargo run --release --features=\"winit glium\" --example <example_name>`");
}
