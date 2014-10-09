
#![feature(phase)]

#[phase(plugin, link)]
extern crate conrod;
extern crate graphics;
extern crate piston;
extern crate sdl2_game_window;
extern crate opengl_graphics;

use conrod::{
    Button,
    Callable,
    Color,
    Colorable,
    Drawable,
    DropDownList,
    EnvelopeEditor,
    Frameable,
    Label,
    Labelable,
    NumberDialer,
    Point,
    Positionable,
    Slider,
    Shapeable,
    TextBox,
    Toggle,
    UIContext,
    WidgetMatrix,
    XYPad,
};
use graphics::{
    AddColor,
    AddEllipse,
    Context,
    Draw,
};
use opengl_graphics::Gl;
use piston::{
    Event,
    WindowSettings,
    EventIterator,
    EventSettings,
    Render,
    RenderArgs,
};
use sdl2_game_window::WindowSDL2;

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
    envelopes: Vec<(Vec<Point<f32>>, String)>,
}

impl DemoApp {
    /// Constructor for the Demonstration Application data.
    fn new() -> DemoApp {
        DemoApp {
            bg_color: Color::new(0.2, 0.35, 0.45, 1.0),
            show_button: false,
            toggle_label: "OFF".to_string(),
            title_padding: 50.0,
            v_slider_height: 230.0,
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
            envelopes: vec![(vec![ Point::new(0.0, 0.0, 0.0),
                                   Point::new(0.1, 17000.0, 0.0),
                                   Point::new(0.25, 8000.0, 0.0),
                                   Point::new(0.5, 2000.0, 0.0),
                                   Point::new(1.0, 0.0, 0.0), ], "Envelope A".to_string()),
                            (vec![ Point::new(0.0, 0.85, 0.0),
                                   Point::new(0.3, 0.2, 0.0),
                                   Point::new(0.6, 0.6, 0.0),
                                   Point::new(1.0, 0.0, 0.0), ], "Envelope B".to_string())],
        }
    }
}

