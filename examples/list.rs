//! A simple example demonstrating the `List` widget.

#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use piston_window::{EventLoop, OpenGL, PistonWindow, UpdateEvent, WindowSettings};

fn main() {

    // Construct the window.
    const WIDTH: u32 = 150;
    const HEIGHT: u32 = 600;
    let mut window: PistonWindow =
        WindowSettings::new("List Demo", [WIDTH, HEIGHT])
            .opengl(OpenGL::V3_2).exit_on_esc(true).samples(4).vsync(true).build().unwrap();
    window.set_ups(60);

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new().build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // No text to draw, so we'll just create an empty text texture cache.
    let mut text_texture_cache =
        conrod::backend::piston_window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    let mut list = vec![true; 16];

    // Poll events from the window.
    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            ui.set_widgets(|ui_cell| set_ui(ui_cell, &mut list))
        });

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed(&image_map) {
                fn texture_from_image<T>(img: &T) -> &T { img };
                conrod::backend::piston_window::draw(c, g, primitives,
                                                     &mut text_texture_cache,
                                                     texture_from_image);
            }
        });
    }

}

// Declare the `WidgetId`s and instantiate the widgets.
fn set_ui(ref mut ui: conrod::UiCell, list: &mut [bool]) {
    use conrod::{Colorable, Labelable, Positionable, Sizeable, Widget};

    widget_ids!{CANVAS, LIST};

    conrod::Canvas::new().color(conrod::color::DARK_CHARCOAL).set(CANVAS, ui);

    const ITEM_HEIGHT: conrod::Scalar = 50.0;

    conrod::List::new(list.len() as u32, ITEM_HEIGHT)
        .scrollbar_on_top()
        .middle_of(CANVAS)
        .wh_of(CANVAS)
        .item(|item| {
            let i = item.i;
            let label = format!("item {}: {}", i, list[i]);
            let toggle = conrod::Toggle::new(list[i])
                .label(&label)
                .label_color(conrod::color::WHITE)
                .color(conrod::color::LIGHT_BLUE)
                .react(|v| list[i] = v);
            item.set(toggle);
        })
        .set(LIST, ui);
}
