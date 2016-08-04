//! A simple example demonstrating the `RangeSlider` widget.

#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use piston_window::{EventLoop, PistonWindow, UpdateEvent, WindowSettings};


fn main() {
    const WIDTH: u32 = 360;
    const HEIGHT: u32 = 360;

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("RangeSlider Demo", [WIDTH, HEIGHT])
            .opengl(piston_window::OpenGL::V3_2)
            .exit_on_esc(true).samples(4).vsync(true).build().unwrap();
    window.set_ups(60);

    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new().build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache =
        conrod::backend::piston_window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    let mut oval_range = (0.25, 0.75);

    // Poll events from the window.
    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            ui.set_widgets(|ui_cell| set_ui(ui_cell, &mut oval_range));
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
fn set_ui(ref mut ui: conrod::UiCell, oval_range: &mut (conrod::Scalar, conrod::Scalar)) {
    use conrod::{Canvas, color, Colorable, Oval, Positionable, RangeSlider, Sizeable, Widget};

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
