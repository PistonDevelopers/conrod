#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;


fn main() {
    const WIDTH: u32 = 200;
    const HEIGHT: u32 = 200;

    use conrod::{widget, Labelable, Positionable, Sizeable, Widget};
    use piston_window::{EventLoop, OpenGL, PistonWindow, UpdateEvent, WindowSettings};

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;
    
    // Construct the window.
    let mut window: PistonWindow = WindowSettings::new("Click me!", [WIDTH, HEIGHT])
        .opengl(opengl).exit_on_esc(true).build().unwrap();
    window.set_ups(60);

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new().build();

    // Generate the widget identifiers.
    widget_ids!(Ids { canvas, counter });
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache =
        conrod::backend::piston_window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    let mut count = 0;

    // Poll events from the window.
    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        // `Update` the widgets.
        event.update(|_| {
            let ui = &mut ui.set_widgets();

            // Create a background canvas upon which we'll place the button.
            widget::Canvas::new().pad(40.0).set(ids.canvas, ui);

            // Draw the button and increment `count` if pressed.
            for _click in widget::Button::new()
                .middle_of(ids.canvas)
                .w_h(80.0, 80.0)
                .label(&count.to_string())
                .set(ids.counter, ui)
            {
                count += 1;
            }
        });

        // Draw the `Ui` if it has changed.
        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T { img };
                conrod::backend::piston_window::draw(c, g, primitives,
                                                     &mut text_texture_cache,
                                                     &image_map,
                                                     texture_from_image);
            }
        });
    }
}
