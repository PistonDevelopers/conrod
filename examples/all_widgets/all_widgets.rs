
#![feature(phase)]

#[phase(plugin, link)]
extern crate conrod;
extern crate graphics;
extern crate piston;
extern crate sdl2_game_window;
extern crate opengl_graphics;

use conrod::{
    button,
    drop_down_list,
    envelope_editor,
    label,
    number_dialer,
    slider,
    toggle,
    widget_matrix,
    xy_pad,
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
    AddEllipse,
    Draw,
};
use opengl_graphics::Gl;
use piston::{
    GameEvent,
    GameWindowSettings,
    GameIterator,
    GameIteratorSettings,
    RenderArgs,
    Render,
};
use sdl2_game_window::GameWindowSDL2;

/// This struct holds all of the variables used to demonstrate
/// application data being passed through the widgets. If some
/// of these seem strange, that's because they are! Most of
/// these simply represent the aesthetic state of different
/// parts of the GUI to offer visual feedback during interaction
/// with the widgets.
struct DemoApp {
    /// Background color (for demonstration of button and sliders).
    bg_color: Color,
    /// Should the button be shown (for demonstration of button).
    show_button: bool,
    /// The label that will be drawn to the Toggle.
    toggle_label: String,
    /// The number of pixels between the left side of the window
    /// and the title.
    title_padding: f64,
    /// The height of the vertical sliders (we will play with this
    /// using a number_dialer).
    v_slider_height: f64,
    /// The widget frame width (we'll use this to demo Framing
    /// and number_dialer).
    frame_width: f64,
    /// Bool matrix for widget_matrix demonstration.
    bool_matrix: Vec<Vec<bool>>,
    /// A vector of strings for drop_down_list demonstration.
    ddl_colors: Vec<String>,
    /// We also need an Option<idx> to indicate whether or not an
    /// item is selected.
    selected_idx: Option<uint>,
    /// Co-ordinates for a little circle used to demonstrate the
    /// xy_pad.
    circle_pos: Point<f64>,
    /// Envelope for demonstration of EnvelopeEditor.
    envelopes: Vec<Vec<Point<f32>>>,
}

impl DemoApp {
    /// Constructor for the Demonstration Application data.
    fn new() -> DemoApp {
        DemoApp {
            bg_color: Color::new(0.2, 0.35, 0.45, 1.0),
            show_button: false,
            toggle_label: "OFF".to_string(),
            title_padding: 50.0,
            v_slider_height: 185.0,
            frame_width: 1.0,
            bool_matrix: vec![ vec![true, true, true, true, true, true, true, true],
                               vec![true, false, false, false, false, false, false, true],
                               vec![true, false, true, false, true, true, true, true],
                               vec![true, false, true, false, true, true, true, true],
                               vec![true, false, false, false, true, true, true, true],
                               vec![true, true, true, true, true, true, true, true],
                               vec![true, true, false, true, false, false, false, true],
                               vec![true, true, true, true, true, true, true, true] ],
            ddl_colors: vec!["Black".to_string(),
                              "White".to_string(),
                              "Red".to_string(),
                              "Green".to_string(),
                              "Blue".to_string()],
            selected_idx: None,
            circle_pos: Point::new(700.0, 200.0, 0.0),
            envelopes: vec![ vec![ Point::new(0.0, 0.0, 0.0),
                                   Point::new(0.1, 0.85, 0.0),
                                   Point::new(0.25, 0.4, 0.0),
                                   Point::new(0.5, 0.1, 0.0),
                                   Point::new(1.0, 0.0, 0.0), ],
                             vec![ Point::new(0.0, 0.0, 0.0),
                                   Point::new(0.4, 0.6, 0.0),
                                   Point::new(1.0, 0.0, 0.0), ],
                             vec![ Point::new(0.0, 0.85, 0.0),
                                   Point::new(0.3, 0.2, 0.0),
                                   Point::new(0.6, 0.6, 0.0),
                                   Point::new(1.0, 0.0, 0.0), ] ],
        }
    }
}

