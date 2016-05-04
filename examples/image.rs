//!
//! A simple demonstration of how to instantiate an `Image` widget.
//!

#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

fn main() {
    use conrod::{Canvas, Colorable, Image, Positionable, Theme, Widget, color};
    use piston_window::{EventLoop, Flip, G2d, Glyphs, OpenGL, PistonWindow, Texture, TextureSettings,
                        UpdateEvent, WindowSettings};
    use std::sync::Arc;

    // Conrod is backend agnostic. Here, we define the `piston_window` backend to use for our `Ui`.
    type Backend = (<G2d<'static> as conrod::Graphics>::Texture, Glyphs);
    type Ui = conrod::Ui<Backend>;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("Image Widget Demonstration", [800, 600])
            .opengl(opengl).exit_on_esc(true).vsync(true).samples(4).build().unwrap();

    // Get the path to our `assets` directory (where the fonts and images are).
    let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();

    // construct our `Ui`.
    let mut ui = {
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        let theme = Theme::default();
        let glyph_cache = Glyphs::new(&font_path, window.factory.clone());
        Ui::new(glyph_cache.unwrap(), theme)
    };

    // The texture to use for the `Image`.
    let rust_logo = {
        let path = assets.join("images/rust.png");
        let factory = &mut window.factory;
        Arc::new(Texture::from_path(factory, &path, Flip::None, &TextureSettings::new()).unwrap())
    };

    window.set_ups(60);

    // Poll events from the window.
    while let Some(event) = window.next() {
        ui.handle_event(&event);
        window.draw_2d(&event, |c, g| ui.draw_if_changed(c, g));
        event.update(|_| ui.set_widgets(|mut ui| {
            widget_ids!(CANVAS, RUST_LOGO);
            Canvas::new().color(color::LIGHT_BLUE).set(CANVAS, &mut ui);
            Image::from_texture(rust_logo.clone())
                .middle_of(CANVAS)
                .set(RUST_LOGO, &mut ui);
        }));
    }
}
