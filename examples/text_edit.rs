#[macro_use] extern crate conrod;
extern crate find_folder;

use conrod::backend::piston::{Window, UpdateEvent, OpenGL, EventWindow};
use conrod::backend::piston::core_events::{EventLoop, WindowEvents};
use conrod::backend::piston::window as piston_window;

widget_ids! { 
    struct Ids { canvas, text_edit }
}

fn main() {
    const WIDTH: u32 = 360;
    const HEIGHT: u32 = 720;

    // Construct the window.
    let mut window: Window =
        piston_window::WindowSettings::new("Text Demo", [WIDTH, HEIGHT])
            .opengl(OpenGL::V3_2).exit_on_esc(true).build().unwrap();

    // Create the event loop.
    let mut events = WindowEvents::new();
    events.set_ups(60);

    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new().build();

    // A unique identifier for each widget.
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache = piston_window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    // Some starting text to edit.
    let mut demo_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
        Mauris aliquet porttitor tellus vel euismod. Integer lobortis volutpat bibendum. Nulla \
        finibus odio nec elit condimentum, rhoncus fermentum purus lacinia. Interdum et malesuada \
        fames ac ante ipsum primis in faucibus. Cras rhoncus nisi nec dolor bibendum pellentesque. \
        Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. \
        Quisque commodo nibh hendrerit nunc sollicitudin sodales. Cras vitae tempus ipsum. Nam \
        magna est, efficitur suscipit dolor eu, consectetur consectetur urna.".to_owned();

    // Poll events from the window.
    while let Some(event) = window.next(&mut events) {

        // Convert the piston event to a conrod event.
        if let Some(e) = piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| set_ui(ui.set_widgets(), &ids, &mut demo_text));

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T { img };
                piston_window::draw(c, g, primitives,
                                    &mut text_texture_cache,
                                    &image_map,
                                    texture_from_image);
            }
        });
    }

}

// Declare the `WidgetId`s and instantiate the widgets.
fn set_ui(ref mut ui: conrod::UiCell, ids: &Ids, demo_text: &mut String) {
    use conrod::{color, widget, Colorable, Positionable, Sizeable, Widget};

    widget::Canvas::new().color(color::DARK_CHARCOAL).set(ids.canvas, ui);

    for edit in widget::TextEdit::new(demo_text)
        .color(color::LIGHT_BLUE)
        .padded_wh_of(ids.canvas, 20.0)
        .mid_top_of(ids.canvas)
        .align_text_x_middle()
        .line_spacing(2.5)
        .set(ids.text_edit, ui)
    {
        *demo_text = edit;
    }
}
