extern crate conrod;
extern crate find_folder;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston;

use conrod::{Background, Button, color, Colorable, Labelable, Sizeable, Theme, Ui, Widget};
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use opengl_graphics::glyph_cache::GlyphCache;
use piston::event::*;
use piston::window::{WindowSettings, Size};

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
    let assets = find_folder::Search::Both(3, 3).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    let theme = Theme::default();
    let glyph_cache = GlyphCache::new(&font_path).unwrap();
    let ui = &mut Ui::new(glyph_cache, theme);

    let mut count: u32 = 0;

    for event in event_iter {
        ui.handle_event(&event);
        if let Some(args) = event.render_args() {
            gl.draw(args.viewport(), |c, gl| {

                // Draw the background.
                Background::new().rgb(0.2, 0.25, 0.4).draw(ui, gl);

                // Draw the button and increment count if pressed..
                Button::new()
                    .color(color::red())
                    .dimensions(80.0, 80.0)
                    .label(&count.to_string())
                    .react(|| count += 1)
                    .set(0, ui);

                // Draw our Ui!
                ui.draw(c, gl);

            });
        }
    }

}
