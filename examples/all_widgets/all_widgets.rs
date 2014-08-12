
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
    label,
    toggle,
    slider,
    Color,
    Point,
};
use conrod::label::{
    Label,
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
    // Create the UIContext.
    let mut uic = UIContext::new();
    // Background color (for demonstration of button and sliders).
    let mut bg_color = Color::new(0.05f32, 0.025f32, 0.1f32, 1f32);
    // Should the button be shown (for demonstration of button).
    let mut show_button = false;
    // The label that will be drawn to the Toggle.
    let mut toggle_label = "OFF".to_string();
    // The number of pixels between the left side of the window and the title.
    let mut title_padding = 50f64;

    // Main program loop begins.
    loop {
        match game_iter.next() {
            None => break,
            Some(mut e) => handle_event(&mut e,
                                        &mut gl,
                                        &mut uic,
                                        &mut bg_color,
                                        &mut show_button,
                                        &mut toggle_label,
                                        &mut title_padding),
        }
    }

}

/// Match the game event.
fn handle_event(event: &mut GameEvent,
                gl: &mut Gl,
                uic: &mut UIContext,
                bg_color: &mut Color,
                show_button: &mut bool,
                toggle_label: &mut String,
                title_padding: &mut f64) {
    uic.event(event);
    match *event {
        Render(ref mut args) => {
            draw_background(args, gl, bg_color);
            draw_ui(args, gl, uic, bg_color, show_button, toggle_label, title_padding);
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
           show_button: &mut bool,
           toggle_label: &mut String,
           title_padding: &mut f64) {

    // Label example.
    label::draw(args, // RenderArgs.
                gl, // Open GL instance.
                uic, // UIContext.
                Point::new(*title_padding, 30f64, 0f64), // Screen position.
                48u32, // Font size.
                Color::white(),
                "Widgets Demonstration");

    if *show_button {
        // Button widget example.
        button::draw(args, // RenderArgs.
                     gl, // Open GL instance.
                     uic, // UIContext.
                     0u64, // UI ID.
                     Point::new(50f64, 115f64, 0f64), // Screen position.
                     90f64, // Width.
                     60f64, // Height.
                     6f64, // Border.
                     Color::new(0.4f32, 0.75f32, 0.6f32, 1f32), // Button Color.
                     Label("PRESS", 24u32, Color::black()), // Label for button.
                     || { // Button "callback" event.
            *bg_color = Color::random();
        });
    }

    // Horizontal slider example.
    else {

        // Create the label for the slider.
        let pad = *title_padding as i16;
        let pad_string = pad.to_string();
        let label = "Padding: ".to_string().append(pad_string.as_slice());

        // Draw the slider.
        slider::draw(args, // RenderArgs.
                     gl, // OpenGL Instance.
                     uic, // UIContext.
                     1u64, // UIID
                     Point::new(50.0f64, 115.0, 0.0), // Screen position.
                     200f64, // Width.
                     50f64, // Height.
                     6f64, // Border.
                     Color::new(0.5, 0.3, 0.6, 1.0), // Rectangle color.
                     //NoLabel,
                     Label(label.as_slice(), 24u32, Color::white()),
                     pad as i16, // Slider value.
                     10i16, // Min value.
                     250i16, // Max value.
                     |new_pad| {
            *title_padding = new_pad as f64;
        });

    }

    // Clone the label toggle to be drawn.
    let label = toggle_label.clone();

    // Toggle widget example.
    toggle::draw(args, // RenderArgs.
                 gl, // Open GL instance.
                 uic, // UIContext.
                 2u64, // UI ID.
                 Point::new(50f64, 200f64, 0f64), // Screen position.
                 75f64, // Width.
                 75f64, // Height.
                 6f64, // Border.
                 Color::new(0.6f32, 0.25f32, 0.75f32, 1f32), // Button Color.
                 Label(label.as_slice(), 24u32, Color::white()),
                 *show_button, // bool.
                 |value| {
        *show_button = value;
        match value {
            true => {
                *toggle_label = "ON".to_string();
            },
            false => {
                *toggle_label = "OFF".to_string();
            },
        }
    });

    // Let's draw a slider for each color element.
    // 0 => red, 1 => green, 2 => blue.
    for i in range(0u, 3) {

        // We'll color the slider similarly to the color element which it will control.
        let color = match i {
            0u => Color::new(0.75f32, 0.3f32, 0.3f32, 1f32),
            1u => Color::new(0.3f32, 0.75f32, 0.3f32, 1f32),
            _  => Color::new(0.3f32, 0.3f32, 0.75f32, 1f32),
        };

        // Grab the value of the color element.
        let value = match i {
            0u => bg_color.r(),
            1u => bg_color.g(),
            _  => bg_color.b(),
        };

        // Create the label to be drawn with the slider.
        let mut label = value.to_string();
        if label.len() > 4u { label.truncate(4u); }

        // Vertical slider widget example.
        slider::draw(args, // RenderArgs.
                     gl, // Open GL instance
                     uic, // UIContext.
                     3u64 + i as u64, // UI ID.
                     Point::new(50f64 + i as f64 * 60f64, 300f64, 0f64), // Position.
                     35f64, // Width.
                     200f64, // Height.
                     6f64, // Border.
                     color, // Slider color.
                     //NoLabel,
                     Label(label.as_slice(), 24u32, Color::white()),
                     value, // Slider value.
                     0f32, // Minimum value.
                     1f32, // Maximum value.
                     |color| {
            match i {
                0u => { bg_color.set_r(color); },
                1u => { bg_color.set_g(color); },
                _ => { bg_color.set_b(color); },
            }
        });

    }

}

