extern crate conrod;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston;

use conrod::{
    Background,
    Button,
    Colorable,
    Shapeable,
    Drawable,
    Label,
    Positionable,
    Theme,
    Ui
};
use glutin_window::GlutinWindow;
use opengl_graphics::{ GlGraphics, OpenGL };
use opengl_graphics::glyph_cache::GlyphCache;
use piston::event::*;
use piston::window::{ WindowSettings, Size };
use std::path::Path;

fn main() {

    let opengl = OpenGL::_3_2;
    let window = GlutinWindow::new(
        opengl,
        WindowSettings::new(
            "Hello Conrod".to_string(),
            Size { width: 200, height: 100 }
        )
        .exit_on_esc(true)
        .samples(4)
    );
    let event_iter = window.events().ups(180).max_fps(60);
    let mut gl = GlGraphics::new(opengl);
    let font_path = Path::new("./assets/NotoSans/NotoSans-Regular.ttf");
    let theme = Theme::default();
    let glyph_cache = GlyphCache::new(&font_path).unwrap();
    let ui = &mut Ui::new(glyph_cache, theme);

    let mut count = 0;

    for event in event_iter {
        ui.handle_event(&event);
        if let Some(args) = event.render_args() {
            gl.draw(args.viewport(), |_, gl| {

                // Draw the background.
                Background::new().rgba(0.2, 0.25, 0.4, 1.0).draw(ui, gl);

                // Draw the counter.
                counter(gl, ui, &mut count)

            });
        }
    }

}

/// Function for drawing the counter widget.
fn counter<'a>(gl: &mut GlGraphics,
               ui: &mut Ui<GlyphCache<'a>>,
               count: &mut u32) {

    // Draw the value.
    Label::new(&count.to_string()).position(10.0, 10.0).draw(ui, gl);

    // Draw the button and increment count if pressed..
    Button::new(0)
        .position(110.0, 10.0)
        .dimensions(80.0, 80.0)
        .callback(|| *count += 1)
        .draw(ui, gl)

}
