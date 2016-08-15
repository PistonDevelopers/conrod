//!
//! A simple demonstration of how to instantiate an `Image` widget.
//!

#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use conrod::{widget, Colorable, Positionable, Sizeable, Widget, color};
use piston_window::{EventLoop, Flip, ImageSize, G2dTexture, PistonWindow, Texture, UpdateEvent};

fn main() {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = piston_window::OpenGL::V3_2;

    // Construct the window.
    let mut window: PistonWindow =
        piston_window::WindowSettings::new("Image Widget Demonstration", [WIDTH, HEIGHT])
            .opengl(opengl).exit_on_esc(true).vsync(true).samples(4).build().unwrap();
    window.set_ups(60);

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new().build();

    // Create an empty texture to pass for the text cache as we're not drawing any text.
    let mut text_texture_cache = conrod::backend::piston_window::GlyphCache::new(&mut window, 0, 0);

    // The `WidgetId` for our background and `Image` widgets.
    widget_ids!(BACKGROUND, RUST_LOGO);

    // Create our `conrod::image::Map` which describes each of our widget->image mappings.
    // In our case we only have one image, however the macro may be used to list multiple.
    let image_map = image_map! {
        (RUST_LOGO, rust_logo(&mut window)),
    };

    // We'll instantiate the `Image` at its full size, so we'll retrieve its dimensions.
    let (w, h) = image_map.get(RUST_LOGO).unwrap().get_size();

    // Poll events from the window.
    while let Some(event) = window.next() {
        ui.handle_event(event.clone());

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T { img };
                conrod::backend::piston_window::draw(c, g, primitives,
                                                     &mut text_texture_cache,
                                                     &image_map,
                                                     texture_from_image);
            }
        });

        event.update(|_| {
            let ui = &mut ui.set_widgets();
            // Draw a light blue background.
            widget::Canvas::new().color(color::LIGHT_BLUE).set(BACKGROUND, ui);
            // Instantiate the `Image` at its full size in the middle of the window.
            widget::Image::new().w_h(w as f64, h as f64).middle().set(RUST_LOGO, ui);
        });
    }
}

// Load the Rust logo from our assets folder.
fn rust_logo(window: &mut PistonWindow) -> G2dTexture<'static> {
    let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
    let path = assets.join("images/rust.png");
    let factory = &mut window.factory;
    let settings = piston_window::TextureSettings::new();
    Texture::from_path(factory, &path, Flip::None, &settings).unwrap()
}
