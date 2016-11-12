#[macro_use] extern crate conrod;
extern crate find_folder;

use conrod::backend::piston::{self, Window, WindowEvents, OpenGL};
use conrod::backend::piston::event::UpdateEvent;

// Generate a type that will produce a unique `widget::Id` for each widget.
widget_ids! {
    struct Ids {
        canvas,
        line,
        point_path,
        rectangle_fill,
        rectangle_outline,
        trapezoid,
        oval_fill,
        oval_outline,
        circle,
    }
}


fn main() {
    const WIDTH: u32 = 400;
    const HEIGHT: u32 = 720;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Construct the window.
    let mut window: Window =
        piston::window::WindowSettings::new("Primitives Demo", [WIDTH, HEIGHT])
            .opengl(opengl).samples(4).exit_on_esc(true).build().unwrap();

    // Create the event loop.
    let mut events = WindowEvents::new();

    // construct our `Ui`.
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

        // Update the widgets.
        event.update(|_| set_ui(ui.set_widgets(), &ids));

        // Draw the `Ui`.
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


fn set_ui(ref mut ui: conrod::UiCell, ids: &Ids) {
    use conrod::{Positionable, Widget};
    use conrod::widget::{Canvas, Circle, Line, Oval, PointPath, Polygon, Rectangle};
    use std::iter::once;

    // The background canvas upon which we'll place our widgets.
    Canvas::new().pad(80.0).set(ids.canvas, ui);

    Line::centred([-40.0, -40.0], [40.0, 40.0]).top_left_of(ids.canvas).set(ids.line, ui);

    let left = [-40.0, -40.0];
    let top = [0.0, 40.0];
    let right = [40.0, -40.0];
    let points = once(left).chain(once(top)).chain(once(right));
    PointPath::centred(points).down(80.0).set(ids.point_path, ui);

    Rectangle::fill([80.0, 80.0]).down(80.0).set(ids.rectangle_fill, ui);

    Rectangle::outline([80.0, 80.0]).down(80.0).set(ids.rectangle_outline, ui);

    let bl = [-40.0, -40.0];
    let tl = [-20.0, 40.0];
    let tr = [20.0, 40.0];
    let br = [40.0, -40.0];
    let points = once(bl).chain(once(tl)).chain(once(tr)).chain(once(br));
    Polygon::centred_fill(points).right_from(ids.line, 80.0).set(ids.trapezoid, ui);

    Oval::fill([40.0, 80.0]).down(80.0).align_middle_x().set(ids.oval_fill, ui);

    Oval::outline([80.0, 40.0]).down(100.0).align_middle_x().set(ids.oval_outline, ui);

    Circle::fill(40.0).down(100.0).align_middle_x().set(ids.circle, ui);
}