fn main() {

    // Create a SDL2 window.
    let mut window = GameWindowSDL2::new(
        piston::shader_version::opengl::OpenGL_3_2,
        GameWindowSettings {
            title: "Hello Conrod".to_string(),
            size: [1200, 600],
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
    // Create the Demonstration Application data.
    let mut demo = DemoApp::new();

    // Main program loop begins.
    loop {
        match game_iter.next() {
            None => break,
            Some(mut e) => handle_event(&mut e, &mut gl, &mut uic, &mut demo),
        }
    }

}

/// Match the game event.
fn handle_event(event: &mut GameEvent,
                gl: &mut Gl,
                uic: &mut UIContext,
                demo: &mut DemoApp) {
    uic.event(event);
    match *event {
        Render(ref mut args) => {
            draw_background(args, gl, &demo.bg_color);
            draw_ui(args, gl, uic, demo);
        },
        _ => (),
    }
}

/// Draw the window background.
fn draw_background(args: &RenderArgs, gl: &mut Gl, bg_color: &Color) {
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
           demo: &mut DemoApp) {

    // Label example.
    label::draw(args, // RenderArgs.
                gl, // Open GL instance.
                uic, // UIContext.
                Point::new(demo.title_padding, 30f64, 0f64), // Screen position.
                48u32, // Font size.
                demo.bg_color.plain_contrast(),
                "Widgets Demonstration");

    if demo.show_button {
        // Button widget example.
        button::draw(args, // RenderArgs.
                     gl, // Open GL instance.
                     uic, // UIContext.
                     0u64, // UI ID.
                     Point::new(50f64, 115f64, 0f64), // Screen position.
                     90f64, // Width.
                     60f64, // Height.
                     Frame(demo.frame_width, Color::black()), // Widget Frame.
                     Color::new(0.4f32, 0.75f32, 0.6f32, 1f32), // Button Color.
                     Label("PRESS", 24u32, Color::black()), // Label for button.
                     || demo.bg_color = Color::random()); // Callback closure.
    }

    // Horizontal slider example.
    else {

        // Create the label for the slider.
        let pad = demo.title_padding as i16;
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
                     Frame(demo.frame_width, Color::black()), // Widget Frame.
                     Color::new(0.5, 0.3, 0.6, 1.0), // Rectangle color.
                     //NoLabel,
                     Label(label.as_slice(), 24u32, Color::white()),
                     pad as i16, // Slider value.
                     10i16, // Min value.
                     910i16, // Max value.
                     |new_pad| demo.title_padding = new_pad as f64); // Callback closure.

    }

    // Clone the label toggle to be drawn.
    let label = demo.toggle_label.clone();

    // Toggle widget example.
    toggle::draw(args, // RenderArgs.
                 gl, // Open GL instance.
                 uic, // UIContext.
                 2u64, // UI ID.
                 Point::new(50f64, 200f64, 0f64), // Screen position.
                 75f64, // Width.
                 75f64, // Height.
                 Frame(demo.frame_width, Color::black()), // Widget Frame.
                 Color::new(0.6f32, 0.25f32, 0.75f32, 1f32), // Button Color.
                 Label(label.as_slice(), 24u32, Color::white()),
                 demo.show_button, // bool.
                 |value| {
        demo.show_button = value;
        match value {
            true => {
                demo.toggle_label = "ON".to_string();
            },
            false => {
                demo.toggle_label = "OFF".to_string();
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
            0u => demo.bg_color.r(),
            1u => demo.bg_color.g(),
            _  => demo.bg_color.b(),
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
                     demo.v_slider_height, // Height.
                     Frame(demo.frame_width, Color::black()), // Widget Frame.
                     color, // Slider color.
                     //NoLabel,
                     Label(label.as_slice(), 24u32, Color::white()),
                     value, // Slider value.
                     0f32, // Minimum value.
                     1f32, // Maximum value.
                     |color| {
            match i {
                0u => { demo.bg_color.set_r(color); },
                1u => { demo.bg_color.set_g(color); },
                _ => { demo.bg_color.set_b(color); },
            }
        });

    }

    // Number Dialer example.
    number_dialer::draw(args, // RenderArgs.
                        gl, // OpenGL instance.
                        uic, // UIContext.
                        6u64, // UIID.
                        Point::new(300.0, 115.0, 0.0), // Position.
                        260.0, // width.
                        60.0, // height.
                        24u32, // Font size. If a label is given, that size will be used instead.
                        Frame(demo.frame_width, Color::black()), // Widget Frame
                        demo.bg_color.invert(), // Number Dialer Color.
                        Label("Height (pixels)", 24u32, demo.bg_color.invert().plain_contrast()),
                        demo.v_slider_height, // Initial value.
                        25f64, // Minimum value.
                        250f64, // Maximum value.
                        1u8, // Precision (number of digits to show after decimal point).
                        |new_height| demo.v_slider_height = new_height); // Callback closure.

    // Number Dialer example.
    number_dialer::draw(args, // RenderArgs.
                        gl, // OpenGL instance.
                        uic, // UIContext.
                        7u64, // UIID.
                        Point::new(300.0, 195.0, 0.0), // Position.
                        260.0, // width.
                        60.0, // height.
                        24u32, // Font size. If a label is given, label size will be used instead.
                        Frame(demo.frame_width, demo.bg_color.plain_contrast()), // Widget Frame
                        demo.bg_color.invert().plain_contrast(), // Number Dialer Color.
                        Label("Frame (pixels)", 24u32, demo.bg_color.plain_contrast()),
                        demo.frame_width, // Initial value.
                        0f64, // Minimum value.
                        15f64, // Maximum value.
                        2u8, // Precision (number of digits to show after decimal point).
                        |new_width| demo.frame_width = new_width); // Callback closure.

    // A demonstration using widget_matrix to easily draw
    // a matrix of any kind of widget.
    let (cols, rows) = (8u, 8u);
    widget_matrix::draw(cols, // cols.
                        rows, // rows.
                        Point::new(300.0, 270.0, 0.0), // matrix position.
                        260.0, // width.
                        260.0, // height.
                        |num, col, row, pos, width, height| { // This is called for every widget.

        // Now draw the widgets with the given callback.
        let val = demo.bool_matrix[col][row];
        toggle::draw(args, gl, uic, 8u64 + num as u64, pos, width, height,
                     Frame(demo.frame_width, Color::black()),
                     Color::new(0.5 + (col as f32 / cols as f32) / 2.0,
                                0.75,
                                1.0 - (row as f32 / rows as f32) / 2.0,
                                1.0),
                     NoLabel,
                     val,
                     |new_val| {
            *demo.bool_matrix.get_mut(col).get_mut(row) = new_val;
        });

    });

    let ddl_color = match demo.selected_idx {
        Some(idx) => match demo.ddl_colors[idx].as_slice() {
            "Black" => Color::black(),
            "White" => Color::white(),
            "Red" => Color::new(0.75, 0.4, 0.4, 1.0),
            "Green" => Color::new(0.4, 0.8, 0.4, 1.0),
            "Blue" => Color::new(0.4, 0.4, 0.8, 1.0),
            _ => Color::new(0.75, 0.55, 0.85, 1.0),
        },
        None => Color::new(0.75, 0.55, 0.85, 1.0),
    };

    // Draw the circle that's controlled by the XYPad.
    draw_circle(args, gl, demo.circle_pos, ddl_color);

    // A demonstration using drop_down_list.
    drop_down_list::draw(args, // RenderArgs.
                         gl, // OpenGL instance.
                         uic, // UIContext.
                         75u64, // UIID.
                         Point::new(620.0, 115.0, 0.0), // Position.
                         150.0, // width.
                         40.0, // height.
                         Frame(demo.frame_width, ddl_color.plain_contrast()),
                         ddl_color, // Color of drop_down_list.
                         Label("Colors", 24u32, ddl_color.plain_contrast()),
                         &mut demo.ddl_colors, // String vector.
                         &mut demo.selected_idx, // Currently selected string index.
                         |selected_idx, new_idx, _string| { // Callback (upon new selection).
        *selected_idx = Some(new_idx);
    });

    // Draw a xy_pad.
    xy_pad::draw(args, // RenderArgs.
                 gl, // OpenGL instance.
                 uic, // UIContext.
                 76u64, // UIID.
                 Point::new(620.0, 370.0, 0.0), // Position.
                 150.0, // width.
                 150.0, // height.
                 18u32, // font size.
                 Frame(demo.frame_width, Color::white()),
                 ddl_color, // rect color.
                 Label("Circle Position", 32u32,
                       Color::new(1.0, 1.0, 1.0, 0.5) * ddl_color.plain_contrast()),
                 demo.circle_pos.x, 760.0, 610.0, // x range.
                 demo.circle_pos.y, 320.0, 170.0, // y range.
                 |new_x, new_y| { // Callback when x/y changes or mousepress/release.
        demo.circle_pos.x = new_x;
        demo.circle_pos.y = new_y;
    });

    let (cols, rows) = (1u, 3u);
    widget_matrix::draw(cols, // cols.
                        rows, // rows.
                        Point::new(830.0, 115.0, 0.0), // matrix position.
                        320.0, // width.
                        415.0, // height.
                        |num, _col, _row, pos, width, height| { // This is called for every widget.

        // Draw an EnvelopeEditor.
        envelope_editor::draw(args, // RenderArgs.
                              gl, // OpenGL instance.
                              uic, // UIContext.
                              77u64 + num as u64, // UIID.
                              pos, // Position.
                              width, // Width.
                              height - 10.0, // Height.
                              18u32, // Font size.
                              Frame(demo.frame_width, demo.bg_color.invert().plain_contrast()),
                              demo.bg_color.invert(), // Rect color.
                              Label("Envelope Editor ", 32u32,
                                    Color::new(1.0, 1.0, 1.0, 0.5)
                                    * demo.bg_color.invert().plain_contrast()),
                              demo.envelopes.get_mut(num), // Envelope.
                              0.0, 1.0, // `x` axis range.
                              0.0, 1.0, // `y` axis range.
                              6.0, // Point radius.
                              2.0, // Line width.
                              |_env, _idx| { }); // Callback upon x/y changes or mousepress/release.

    });

}

/// Draw a circle controlled by the XYPad.
fn draw_circle(args: &RenderArgs,
               gl: &mut Gl,
               pos: Point<f64>,
               color: Color) {
    let context = &Context::abs(args.width as f64, args.height as f64);
    let (r, g, b, a) = color.as_tuple();
    context
        .ellipse(pos.x, pos.y, 30.0, 30.0)
        .rgba(r, g, b, a)
        .draw(gl)
}

