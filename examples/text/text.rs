
#![feature(if_let)]

extern crate shader_version;
extern crate event;
extern crate conrod;
extern crate graphics;
extern crate sdl2_window;
extern crate opengl_graphics;
extern crate vecmath;

use conrod::{
    Background,
    Color,
    Colorable,
    Drawable,
    Label,
    Labelable,
    Positionable,
    UiContext,
};
use graphics::{
    AddColor,
    Draw,
};
use opengl_graphics::{ Gl, Texture };
use event::{
    WindowSettings,
    EventIterator,
    EventSettings,
    Render,
};
use sdl2_window::Sdl2Window;

/// This struct holds all of the variables used to demonstrate
/// application data being passed through the widgets. If some
/// of these seem strange, that's because they are! Most of
/// these simply represent the aesthetic state of different
/// parts of the GUI to offer visual feedback during interaction
/// with the widgets.
struct DemoApp {
    /// Background color (for demonstration of button and sliders).
    bg_color: Color,
    /// The number of pixels between the left side of the window
    /// and the title.
    title_padding: f64,
}

impl DemoApp {
    /// Constructor for the Demonstration Application data.
    fn new() -> DemoApp {
        DemoApp {
            bg_color: Color::new(0.2, 0.35, 0.45, 1.0),
            title_padding: 50.0,
        }
    }
}

fn main() {

    // Create a SDL2 window.
    let mut window = Sdl2Window::new(
        shader_version::opengl::OpenGL_3_2,
        WindowSettings {
            title: "Hello Conrod".to_string(),
            size: [1180, 580],
            fullscreen: false,
            exit_on_esc: true,
            samples: 4,
        }
    );

    // Some settings for how the game should be run.
    let event_settings = EventSettings {
        updates_per_second: 180,
        max_frames_per_second: 60
    };

    // Create GameIterator to begin the event iteration loop.
    let mut event_iter = EventIterator::new(&mut window, &event_settings);
    // Create OpenGL instance.
    let mut gl = Gl::new(shader_version::opengl::OpenGL_3_2);
    // Create the UiContext and specify the name of a font that's in our "assets" directory.
    let mut uic = UiContext::new(&Path::new("./assets/Dense-Regular.otf"), None);
    // Create the Demonstration Application data.
    let mut demo = DemoApp::new();

    // Main program loop begins.
    for event in event_iter {
        uic.handle_event(&event);
        if let Render(_) = event {
            uic.with_texture_constructor(
                |buffer, width, height| {
                    Texture::from_memory_alpha(buffer, width, height).unwrap()
                },
                |uic| {
                    draw_ui(&mut gl, uic, &mut demo)
                }
            );
        }
    }

}

/// Draw the User Interface.
fn draw_ui(gl: &mut Gl,
           uic: &mut UiContext<Texture>,
           demo: &mut DemoApp) {

    // Draw the background.
    uic.background().color(demo.bg_color).draw(gl);

    // Label example.
    uic.label("Widget Demonstration")
        .position(demo.title_padding, 30.0)
        .size(48u32)
        .color(demo.bg_color.plain_contrast())
        .draw(gl);

}

