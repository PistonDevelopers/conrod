//! A simple example demonstrating the `RangeSlider` widget.

#[macro_use] extern crate conrod;
extern crate find_folder;

use conrod::backend::piston::{self, Window, WindowEvents, OpenGL};
use conrod::backend::piston::event::UpdateEvent;

widget_ids! { 
    struct Ids { canvas, oval, range_slider }
}


fn main() {
    const WIDTH: u32 = 360;
    const HEIGHT: u32 = 360;

    // Construct the window.
    let mut window: Window =
        piston::window::WindowSettings::new("RangeSlider Demo", [WIDTH, HEIGHT])
            .opengl(OpenGL::V3_2)
            .exit_on_esc(true).samples(4).vsync(true).build().unwrap();

    // Create the event loop.
    let mut events = WindowEvents::new();

    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // A unique identifier for each widget.
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache = piston::window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    let mut oval_range = (0.25, 0.75);

    // Poll events from the window.
    while let Some(event) = window.next_event(&mut events) {

        // Convert the piston event to a conrod event.
        if let Some(e) = piston::window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            set_ui(ui.set_widgets(), &ids, &mut oval_range);
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

// Declare the `WidgetId`s and instantiate the widgets.
fn set_ui(ref mut ui: conrod::UiCell, ids: &Ids, oval_range: &mut (conrod::Scalar, conrod::Scalar)) {
    use conrod::{color, widget, Colorable, Positionable, Sizeable, Widget};

    widget::Canvas::new().color(color::DARK_CHARCOAL).set(ids.canvas, ui);

    const PAD: conrod::Scalar = 20.0;
    let (ref mut start, ref mut end) = *oval_range;
    let min = 0.0;
    let max = 1.0;
    for (edge, value) in widget::RangeSlider::new(*start, *end, min, max)
        .color(color::LIGHT_BLUE)
        .padded_w_of(ids.canvas, PAD)
        .h(30.0)
        .mid_top_with_margin_on(ids.canvas, PAD)
        .set(ids.range_slider, ui)
    {
        match edge {
            widget::range_slider::Edge::Start => *start = value,
            widget::range_slider::Edge::End => *end = value,
        }
    }

    let range_slider_w = ui.w_of(ids.range_slider).unwrap();
    let w = (*end - *start) * range_slider_w;
    let h = 200.0;
    widget::Oval::fill([w, h])
        .mid_left_with_margin_on(ids.canvas, PAD + *start * range_slider_w)
        .color(color::LIGHT_BLUE)
        .down(50.0)
        .set(ids.oval, ui);
}
