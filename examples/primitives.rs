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

    // construct our `Ui`.
    let mut ui = conrod::Ui::new(conrod::Theme::default());

    // No text to draw, so we'll just create an empty text texture cache.
    let mut text_texture_cache: G2dTexture<'static> =
        G2dTexture::empty(&mut window.factory).unwrap();

    window.set_ups(60);

    // Poll events from the window.
    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        let size = window.size();
        let (w, h) = (size.width as conrod::Scalar, size.height as conrod::Scalar);
        if let Some(e) = conrod::backend::piston::event::convert_event(event.clone(), w, h) {
            ui.handle_event(e);
        }

        // Update the widgets.
        event.update(|_| ui.set_widgets(set_ui));

        // Draw the `Ui`.
        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {

                // Data and functions for rendering the primitives.
                let renderer = conrod::backend::piston::draw::Renderer {
                    context: c,
                    graphics: g,
                    texture_cache: &mut text_texture_cache,
                    // No text to draw.
                    cache_queued_glyphs: |_graphics: &mut piston_window::G2d,
                                          _cache: &mut G2dTexture<'static>,
                                          _rect: conrod::text::rt::Rect<u32>,
                                          _data: &[u8]| {
                        unimplemented!();
                    },
                    // No images to draw.
                    get_texture: |_id| None,
                };

                conrod::backend::piston::draw::primitives(primitives, renderer);
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
