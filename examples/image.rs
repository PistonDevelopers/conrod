//!
//! A simple demonstration of how to instantiate an `Image` widget.
//!

#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use conrod::{Canvas, Image, Theme, Widget, color};
use piston_window::{EventLoop, Flip, Glyphs, PistonWindow, Texture, TextureSettings, UpdateEvent, WindowSettings};
use std::rc::Rc;

type Ui = conrod::Ui<Glyphs>;


fn main() {

    // Construct the window.
    let window: PistonWindow =
        WindowSettings::new("Image Widget Demonstration", [800, 600])
            .exit_on_esc(true).vsync(true).samples(4).build().unwrap();

    // Get the path to our `assets` directory (where the fonts and images are).
    let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();

    // construct our `Ui`.
    let mut ui = {
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        let theme = Theme::default();
        let glyph_cache = Glyphs::new(&font_path, window.factory.borrow().clone());
        Ui::new(glyph_cache.unwrap(), theme)
    };

    // The texture to use for the `Image`.
    let rust_logo = {
        let path = assets.join("images/rust.png");
        let factory = &mut *window.factory.borrow_mut();
        Rc::new(Texture::from_path(factory, &path, Flip::None, &TextureSettings::new()).unwrap())
    };

    // Poll events from the window.
    for event in window.ups(60) {
        ui.handle_event(&event);
        event.update(|_| ui.set_widgets(|ui| {
            widget_ids!(RUST_LOGO);
            Image::from_texture(rust_logo).set(RUST_LOGO, &mut ui);
        }));
        event.draw_2d(|c, g| ui.draw_if_changed(c, g));
    }

}
