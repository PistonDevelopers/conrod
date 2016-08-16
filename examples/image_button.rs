//!
//! A demonstration of all non-primitive widgets available in Conrod.
//!
//!
//! Don't be put off by the number of method calls, they are only for demonstration and almost all
//! of them are optional. Conrod supports `Theme`s, so if you don't give it an argument, it will
//! check the current `Theme` within the `Ui` and retrieve defaults from there.
//!

#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;
extern crate rand; // for making a random color.

use piston_window::{EventLoop, ImageSize, PistonWindow, UpdateEvent, WindowSettings};


/// This struct holds all of the variables used to demonstrate application data being passed
/// through the widgets. If some of these seem strange, that's because they are! Most of these
/// simply represent the aesthetic state of different parts of the GUI to offer visual feedback
/// during interaction with the widgets.
struct Context {
    /// Background color (for demonstration of button and sliders).
    bg_color: conrod::Color,
}

impl Context {
    /// Constructor for the Demonstration Application model.
    fn new() -> Context {
        Context {
            bg_color: conrod::color::rgb(0.2, 0.35, 0.45),
        }
    }

}


fn main() {
    const WIDTH: u32 = 1100;
    const HEIGHT: u32 = 560;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = piston_window::OpenGL::V3_2;

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("A button with an image", [WIDTH, HEIGHT])
            .opengl(opengl).exit_on_esc(true).vsync(true).build().unwrap();

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new().build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache =
        conrod::backend::piston_window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // Declare the ID for each of our widgets.
    widget_ids! {
        CANVAS,
        BUTTON,
        RUST_LOGO
    }

    // Create our `conrod::image::Map` which describes each of our widget->image mappings.
    // In our case we only have one image, however the macro may be used to list multiple.
    let image_map = image_map! {
        (RUST_LOGO, rust_logo(&mut window)),
    };

    // We'll instantiate the `Button` at the logo's full size, so we'll retrieve its dimensions.
    let (w, h) = image_map.get(RUST_LOGO).unwrap().get_size();

    // Our demonstration app that we'll control with our GUI.
    let mut app = Context::new();

    window.set_ups(60);

    // Poll events from the window.
    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            use conrod::{color, widget, Colorable, Positionable, Sizeable, Widget};

            let ui = &mut ui.set_widgets();

            // We can use this `Canvas` as a parent Widget upon which we can place other widgets.
            widget::Canvas::new()
                .pad(30.0)
                .color(app.bg_color)
                .set(CANVAS, ui);

            // Button widget example button.
            if widget::Button::new()
                .w_h(w as conrod::Scalar, h as conrod::Scalar)
                .middle_of(CANVAS)
                .rgb(0.4, 0.75, 0.6)
                .image(RUST_LOGO)
                .set(BUTTON, ui)
	        	.was_clicked()
	        {
                app.bg_color = color::rgb(rand::random(), rand::random(), rand::random());
	        }
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


// Load the Rust logo from our assets folder.
use piston_window::{Flip, G2dTexture, Texture};
fn rust_logo(window: &mut PistonWindow) -> G2dTexture<'static> {
    let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
    let path = assets.join("images/rust.png");
    let factory = &mut window.factory;
    let settings = piston_window::TextureSettings::new();
    Texture::from_path(factory, &path, Flip::None, &settings).unwrap()
}
