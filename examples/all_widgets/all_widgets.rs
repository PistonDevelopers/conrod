
#![feature(phase)]

#[phase(plugin, link)]
extern crate conrod;
extern crate graphics;
extern crate piston;
extern crate sdl2_game_window;
extern crate opengl_graphics;

use sdl2_game_window::GameWindowSDL2;
use opengl_graphics::Gl;
use piston::{
    Game,
    GameEvent,
    GameWindowSettings,
    GameIterator,
    GameIteratorSettings,
    RenderArgs,
    Render,
};
use conrod::{
    UIContext,
    button,
    toggle,
    slider,
    Color,
    Point,
};
use graphics::{
    Context,
    AddColor,
    Draw,
};

fn main() {

    // Create a SDL2 window.
    let mut window = GameWindowSDL2::new(
        GameWindowSettings {
            title: "Hello Conrod".to_string(),
            size: [600, 600],
            fullscreen: false,
            exit_on_esc: true
        }
    );

    // Some settings for how the game should be run.
    let game_iter_settings = GameIteratorSettings {
        updates_per_second: 180,
        max_frames_per_second: 60
    };

    // Create GameIterator to begin the event iteration loop.
    let mut game_iter = GameIterator::new(&mut window, &game_iter_settings);
    // Create OpenGL instance.
    let mut gl = Gl::new();
    // Create UI app.
    let mut uic = UIContext::new();
    // Background color (for demonstration or button and sliders).
    let mut bg_color = Color::new(0.05f32, 0.025f32, 0.1f32, 1f32);
    // Should the button be shown (for demonstration of button).
    let mut show_button = true;

    // Main program loop begins.
    loop {
        match game_iter.next() {
            None => break,
            Some(mut e) => handle_event(&mut e,
                                        &mut gl,
                                        &mut uic,
                                        &mut bg_color,
                                        &mut show_button),
        }
    }

}

/// Match the game event.
fn handle_event(event: &mut GameEvent,
                gl: &mut Gl,
                uic: &mut UIContext,
                bg_color: &mut Color,
                show_button: &mut bool) {
    uic.event(event);
    match *event {
        Render(ref mut args) => {
            draw_background(args, gl, bg_color);
            draw_ui(args, gl, uic, bg_color, show_button);
        },
        _ => (),
    }
}

/// Draw the window background.
fn draw_background(args: &RenderArgs,
                   gl: &mut Gl,
                   bg_color: &mut Color) {
    // Set up a context to draw into.
    let context = &Context::abs(args.width as f64, args.height as f64);
    // Return the individual  elements of the background color.
    let (r, g, b, a) = bg_color.as_tuple();
    // Draw the background.
    context.rgba(r, g, b, a).draw(gl);
}

/// Draw the User Interface.
fn draw_ui(args: &RenderArgs,
           gl: &mut Gl,
           uic: &mut UIContext,
           bg_color: &mut Color,
           show_button: &mut bool) {

    // Button widget example.
    if *show_button {
        button::draw(args, // RenderArgs.
                     gl, // Open GL instance.
                     uic, // UIContext.
                     0u, // UI ID.
                     Point::new(50f64, 50f64, 0f64), // Screen position.
                     90f64, // Width.
                     60f64, // Height.
                     6f64, // Border.
                     Color::new(0.4f32, 0.75f32, 0.6f32, 1f32), // Button Color.
                     || {
            *bg_color = Color::random();
        });
    }

    // Toggle widget example.
    toggle::draw(args, // RenderArgs.
                 gl, // Open GL instance.
                 uic, // UIContext.
                 1u, // UI ID.
                 Point::new(50f64, 150f64, 0f64), // Screen position.
                 75f64, // Width.
                 75f64, // Height.
                 6f64, // Border.
                 Color::new(0.6f32, 0.25f32, 0.75f32, 1f32), // Button Color.
                 *show_button, // bool.
                 |value| {
        *show_button = value;
    });

    // A slider for each color.
    for i in range(0u, 3) {

        let color = match i {
            0u => Color::new(0.75f32, 0.3f32, 0.3f32, 1f32),
            1u => Color::new(0.3f32, 0.75f32, 0.3f32, 1f32),
            _  => Color::new(0.3f32, 0.3f32, 0.75f32, 1f32),
        };

        let value = match i {
            0u => bg_color.r,
            1u => bg_color.g,
            _  => bg_color.b,
        };
        
        // Slider widget example.
        slider::draw(args, // RenderArgs.
                     gl, // Open GL instance
                     uic, // UIContext.
                     2u + i, // UI ID.
                     Point::new(50f64 + i as f64 * 60f64, 250f64, 0f64), // Position.
                     35f64, // Width.
                     200f64, // Height.
                     6f64, // Border.
                     color,
                     value,
                     0f32, // Minimum value.
                     1f32, // Maximum value.
                     |color| {
            match i {
                0u => { bg_color.r = color; },
                1u => { bg_color.g = color; },
                _ => { bg_color.b = color; },
            }
        });

    }
                 
}

