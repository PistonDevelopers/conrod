#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use piston_window::*;


fn main() {

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("Primitives Demo", [400, 720])
            .opengl(opengl).samples(4).exit_on_esc(true).build().unwrap();
    window.set_ups(60);

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new()
        .build()
        .unwrap();

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

        // Update the widgets.
        event.update(|_| ui.set_widgets(set_ui));

        // Draw the `Ui`.
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


fn set_ui(ref mut ui: conrod::UiCell) {
    use conrod::{Canvas, Circle, Line, Oval, PointPath, Polygon, Positionable, Rectangle, Widget};
    use std::iter::once;

    // Generate a unique const `WidgetId` for each widget.
    widget_ids!{
        CANVAS,
        LINE,
        POINT_PATH,
        RECTANGLE_FILL,
        RECTANGLE_OUTLINE,
        TRAPEZOID,
        OVAL_FILL,
        OVAL_OUTLINE,
        CIRCLE,
    };

    // The background canvas upon which we'll place our widgets.
    Canvas::new().pad(80.0).set(CANVAS, ui);

    Line::centred([-40.0, -40.0], [40.0, 40.0]).top_left_of(CANVAS).set(LINE, ui);

    let left = [-40.0, -40.0];
    let top = [0.0, 40.0];
    let right = [40.0, -40.0];
    let points = once(left).chain(once(top)).chain(once(right));
    PointPath::centred(points).down(80.0).set(POINT_PATH, ui);

    Rectangle::fill([80.0, 80.0]).down(80.0).set(RECTANGLE_FILL, ui);

    Rectangle::outline([80.0, 80.0]).down(80.0).set(RECTANGLE_OUTLINE, ui);

    let bl = [-40.0, -40.0];
    let tl = [-20.0, 40.0];
    let tr = [20.0, 40.0];
    let br = [40.0, -40.0];
    let points = once(bl).chain(once(tl)).chain(once(tr)).chain(once(br));
    Polygon::centred_fill(points).right_from(LINE, 80.0).set(TRAPEZOID, ui);

    Oval::fill([40.0, 80.0]).down(80.0).align_middle_x().set(OVAL_FILL, ui);

    Oval::outline([80.0, 40.0]).down(100.0).align_middle_x().set(OVAL_OUTLINE, ui);

    Circle::fill(40.0).down(100.0).align_middle_x().set(CIRCLE, ui);
}
