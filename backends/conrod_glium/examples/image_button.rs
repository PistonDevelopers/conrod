//!
//! A demonstration of all non-primitive widgets available in Conrod.
//!
//!
//! Don't be put off by the number of method calls, they are only for demonstration and almost all
//! of them are optional. Conrod supports `Theme`s, so if you don't give it an argument, it will
//! check the current `Theme` within the `Ui` and retrieve defaults from there.
//!

#[macro_use] extern crate conrod_core;
extern crate conrod_glium;
#[macro_use] extern crate conrod_winit;
extern crate find_folder;
extern crate glium;
extern crate image;
extern crate rand; // for making a random color.

mod support;

use conrod_core::{widget, Borderable, Colorable, Positionable, Sizeable, Widget, color};
use glium::Surface;

const WIDTH: u32 = 1100;
const HEIGHT: u32 = 560;

fn main() {
    // Build the window.
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title("Image Button Demonstration")
        .with_dimensions((WIDTH, HEIGHT).into());
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let display = support::GliumDisplayWinitWrapper(display);

    // construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // A type used for converting `conrod_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod_glium::Renderer::new(&display.0).unwrap();

    // Declare the ID for each of our widgets.
    widget_ids!(struct Ids { canvas, button, rust_logo });
    let ids = Ids::new(ui.widget_id_generator());

    // Create our `conrod_core::image::Map` which describes each of our widget->image mappings.
    // In our case we only have one image, however the macro may be used to list multiple.
    let mut image_map = conrod_core::image::Map::new();

    struct ImageIds {
        normal: conrod_core::image::Id,
        hover: conrod_core::image::Id,
        press: conrod_core::image::Id,
    }

    // Load the images into our `ImageIds` type for easy access.
    let image_path = assets.join("images");
    let rust_logo = load_image(&display.0, image_path.join("rust.png"));
    let (w, h) = (rust_logo.get_width(), rust_logo.get_height().unwrap());
    let image_ids = ImageIds {
        normal: image_map.insert(rust_logo),
        hover: image_map.insert(load_image(&display.0, image_path.join("rust_hover.png"))),
        press: image_map.insert(load_image(&display.0, image_path.join("rust_press.png"))),
    };

    // We'll change the background colour with the image button.
    let mut bg_color = conrod_core::color::LIGHT_BLUE;

    // Poll events from the window.
    let mut event_loop = support::EventLoop::new();
    'main: loop {

        // Handle all events.
        for event in event_loop.next(&mut events_loop) {

            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = support::convert_event(event.clone(), &display) {
                ui.handle_event(event);
                event_loop.needs_update();
            }

            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::WindowEvent::CloseRequested |
                    glium::glutin::WindowEvent::KeyboardInput {
                        input: glium::glutin::KeyboardInput {
                            virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => break 'main,
                    _ => (),
                },
                _ => (),
            }
        }

        {
            let ui = &mut ui.set_widgets();

            // We can use this `Canvas` as a parent Widget upon which we can place other widgets.
            widget::Canvas::new()
                .pad(30.0)
                .color(bg_color)
                .set(ids.canvas, ui);

            // Button widget example button.
            if widget::Button::image(image_ids.normal)
                .hover_image(image_ids.hover)
                .press_image(image_ids.press)
                .w_h(w as conrod_core::Scalar, h as conrod_core::Scalar)
                .middle_of(ids.canvas)
                .border(0.0)
                .set(ids.button, ui)
                .was_clicked()
            {
                bg_color = color::rgb(rand::random(), rand::random(), rand::random());
            }
        }

        // Render the `Ui` and then display it on the screen.
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display.0, primitives, &image_map);
            let mut target = display.0.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display.0, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}

// Load an image from our assets folder as a texture we can draw to the screen.
fn load_image<P>(display: &glium::Display, path: P) -> glium::texture::SrgbTexture2d
    where P: AsRef<std::path::Path>,
{
    let path = path.as_ref();
    let rgba_image = image::open(&std::path::Path::new(&path)).unwrap().to_rgba();
    let image_dimensions = rgba_image.dimensions();
    let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&rgba_image.into_raw(), image_dimensions);
    let texture = glium::texture::SrgbTexture2d::new(display, raw_image).unwrap();
    texture
}
