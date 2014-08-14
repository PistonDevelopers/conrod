
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
    button,
    label,
    number_dialer,
    slider,
    toggle,
    widget_matrix,
    Color,
    Point,
    Frame,
    UIContext,
};
use conrod::label::{
    Label,
    NoLabel,
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
    // Create the UIContext and specify the name of a font that's in our "assets" directory.
    let mut uic = UIContext::new("Dense-Regular.otf");

    // TODO: Put the following vars all in an 'app' struct or something and pass
    // as a mut reference to struct instead.

    // Background color (for demonstration of button and sliders).
    let mut bg_color = Color::new(0.2, 0.35, 0.45, 1.0);
    // Should the button be shown (for demonstration of button).
    let mut show_button = false;
    // The label that will be drawn to the Toggle.
    let mut toggle_label = "OFF".to_string();
    // The number of pixels between the left side of the window and the title.
    let mut title_padding = 50f64;
    // The height of the vertical sliders (we will play with this using a number_dialer).
    let mut v_slider_height = 185f64;
    // The widget frame width (we'll use this to demo Framing and number_dialer).
    let mut frame_width = 6f64;
    // Bool matrix for widget_matrix demonstration.
    let mut bool_matrix = vec![
                            vec![true, true, true, true, true, true, true, true],
                            vec![true, true, true, true, true, true, true, true],
                            vec![true, true, true, true, true, true, true, true],
                            vec![true, true, true, true, true, true, true, true],
                            vec![true, true, true, true, true, true, true, true],
                            vec![true, true, true, true, true, true, true, true],
                            vec![true, true, true, true, true, true, true, true],
                            vec![true, true, true, true, true, true, true, true],
                            ];

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
                                        &mut title_padding,
                                        &mut v_slider_height,
                                        &mut frame_width,
                                        &mut bool_matrix),
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
                title_padding: &mut f64,
                v_slider_height: &mut f64,
                frame_width: &mut f64,
                bool_matrix: &mut Vec<Vec<bool>>) {
    uic.event(event);
    match *event {
        Render(ref mut args) => {
            draw_background(args, gl, bg_color);
            draw_ui(args,
                    gl,
                    uic,
                    bg_color,
                    show_button,
                    toggle_label,
                    title_padding,
                    v_slider_height,
                    frame_width,
                    bool_matrix);
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
           title_padding: &mut f64,
           v_slider_height: &mut f64,
           frame_width: &mut f64,
           bool_matrix: &mut Vec<Vec<bool>>) {

    // Label example.
    label::draw(args, // RenderArgs.
                gl, // Open GL instance.
                uic, // UIContext.
                Point::new(*title_padding, 30f64, 0f64), // Screen position.
                48u32, // Font size.
                bg_color.plain_contrast(),
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
                     Frame(*frame_width, Color::black()), // Widget Frame.
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
                     Frame(*frame_width, Color::black()), // Widget Frame.
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
                 Frame(*frame_width, Color::black()), // Widget Frame.
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
                     *v_slider_height, // Height.
                     Frame(*frame_width, Color::black()), // Widget Frame.
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

    // Number Dialer example.
    number_dialer::draw(args, // RenderArgs.
                        gl, // OpenGL instance.
                        uic, // UIContext.
                        6u64, // UIID.
                        Point::new(350.0, 115.0, 0.0), // Position.
                        24u32, // Number Dialer font size.
                        Frame(*frame_width, Color::black()), // Widget Frame
                        bg_color.invert(), // Number Dialer Color.
                        Label("Height (pixels)", 24u32, bg_color.invert().plain_contrast()),
                        *v_slider_height, // Initial value.
                        25f64, // Minimum value.
                        250f64, // Maximum value.
                        1u8, // Precision (number of digits to show after decimal point).
                        |new_height| { // Callback closure.
        *v_slider_height = new_height;
    });

    // Number Dialer example.
    number_dialer::draw(args, // RenderArgs.
                        gl, // OpenGL instance.
                        uic, // UIContext.
                        7u64, // UIID.
                        Point::new(350.0, 195.0, 0.0), // Position.
                        24u32, // Number Dialer font size.
                        Frame(4.0, bg_color.plain_contrast()), // Widget Frame
                        bg_color.invert().plain_contrast(), // Number Dialer Color.
                        Label("Frame (pixels)", 24u32, bg_color.plain_contrast()),
                        *frame_width, // Initial value.
                        0f64, // Minimum value.
                        15f64, // Maximum value.
                        2u8, // Precision (number of digits to show after decimal point).
                        |new_width| { // Callback closure.
        *frame_width = new_width;
    });

    // A demonstration using widget_matrix to easily draw
    // a matrix of any kind of widget.
    let (cols, rows) = (8u, 8u);
    widget_matrix::draw(cols, // cols.
                        rows, // rows.
                        Point::new(300.0, 270.0, 0.0), // matrix position.
                        260.0, // width.
                        260.0, // height.
                        |num, col, row, pos, width, height| {
        // Now draw the widgets with the given callback.
        let val = (*bool_matrix)[col][row];
        toggle::draw(args, gl, uic, 8u64 + num as u64, pos, width, height,
                     Frame(*frame_width, Color::black()),
                     Color::new(0.5 + (col as f32 / cols as f32) / 2.0,
                                0.75,
                                1.0 - (row as f32 / rows as f32) / 2.0,
                                1.0),
                     NoLabel,
                     val,
                     |new_val| {
            *bool_matrix.get_mut(col).get_mut(row) = new_val;
        });
    });


}

