#[macro_use] extern crate conrod;
extern crate find_folder;

use conrod::backend::piston::{self, Window, WindowEvents, OpenGL};
use conrod::backend::piston::event::UpdateEvent;

widget_ids! {
    struct Ids { canvas, plot }
}

fn main() {
    const WIDTH: u32 = 720;
    const HEIGHT: u32 = 360;

    // Construct the window.
    let mut window: Window =
        piston::window::WindowSettings::new("PlotPath Demo", [WIDTH, HEIGHT])
            .opengl(OpenGL::V3_2)
            .samples(4)
            .exit_on_esc(true)
            .build()
            .unwrap();

    // Create the event loop.
    let mut events = WindowEvents::new();

    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // A unique identifier for each widget.
    let ids = Ids::new(ui.widget_id_generator());

    // No text to draw, so we'll just create an empty text texture cache.
    let mut text_texture_cache = piston::window::GlyphCache::new(&mut window, 0, 0);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    // Poll events from the window.
    while let Some(event) = window.next_event(&mut events) {

        // Convert the piston event to a conrod event.
        if let Some(e) = piston::window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            use conrod::{color, widget, Colorable, Positionable, Sizeable, Widget};

            let ui = &mut ui.set_widgets();

            widget::Canvas::new().color(color::DARK_CHARCOAL).set(ids.canvas, ui);

            let min_x = 0.0;
            let max_x = std::f64::consts::PI * 2.0;
            let min_y = -1.0;
            let max_y = 1.0;
            widget::PlotPath::new(min_x, max_x, min_y, max_y, f64::sin)
                .color(color::LIGHT_BLUE)
                .wh_of(ids.canvas)
                .middle_of(ids.canvas)
                .set(ids.plot, ui);
        });

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T { img };
                piston::window::draw(c, g, primitives,
                                     &mut text_texture_cache,
                                     &image_map,
                                     texture_from_image);
            }
        });
    }
}
