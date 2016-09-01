#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use piston_window::{EventLoop, PistonWindow, UpdateEvent, WindowSettings};


fn main() {
    const WIDTH: u32 = 600;
    const HEIGHT: u32 = 300;

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("FileNavigator Demo", [WIDTH, HEIGHT])
            .opengl(piston_window::OpenGL::V3_2)
            .vsync(true)
            .samples(4)
            .exit_on_esc(true)
            .build()
            .unwrap();
    window.set_ups(60);

    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new().build();

    // A unique identifier for each widget.
    widget_ids!(Ids { canvas, file_navigator });
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

    let directory = find_folder::Search::KidsThenParents(3, 5).for_folder("conrod").unwrap();

    // Poll events from the window.
    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            use conrod::{widget, Colorable, Positionable, Sizeable, Widget};

            // Instantiate the conrod widgets.
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
        });

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
