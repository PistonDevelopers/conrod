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

use piston_window::{EventLoop, PistonWindow, UpdateEvent, WindowSettings};


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
        WindowSettings::new("All The Widgets!", [WIDTH, HEIGHT])
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

    // Create our `conrod::image::Map` which describes each of our widget->image mappings.
    // In our case we only have one image, however the macro may be used to list multiple.
    let image_map = image_map! {
        (RUST_LOGO, rust_logo(&mut window)),
    };

    // Our demonstration app that we'll control with our GUI.
    let mut app = Context::new();


    window.set_ups(60);

    // Poll events from the window.
    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        // We'll set all our widgets in a single function called `set_widgets`.
        event.update(|_| {
            ui.set_widgets(|mut ui| {
                set_widgets(&mut ui, &mut app);
            });
        });

        // Draw our Ui!
        //
        // The `draw_if_changed` method only re-draws the GUI if some `Widget`'s `Element`
        // representation has changed. Normally, a `Widget`'s `Element` should only change
        // if a Widget was interacted with in some way, however this is up to the `Widget`
        // designer's discretion.
        //
        // If instead you need to re-draw your conrod GUI every frame, use `Ui::draw`.
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

// In conrod, each widget must have its own unique identifier so that the `Ui` can keep track
// of its state between updates.
//
// To make this easier, conrod provides the `widget_ids` macro, which generates a unique
// const `widget::Id` for each identifier given in the list.
//
// The `with n` syntax reserves `n` number of `widget::Id`s for that identifier, rather than
// just one.
//
// This is often useful when you need to use an identifier in some kind of loop.
widget_ids! {
    CANVAS,
    BUTTON,
    RUST_LOGO
}

/// Set all `Widget`s within the User Interface.
///
/// The first time this gets called, each `Widget`'s `State` will be initialised and cached within
/// the `Ui` at their given indices. Every other time this get called, the `Widget`s will avoid any
/// allocations by updating the pre-existing cached state. A new graphical `Element` is only
/// retrieved from a `Widget` in the case that it's `State` has changed in some way.
fn set_widgets(ui: &mut conrod::UiCell, app: &mut Context) {
    use conrod::{color, widget, Colorable, Positionable, Sizeable, Texturable, Widget};

    // We can use this `Canvas` as a parent Widget upon which we can place other widgets.
    widget::Canvas::new()
        .pad(30.0)
        .color(app.bg_color)
        .scroll_kids()
        .set(CANVAS, ui);
    // Button widget example button.
    widget::Button::new()
        .w_h(200.0, 200.0)
        .middle_of(CANVAS)
        .rgb(0.4, 0.75, 0.6)
        .texture(widget::Index::from(RUST_LOGO))
        .react(|| app.bg_color = color::rgb(rand::random(), rand::random(), rand::random()))
        .set(BUTTON, ui);
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
