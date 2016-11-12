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
extern crate rand; // for making a random color.

use conrod::backend::piston::gfx::{GfxContext, G2dTexture, Texture, TextureSettings, Flip};
use conrod::backend::piston::{self, Window, WindowEvents, OpenGL};
use conrod::backend::piston::draw::ImageSize;
use conrod::backend::piston::event::UpdateEvent;

fn main() {
    const WIDTH: u32 = 1100;
    const HEIGHT: u32 = 560;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Construct the window.
    let mut window: Window =
        piston::window::WindowSettings::new("A button with an image", [WIDTH, HEIGHT])
            .opengl(opengl).exit_on_esc(true).vsync(true).build().unwrap();

    // Create the event loop.
    let mut events = WindowEvents::new();

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache = piston::window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // Declare the ID for each of our widgets.
    widget_ids!(struct Ids { canvas, button, rust_logo });
    let ids = Ids::new(ui.widget_id_generator());

    // Create our `conrod::image::Map` which describes each of our widget->image mappings.
    // In our case we only have one image, however the macro may be used to list multiple.
    let image_map = image_map! {
        (ids.rust_logo, load_rust_logo(&mut window.context)),
    };

    // We'll instantiate the `Button` at the logo's full size, so we'll retrieve its dimensions.
    let (w, h) = image_map.get(&ids.rust_logo).unwrap().get_size();

    // Our demonstration app that we'll control with our GUI.
    let mut bg_color = conrod::color::LIGHT_BLUE;

    // Poll events from the window.
    while let Some(event) = window.next_event(&mut events) {

        // Convert the piston event to a conrod event.
        if let Some(e) = piston::window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            use conrod::{color, widget, Colorable, Borderable, Positionable, Sizeable, Widget};

            let ui = &mut ui.set_widgets();

            // We can use this `Canvas` as a parent Widget upon which we can place other widgets.
            widget::Canvas::new()
                .pad(30.0)
                .color(bg_color)
                .set(ids.canvas, ui);

            // Button widget example button.
            if widget::Button::image(ids.rust_logo)
                .w_h(w as conrod::Scalar, h as conrod::Scalar)
                .middle_of(ids.canvas)
                .color(color::TRANSPARENT)
                .border(0.0)
                .image_color_with_feedback(color::BLACK)
                .set(ids.button, ui)
                .was_clicked()
            {
                bg_color = color::rgb(rand::random(), rand::random(), rand::random());
            }
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

// Load the Rust logo from our assets folder.
fn load_rust_logo(context: &mut GfxContext) -> G2dTexture<'static> {
    let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
    let path = assets.join("images/rust.png");
    let factory = &mut context.factory;
    let settings = TextureSettings::new();
    Texture::from_path(factory, &path, Flip::None, &settings).unwrap()
}
