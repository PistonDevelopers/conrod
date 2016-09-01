//! A simple example demonstrating the `List` widget.

#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use piston_window::{EventLoop, OpenGL, PistonWindow, UpdateEvent, WindowSettings};

widget_ids! {
    Ids { canvas, list }
}

fn main() {

    // Construct the window.
    const WIDTH: u32 = 150;
    const HEIGHT: u32 = 600;
    let mut window: PistonWindow =
        WindowSettings::new("List Demo", [WIDTH, HEIGHT])
            .opengl(OpenGL::V3_2).exit_on_esc(true).samples(4).vsync(true).build().unwrap();
    window.set_ups(60);

    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new().build();

    // Unique identifier for each widget.
    let ids = Ids::new();

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
            set_ui(ui.set_widgets(), &mut list, &ids);
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

// Declare the `WidgetId`s and instantiate the widgets.
fn set_ui(ref mut ui: conrod::UiCell, list: &mut [bool], ids: &Ids) {
    use conrod::{widget, Colorable, Labelable, Positionable, Sizeable, Widget};

    let canvas = ids.canvas.get(ui);
    widget::Canvas::new().color(conrod::color::DARK_CHARCOAL).set(canvas, ui);

    const ITEM_HEIGHT: conrod::Scalar = 50.0;
    let num_items = list.len();

    let (mut items, scrollbar) = widget::List::new(num_items, ITEM_HEIGHT)
        .scrollbar_on_top()
        .middle_of(canvas)
        .wh_of(canvas)
        .set(ids.list.get(ui), ui);

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

    scrollbar.unwrap().set(ui);
}
