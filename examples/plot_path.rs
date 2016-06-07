#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use conrod::Widget;
use piston_window::{EventLoop, PistonWindow, UpdateEvent, WindowSettings};

/// Conrod is backend agnostic. Here, we define the `piston_window` backend to use for our `Ui`.
type Backend = (piston_window::G2dTexture<'static>, piston_window::Glyphs);
type Ui = conrod::Ui<Backend>;
type UiCell<'a> = conrod::UiCell<'a, Backend>;


fn main() {

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("PlotPath Demo", [720, 360])
            .opengl(piston_window::OpenGL::V3_2)
            .samples(4)
            .exit_on_esc(true)
            .build()
            .unwrap();

    // Construct our `Ui`.
    let mut ui = {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        let theme = conrod::Theme::default();
        let glyph_cache = piston_window::Glyphs::new(&font_path, window.factory.clone()).unwrap();
        Ui::new(glyph_cache, theme)
    };

    window.set_ups(60);

    // Poll events from the window.
    while let Some(event) = window.next() {
        ui.handle_event(event.clone());
        event.update(|_| ui.set_widgets(set_ui));
        window.draw_2d(&event, |c, g| ui.draw_if_changed(c, g));
    }
}

// Declare the `WidgetId`s and instantiate the widgets.
fn set_ui(ref mut ui: UiCell) {
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
