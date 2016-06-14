#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use conrod::Widget;
use piston_window::{EventLoop, G2dTexture, PistonWindow, UpdateEvent, Window, WindowSettings};


fn main() {

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("PlotPath Demo", [720, 360])
            .opengl(piston_window::OpenGL::V3_2)
            .samples(4)
            .exit_on_esc(true)
            .build()
            .unwrap();

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
        if let Some(e) = conrod::backend::event_piston::convert_event(event.clone(), w, h) {
            ui.handle_event(e);
        }

        event.update(|_| ui.set_widgets(set_ui));

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {

                // Data and functions for rendering the primitives.
                let renderer = conrod::backend::draw_piston::Renderer {
                    context: c,
                    graphics: g,
                    texture_cache: &mut text_texture_cache,
                    // No text to draw.
                    cache_queued_glyphs: |_graphics: &mut piston_window::G2d,
                                          _cache: &mut G2dTexture<'static>,
                                          _rect: conrod::text::RtRect<u32>,
                                          _data: &[u8]| {
                        unimplemented!();
                    },
                    // No images to draw.
                    get_texture: |_id| None,
                };

                conrod::backend::draw_piston::primitives(primitives, renderer);
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
