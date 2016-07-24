//!
//! A simple demonstration of how to instantiate an `Image` widget.
//!

#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

fn main() {
    use conrod::{Canvas, Colorable, Image, Positionable, Sizeable, Theme, Widget, color};
    use piston_window::{EventLoop, Flip, ImageSize, G2dTexture, PistonWindow, Texture, UpdateEvent};

    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = piston_window::OpenGL::V3_2;

    // Construct the window.
    let mut window: PistonWindow =
        piston_window::WindowSettings::new("Image Widget Demonstration", [WIDTH, HEIGHT])
            .opengl(opengl).exit_on_esc(true).vsync(true).samples(4).build().unwrap();

    // construct our `Ui`.
    let mut ui = conrod::Ui::new(Theme::default());

    // Create an empty texture to pass for the text cache as we're not drawing any text.
    let mut text_texture_cache: G2dTexture<'static> =
        G2dTexture::empty(&mut window.factory).unwrap();

    // The texture to use for the `Image`.
    let mut texture_map = conrod::texture::Map::new();
    let (rust_logo, logo_w, logo_h) = {
        let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
        let path = assets.join("images/rust.png");
        let factory = &mut window.factory;
        let settings = piston_window::TextureSettings::new();
        let texture = Texture::from_path(factory, &path, Flip::None, &settings).unwrap();
        let (w, h) = texture.get_size();
        let id = texture_map.insert(texture);
        (id, w as conrod::Scalar, h as conrod::Scalar)
    };

    window.set_ups(60);

    // Poll events from the window.
    while let Some(event) = window.next() {
        ui.handle_event(event.clone());

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {

                // Data and functions for rendering the primitives.
                let renderer = conrod::backend::piston::draw::Renderer {
                    context: c,
                    graphics: g,
                    texture_cache: &mut text_texture_cache,
                    cache_queued_glyphs: |_graphics: &mut piston_window::G2d,
                                          _cache: &mut G2dTexture<'static>,
                                          _rect: conrod::text::rt::Rect<u32>,
                                          _data: &[u8]| {
                        // No text to draw.
                        unimplemented!();
                    },
                    // A function that returns some texture `T` for the given `texture::Id`. We
                    // have no `Image` widgets, so no need to implement this.
                    get_texture: |id| texture_map.get(id),
                };

                conrod::backend::piston::draw::primitives(primitives, renderer);
            }
        });

        event.update(|_| ui.set_widgets(|mut ui| {
            widget_ids!(CANVAS, RUST_LOGO);
            Canvas::new().color(color::LIGHT_BLUE).set(CANVAS, &mut ui);
            Image::new(rust_logo)
                .w_h(logo_w, logo_h)
                .middle_of(CANVAS)
                .set(RUST_LOGO, &mut ui);
        }));
    }
}
