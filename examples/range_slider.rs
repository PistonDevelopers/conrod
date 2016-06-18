//! A simple example demonstrating the `RangeSlider` widget.

#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use conrod::{Theme, Widget};
use piston_window::{EventLoop, OpenGL, PistonWindow, UpdateEvent, WindowSettings};


/// Conrod is backend agnostic. Here, we define the `piston_window` backend to use for our `Ui`.
type Backend = (piston_window::G2dTexture<'static>, piston_window::Glyphs);
type Ui = conrod::Ui<Backend>;
type UiCell<'a> = conrod::UiCell<'a, Backend>;


fn main() {

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("Text Demo", [360, 360])
            .opengl(OpenGL::V3_2).exit_on_esc(true).samples(4).vsync(true).build().unwrap();

    // Construct our `Ui`.
    let mut ui = {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        let theme = Theme::default();
        let glyph_cache = piston_window::Glyphs::new(&font_path, window.factory.clone()).unwrap();
        Ui::new(glyph_cache, theme)
    };

    window.set_ups(60);

    let mut oval_range = (0.25, 0.75);

    // Poll events from the window.
    while let Some(event) = window.next() {
        ui.handle_event(event.clone());
        event.update(|_| ui.set_widgets(|ui_cell| set_ui(ui_cell, &mut oval_range)));
        window.draw_2d(&event, |c, g| ui.draw_if_changed(c, g));
    }

}

// Declare the `WidgetId`s and instantiate the widgets.
fn set_ui(ref mut ui: UiCell, oval_range: &mut (conrod::Scalar, conrod::Scalar)) {
    use conrod::{Canvas, color, Colorable, Oval, Positionable, RangeSlider, Sizeable};

    widget_ids!{CANVAS, OVAL, RANGE_SLIDER};

    Canvas::new().color(color::DARK_CHARCOAL).set(CANVAS, ui);

    const PAD: conrod::Scalar = 20.0;
    let (ref mut start, ref mut end) = *oval_range;
    let min = 0.0;
    let max = 1.0;
    RangeSlider::new(*start, *end, min, max)
        .color(color::LIGHT_BLUE)
        .padded_w_of(CANVAS, PAD)
        .h(30.0)
        .mid_top_with_margin_on(CANVAS, PAD)
        .react(|edge, value| match edge {
            conrod::RangeSliderEdge::Start => *start = value,
            conrod::RangeSliderEdge::End => *end = value,
        })
        .set(RANGE_SLIDER, ui);

    let range_slider_w = ui.w_of(RANGE_SLIDER).unwrap();
    let w = (*end - *start) * range_slider_w;
    let h = 200.0;
    Oval::fill([w, h])
        .mid_left_with_margin_on(CANVAS, PAD + *start * range_slider_w)
        .color(color::LIGHT_BLUE)
        .down(50.0)
        .set(OVAL, ui);
}
