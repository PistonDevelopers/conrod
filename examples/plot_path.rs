#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use piston_window::{EventLoop, PistonWindow, UpdateEvent, WindowSettings};


fn main() {

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("PlotPath Demo", [720, 360])
            .opengl(piston_window::OpenGL::V3_2)
            .samples(4)
            .exit_on_esc(true)
            .build()
            .unwrap();
    window.set_ups(60);

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new().build();

    // No text to draw, so we'll just create an empty text texture cache.
    let mut text_texture_cache = conrod::backend::piston_window::GlyphCache::new(&mut window, 0, 0);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    // Poll events from the window.
    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| ui.set_widgets(set_ui));

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
fn set_ui(ref mut ui: conrod::UiCell) {
    use conrod::{color, Canvas, Colorable, PlotPath, Positionable, Sizeable, Widget};

    widget_ids!{CANVAS, PLOT};

    Canvas::new().color(color::DARK_CHARCOAL).set(CANVAS, ui);

    let min_x = 0.0;
    let max_x = std::f64::consts::PI * 2.0;
    let min_y = -1.0;
    let max_y = 1.0;
    PlotPath::new(min_x, max_x, min_y, max_y, f64::sin)
        .color(color::LIGHT_BLUE)
        .wh_of(CANVAS)
        .middle_of(CANVAS)
        .set(PLOT, ui);
}
