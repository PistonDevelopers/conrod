//!
//! A simple demonstration of how to instantiate an `Image` widget.
//!

#[macro_use] extern crate conrod;
extern crate find_folder;

use conrod::{widget, Colorable, Positionable, Sizeable, Widget, color};
use conrod::backend::piston::gfx::{GfxContext, G2dTexture, Texture, TextureSettings, Flip};
use conrod::backend::piston::{self, Window, WindowEvents, OpenGL};
use conrod::backend::piston::draw::ImageSize;
use conrod::backend::piston::event::UpdateEvent;

fn main() {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Construct the window.
    let mut window: Window =
        piston::window::WindowSettings::new("Image Widget Demonstration", [WIDTH, HEIGHT])
            .opengl(opengl).exit_on_esc(true).vsync(true).samples(4).build().unwrap();

    // Create the event loop.
    let mut events = WindowEvents::new();

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Create an empty texture to pass for the text cache as we're not drawing any text.
    let mut text_texture_cache = piston::window::GlyphCache::new(&mut window, 0, 0);

    // The `WidgetId` for our background and `Image` widgets.
    widget_ids!(struct Ids { background, rust_logo });
    let ids = Ids::new(ui.widget_id_generator());

    // Create our `conrod::image::Map` which describes each of our widget->image mappings.
    // In our case we only have one image, however the macro may be used to list multiple.
    let image_map = image_map! {
        (ids.rust_logo, load_rust_logo(&mut window.context)),
    };

    // We'll instantiate the `Image` at its full size, so we'll retrieve its dimensions.
    let (w, h) = image_map.get(&ids.rust_logo).unwrap().get_size();

    // Poll events from the window.
    while let Some(event) = window.next_event(&mut events) {

        // Convert the piston event to a conrod input event.
        if let Some(e) = piston::window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T { img };
                piston::window::draw(c, g, primitives,
                                     &mut text_texture_cache,
                                     &image_map,
                                     texture_from_image);
            }
        });

        event.update(|_| {
            let ui = &mut ui.set_widgets();
            // Draw a light blue background.
            widget::Canvas::new().color(color::LIGHT_BLUE).set(ids.background, ui);
            // Instantiate the `Image` at its full size in the middle of the window.
            widget::Image::new().w_h(w as f64, h as f64).middle().set(ids.rust_logo, ui);
        });
    }
}

// Load the Rust logo from our assets folder.
fn load_rust_logo(context: &mut GfxContext) -> G2dTexture<'static> {
    let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
    let path = assets.join("images/rust.png");
    let factory = &mut context.factory;
    let settings = TextureSettings::new();
    Texture::from_path(factory, &path, Flip::None, &settings).unwrap()
}
