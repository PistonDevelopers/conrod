
#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use conrod::{Theme, Widget};
use piston_window::*;

type Ui = conrod::Ui<Glyphs>;


fn main() {

    // Construct the window.
    let window: PistonWindow =
        WindowSettings::new("Primitives Demo", [400, 720])
            .exit_on_esc(true).build().unwrap();

    // construct our `Ui`.
    let mut ui = {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        let theme = Theme::default();
        let glyph_cache = Glyphs::new(&font_path, window.factory.borrow().clone());
        Ui::new(glyph_cache.unwrap(), theme)
    };

    // Poll events from the window.
    for event in window {
        ui.handle_event(&event);
        event.update(|_| ui.set_widgets(set_ui));
        event.draw_2d(|c, g| ui.draw_if_changed(c, g));
    }

}


fn set_ui(ui: &mut Ui) {
    use conrod::{Circle, Line, Oval, PointPath, Polygon, Positionable, Rectangle, Split};
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
    Split::new(CANVAS).pad(80.0).set(ui);

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