fn main() {

    // Create a SDL2 window.
    let mut window = WindowSDL2::new(
        piston::shader_version::opengl::OpenGL_3_2,
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
    let mut gl = Gl::new(piston::shader_version::opengl::OpenGL_3_2);
    // Create the UIContext and specify the name of a font that's in our "assets" directory.
    let mut uic = UIContext::new("Dense-Regular.otf");
    // Create the Demonstration Application data.
    let mut demo = DemoApp::new();

    // Main program loop begins.
    for e in event_iter {
        handle_event(&e, &mut gl, &mut uic, &mut demo);
    }

}

/// Match the game event.
fn handle_event(event: &Event,
                gl: &mut Gl,
                uic: &mut UIContext,
                demo: &mut DemoApp) {
    uic.handle_event(event);
    match *event {
        Render(ref args) => {
            draw_background(args, gl, &demo.bg_color);
            draw_ui(gl, uic, demo);
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
fn draw_ui(gl: &mut Gl,
           uic: &mut UIContext,
           demo: &mut DemoApp) {

    // Label example.
    uic.label("Widget Demonstration")
        .position(demo.title_padding, 30.0)
        .size(48u32)
        .color(demo.bg_color.plain_contrast())
        .draw(gl);

    if demo.show_button {

        // Button widget example button(UIID).
        uic.button(0u64)
            .dimensions(90.0, 60.0)
            .position(50.0, 115.0)
            .rgba(0.4, 0.75, 0.6, 1.0)
            .frame(demo.frame_width, Color::black())
            .label("PRESS", 24u32, Color::black())
            .callback(|| demo.bg_color = Color::random())
            .draw(gl);

    }

    // Horizontal slider example.
    else {

        // Create the label for the slider.
        let pad = demo.title_padding as i16;
        let pad_string = pad.to_string();
        let label = {
            let mut text = "Padding: ".to_string();
            text.push_str(pad_string.as_slice());
            text
        };

        // Slider widget example slider(UIID, value, min, max).
        uic.slider(1u64, pad as i16, 10, 910)
            .dimensions(200.0, 50.0)
            .position(50.0, 115.0)
            .rgba(0.5, 0.3, 0.6, 1.0)
            .frame(demo.frame_width, Color::black())
            .label(label.as_slice(), 24u32, Color::white())
            .callback(|new_pad| demo.title_padding = new_pad as f64)
            .draw(gl);

    }

    // Clone the label toggle to be drawn.
    let label = demo.toggle_label.clone();

    // Toggle widget example toggle(UIID, value).
    uic.toggle(2u64, demo.show_button)
        .dimensions(75.0, 75.0)
        .down(20.0)
        .rgba(0.6, 0.25, 0.75, 1.0)
        .frame(demo.frame_width, Color::black())
        .label(label.as_slice(), 24u32, Color::white())
        .callback(|value| {
            demo.show_button = value;
            demo.toggle_label = match value {
                true => "ON".to_string(),
                false => "OFF".to_string()
            }
        })
        .draw(gl);

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

        // Slider widget examples. slider(UIID, value, min, max)
        uic.slider(3u64 + i as u64, value, 0.0, 1.0)
            .dimensions(35.0, demo.v_slider_height)
            .position(50.0 + i as f64 * 60.0, 300.0)
            .color(color)
            .frame(demo.frame_width, Color::black())
            .label(label.as_slice(), 24u32, Color::white())
            .callback(|color| match i {
                0u => demo.bg_color.set_r(color),
                1u => demo.bg_color.set_g(color),
                _ => demo.bg_color.set_b(color),
            })
            .draw(gl);

    }

    // Number Dialer widget example. number_dialer(UIID, value, min, max, precision)
    uic.number_dialer(6u64, demo.v_slider_height, 25.0, 250.0, 1u8)
        .dimensions(260.0, 60.0)
        .position(300.0, 115.0)
        .color(demo.bg_color.invert())
        .frame(demo.frame_width, Color::black())
        .label("Height (pixels)", 24u32, demo.bg_color.invert().plain_contrast())
        .callback(|new_height| demo.v_slider_height = new_height)
        .draw(gl);

    // Number Dialer widget example. number_dialer(UIID, value, min, max, precision)
    uic.number_dialer(7u64, demo.frame_width, 0.0, 15.0, 2u8)
        .dimensions(260.0, 60.0)
        .down(20.0)
        .color(demo.bg_color.invert().plain_contrast())
        .frame(demo.frame_width, demo.bg_color.plain_contrast())
        .label("Frame Width (pixels)", 24u32, demo.bg_color.plain_contrast())
        .callback(|new_width| demo.frame_width = new_width)
        .draw(gl);


    // A demonstration using widget_matrix to easily draw
    // a matrix of any kind of widget.
    let (cols, rows) = (8u, 8u);
    uic.widget_matrix(cols, rows)
        .dimensions(260.0, 260.0) // matrix width and height.
        .position(300.0, 270.0) // matrix position.
        .each_widget(|uic, num, col, row, pos, width, height| { // This is called for every widget.

            // Color effect for fun.
            let (r, g, b, a) = (
                0.5 + (col as f32 / cols as f32) / 2.0,
                0.75,
                1.0 - (row as f32 / rows as f32) / 2.0,
                1.0
            );

            // Now draw the widgets with the given callback.
            let val = demo.bool_matrix[col][row];
            uic.toggle(8u64 + num as u64, val)
                .dimensions(width, height)
                .position(pos.x, pos.y)
                .rgba(r, g, b, a)
                .frame(demo.frame_width, Color::black())
                .callback(|new_val| *demo.bool_matrix.get_mut(col).get_mut(row) = new_val)
                .draw(gl);

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
    draw_circle(uic.win_w, uic.win_h, gl, demo.circle_pos, ddl_color);

    // A demonstration using drop_down_list.
    uic.drop_down_list(75u64, &mut demo.ddl_colors, &mut demo.selected_idx)
        .dimensions(150.0, 40.0)
        .right_from(6u64, 50.0) // Position right from widget 6 by 50 pixels.
        .color(ddl_color)
        .frame(demo.frame_width, ddl_color.plain_contrast())
        .label("Colors", 24u32, ddl_color.plain_contrast())
        .callback(|selected_idx, new_idx, _string| *selected_idx = Some(new_idx))
        .draw(gl);

    // Draw an xy_pad.
    let label_color = Color::new(1.0, 1.0, 1.0, 0.5) * ddl_color.plain_contrast();
    uic.xy_pad(76u64, // UIID
               demo.circle_pos.x, 745.0, 595.0, // x range.
               demo.circle_pos.y, 320.0, 170.0) // y range.
        .dimensions(150.0, 150.0)
        .down(225.0)
        .color(ddl_color)
        .frame(demo.frame_width, Color::white())
        .label("Circle Position", 32u32, label_color)
        .line_width(2.0)
        .value_font_size(18u32)
        .callback(|new_x, new_y| {
            demo.circle_pos.x = new_x;
            demo.circle_pos.y = new_y;
        })
        .draw(gl);

    // Let's use the widget matrix to draw
    // one column of two envelope_editors,
    // each with its own text_box.
    let (cols, rows) = (1u, 2u);
    uic.widget_matrix(cols, rows)
        .position(810.0, 115.0)
        .dimensions(320.0, 425.0)
        .each_widget(|uic, num, _col, _row, pos, width, height| { // This is called for every widget.

            let (ref mut env, ref mut text) = *demo.envelopes.get_mut(num);
            let text_box_height = height / 4.0;
            let env_editor_height = height - text_box_height;
            let env_editor_pos = pos + Point::new(0.0, text_box_height, 0.0);
            let env_label_color = Color::new(1.0, 1.0, 1.0, 0.5)
                                * demo.bg_color.invert().plain_contrast();
            let env_y_max = match num { 0u => 20000.0, _ => 1.0 };
            let tbox_uiid = 77u64 + (num * 2u) as u64;
            let env_uiid = tbox_uiid + 1u64;
            let env_skew_y = match num { 0u => 3.0, _ => 1.0 };

            // Draw a TextBox. text_box(UIID, &mut String, FontSize)
            uic.text_box(tbox_uiid, text)
                .font_size(24u32)
                .dimensions(width, text_box_height - 10.0)
                .position(pos.x, pos.y)
                .frame(demo.frame_width, demo.bg_color.invert().plain_contrast())
                .color(demo.bg_color.invert())
                .draw(gl);

            // Draw an EnvelopeEditor.
            uic.envelope_editor(env_uiid, // UIID
                                env, // vector of `E: EnvelopePoint`s.
                                0.0, 1.0, 0.0, env_y_max) // x_min, x_max, y_min, y_max.
                .dimensions(width, env_editor_height - 10.0)
                .position(env_editor_pos.x, env_editor_pos.y)
                .skew_y(env_skew_y)
                .color(demo.bg_color.invert())
                .frame(demo.frame_width, demo.bg_color.invert().plain_contrast())
                .label(text.as_slice(), 32u32, env_label_color)
                .point_radius(6.0)
                .line_width(2.0)
                .draw(gl);

        }); // End of matrix widget callback.

}

/// Draw a circle controlled by the XYPad.
fn draw_circle(win_w: f64,
               win_h: f64,
               gl: &mut Gl,
               pos: Point<f64>,
               color: Color) {
    let context = &Context::abs(win_w, win_h);
    let (r, g, b, a) = color.as_tuple();
    context
        .ellipse(pos.x, pos.y, 30.0, 30.0)
        .rgba(r, g, b, a)
        .draw(gl)
}

