
#![feature(if_let)]

extern crate sdl2;
extern crate gfx;
extern crate texture;
extern crate shader_version;
extern crate event;
extern crate conrod;
extern crate graphics;
extern crate gfx_graphics;
extern crate sdl2_window;
extern crate vecmath;

use gfx::{ Device, DeviceHelper };
use gfx_graphics::G2D;
use texture::FromMemoryAlpha;
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
    BackEnd,
    Draw,
    ImageSize,
};
use event::{
    Window,
    WindowSettings,
    EventIterator,
    EventSettings,
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

    let mut device = gfx::GlDevice::new(|s| unsafe {
        std::mem::transmute(sdl2::video::gl_get_proc_address(s))
    });
    let (w, h) = window.get_size();
    let frame = gfx::Frame::new(w as u16, h as u16);
    let mut renderer = device.create_renderer();

    // Create GameIterator to begin the event iteration loop.
    let mut event_iter = EventIterator::new(&mut window, &event_settings);
    // Create the UiContext and specify the name of a font that's in our "assets" directory.
    let mut uic = UiContext::new(&Path::new("./assets/Dense-Regular.otf"), None);
    // Create the Demonstration Application data.
    let mut demo = DemoApp::new();

    let mut g2d = G2D::new(&mut device);

    // Main program loop begins.
    for e in event_iter {
        use event::RenderEvent;
        uic.handle_event(&e);
        e.render(|_| {
            g2d.draw(&mut renderer, &frame, |_, g| {
                uic.with_texture_constructor(
                    |buffer, width, height| {
                        FromMemoryAlpha::from_memory_alpha(
                            &mut device, buffer, width, height,
                            |_, tex| tex).unwrap()
                    },
                    |uic| {
                        draw_ui(g, uic, &mut demo)
                    }
                );
            });
            
            device.submit(renderer.as_buffer());
            renderer.reset();
        });
    }

}

/// Draw the User Interface.
fn draw_ui<B: BackEnd<T>, T: ImageSize>(gl: &mut B,
           uic: &mut UiContext<T>,
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

