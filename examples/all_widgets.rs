extern crate piston;
extern crate conrod;
extern crate graphics;
extern crate opengl_graphics;
extern crate glutin_window;
extern crate vecmath;

use conrod::{
    Background,
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
    Theme,
    Toggle,
    WidgetMatrix,
    XYPad,
    Ui
};
use opengl_graphics::{ GlGraphics, OpenGL };
use opengl_graphics::glyph_cache::GlyphCache;
use piston::event::*;
use piston::window::{ WindowSettings, Size };
use glutin_window::GlutinWindow;
use std::path::Path;
use vecmath::vec2_add;

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
    selected_idx: Option<usize>,
    /// Co-ordinates for a little circle used to demonstrate the
    /// xy_pad.
    circle_pos: Point,
    /// Envelope for demonstration of EnvelopeEditor.
    envelopes: Vec<(Vec<Point>, String)>,
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
            circle_pos: [700.0, 200.0],
            envelopes: vec![(vec![ [0.0, 0.0],
                                   [0.1, 17000.0],
                                   [0.25, 8000.0],
                                   [0.5, 2000.0],
                                   [1.0, 0.0], ], "Envelope A".to_string()),
                            (vec![ [0.0, 0.85],
                                   [0.3, 0.2],
                                   [0.6, 0.6],
                                   [1.0, 0.0], ], "Envelope B".to_string())],
        }
    }
}

fn main() {
    let opengl = OpenGL::_3_2;
    let window = GlutinWindow::new(
        opengl,
        WindowSettings::new(
            "Hello Conrod".to_string(),
            Size { width: 1180, height: 580 }
        )
        .exit_on_esc(true)
        .samples(4)
    );
    let event_iter = window.events().ups(180).max_fps(60);
    let mut gl = GlGraphics::new(opengl);

    let font_path = Path::new("./assets/NotoSans/NotoSans-Regular.ttf");
    let theme = Theme::default();
    let glyph_cache = GlyphCache::new(&font_path).unwrap();
    let mut ui = Ui::new(glyph_cache, theme);
    let mut demo = DemoApp::new();

    for event in event_iter {
        ui.handle_event(&event);
        if let Some(args) = event.render_args() {
            gl.draw([0, 0, args.width as i32, args.height as i32], |_, gl| {
                draw_ui(gl, &mut ui, &mut demo);
            });
        }
    }
}

/// Draw the User Interface.
fn draw_ui<'a>(gl: &mut GlGraphics,
               ui: &mut Ui<GlyphCache<'a>>,
               demo: &mut DemoApp) {

    // Draw the background.
    Background::new().color(demo.bg_color).draw(ui, gl);

    // Label example.
    Label::new("Widget Demonstration")
        .position(demo.title_padding, 30.0)
        .size(32)
        .color(demo.bg_color.plain_contrast())
        .draw(ui, gl);

    if demo.show_button {

        // Button widget example button(UIID).
        Button::new(0)
            .dimensions(90.0, 60.0)
            .position(50.0, 115.0)
            .rgba(0.4, 0.75, 0.6, 1.0)
            .frame(demo.frame_width)
            .label("PRESS")
            .callback(|| demo.bg_color = Color::random())
            .draw(ui, gl);

    }

    // Horizontal slider example.
    else {

        // Create the label for the slider.
        let pad = demo.title_padding as i16;
        let pad_string = pad.to_string();
        let label = {
            let mut text = "Padding: ".to_string();
            text.push_str(&pad_string);
            text
        };

        // Slider widget example slider(UIID, value, min, max).
        Slider::new(1, pad as f32, 10.0, 910.0)
            .dimensions(200.0, 50.0)
            .position(50.0, 115.0)
            .rgba(0.5, 0.3, 0.6, 1.0)
            .frame(demo.frame_width)
            .label(&label)
            .label_color(Color::white())
            .callback(|new_pad: f32| demo.title_padding = new_pad as f64)
            .draw(ui, gl);

    }

    // Clone the label toggle to be drawn.
    let label = demo.toggle_label.clone();

    // Toggle widget example toggle(UIID, value).
    Toggle::new(2, demo.show_button)
        .dimensions(75.0, 75.0)
        .down(20.0, ui)
        .rgba(0.6, 0.25, 0.75, 1.0)
        .frame(demo.frame_width)
        .label(&label)
        .label_color(Color::white())
        .callback(|value| {
            demo.show_button = value;
            demo.toggle_label = match value {
                true => "ON".to_string(),
                false => "OFF".to_string()
            }
        })
        .draw(ui, gl);

    // Let's draw a slider for each color element.
    // 0 => red, 1 => green, 2 => blue.
    for i in 0..3 {

        // We'll color the slider similarly to the color element which it will control.
        let color = match i {
            0 => Color::new(0.75, 0.3, 0.3, 1.0),
            1 => Color::new(0.3, 0.75, 0.3, 1.0),
            _ => Color::new(0.3, 0.3, 0.75, 1.0),
        };

        // Grab the value of the color element.
        let value = match i {
            0 => demo.bg_color.r(),
            1 => demo.bg_color.g(),
            _ => demo.bg_color.b(),
        };

        // Create the label to be drawn with the slider.
        let mut label = value.to_string();
        if label.len() > 4 { label.truncate(4); }

        // Slider widget examples. slider(UIID, value, min, max)
        Slider::new(3 + i as u64, value, 0.0, 1.0)
            .dimensions(40.0, demo.v_slider_height)
            .position(50.0 + i as f64 * 60.0, 300.0)
            .color(color)
            .frame(demo.frame_width)
            .label(&label)
            .label_color(Color::white())
            .callback(|color| match i {
                0 => demo.bg_color.set_r(color),
                1 => demo.bg_color.set_g(color),
                _ => demo.bg_color.set_b(color),
            })
            .draw(ui, gl);

    }

    // Number Dialer widget example. number_dialer(UIID, value, min, max, precision)
    NumberDialer::new(6, demo.v_slider_height, 25.0, 250.0, 1u8)
        .dimensions(260.0, 60.0)
        .position(300.0, 115.0)
        .color(demo.bg_color.invert())
        .frame(demo.frame_width)
        .label("Height (px)")
        .label_color(demo.bg_color.invert().plain_contrast())
        .callback(|new_height| demo.v_slider_height = new_height)
        .draw(ui, gl);

    // Number Dialer widget example. number_dialer(UIID, value, min, max, precision)
    NumberDialer::new(7, demo.frame_width, 0.0, 15.0, 2u8)
        .dimensions(260.0, 60.0)
        .down(20.0, ui)
        .color(demo.bg_color.invert().plain_contrast())
        .frame(demo.frame_width)
        .frame_color(demo.bg_color.plain_contrast())
        .label("Frame Width (px)")
        .label_color(demo.bg_color.plain_contrast())
        .callback(|new_width| demo.frame_width = new_width)
        .draw(ui, gl);


    // A demonstration using widget_matrix to easily draw
    // a matrix of any kind of widget.
    let (cols, rows) = (8, 8);
    WidgetMatrix::new(cols, rows)
        .dimensions(260.0, 260.0) // matrix width and height.
        .position(300.0, 270.0) // matrix position.
        .each_widget(|num, col, row, pos, dim| { // This is called for every widget.

            // Color effect for fun.
            let (r, g, b, a) = (
                0.5 + (col as f32 / cols as f32) / 2.0,
                0.75,
                1.0 - (row as f32 / rows as f32) / 2.0,
                1.0
            );

            // Now draw the widgets with the given callback.
            let val = demo.bool_matrix[col][row];
            Toggle::new(8 + num as u64, val)
                .dim(dim)
                .point(pos)
                .rgba(r, g, b, a)
                .frame(demo.frame_width)
                .callback(|new_val: bool| demo.bool_matrix[col][row] = new_val)
                .draw(ui, gl);

        });

    let ddl_color = match demo.selected_idx {
        Some(idx) => match demo.ddl_colors[idx].as_ref() {
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
    draw_circle(ui.win_w, ui.win_h, gl, demo.circle_pos, ddl_color);

    // A demonstration using drop_down_list.
    DropDownList::new(75, &mut demo.ddl_colors, &mut demo.selected_idx)
        .dimensions(150.0, 40.0)
        .right_from(6u64, 50.0, ui) // Position right from widget 6 by 50 pixels.
        .color(ddl_color)
        .frame(demo.frame_width)
        .frame_color(ddl_color.plain_contrast())
        .label("Colors")
        .label_color(ddl_color.plain_contrast())
        .callback(|selected_idx: &mut Option<usize>, new_idx, _string| {
            *selected_idx = Some(new_idx)
        })
        .draw(ui, gl);

    // Draw an xy_pad.
    XYPad::new(76, // UIID
               demo.circle_pos[0], 745.0, 595.0, // x range.
               demo.circle_pos[1], 320.0, 170.0) // y range.
        .dimensions(150.0, 150.0)
        .down(225.0, ui)
        .color(ddl_color)
        .frame(demo.frame_width)
        .frame_color(Color::white())
        .label("Circle Position")
        .label_color(Color::new(1.0, 1.0, 1.0, 0.5) * ddl_color.plain_contrast())
        .line_width(2.0)
        .value_font_size(18u32)
        .callback(|new_x, new_y| {
            demo.circle_pos[0] = new_x;
            demo.circle_pos[1] = new_y;
        })
        .draw(ui, gl);

    // Let's use the widget matrix to draw
    // one column of two envelope_editors,
    // each with its own text_box.
    let (cols, rows) = (1, 2);
    WidgetMatrix::new(cols, rows)
        .position(810.0, 115.0)
        .dimensions(320.0, 425.0)
        .each_widget(|num, _col, _row, pos, dim| { // This is called for every widget.
            use conrod::draw::Drawable;

            let &mut (ref mut env, ref mut text) = &mut demo.envelopes[num];
            let text_box_height = dim[1] / 4.0;
            let env_editor_height = dim[1] - text_box_height;
            let env_editor_pos = vec2_add(pos, [0.0, text_box_height]);
            let env_label_color = Color::new(1.0, 1.0, 1.0, 0.5)
                                * demo.bg_color.invert().plain_contrast();
            let env_y_max = match num { 0 => 20000.0, _ => 1.0 };
            let tbox_uiid = 77 + (num * 2) as u64;
            let env_uiid = tbox_uiid + 1u64;
            let env_skew_y = match num { 0 => 3.0, _ => 1.0 };

            // Draw a TextBox. text_box(UIID, &mut String, FontSize)
            TextBox::new(tbox_uiid, text)
                .font_size(20)
                .dimensions(dim[0], text_box_height - 10.0)
                .point(pos)
                .frame(demo.frame_width)
                .frame_color(demo.bg_color.invert().plain_contrast())
                .color(demo.bg_color.invert())
                .callback(|_string: &mut String|{})
                .draw(ui, gl);

            // Draw an EnvelopeEditor.
            EnvelopeEditor::new(env_uiid, // UIID
                                env, // vector of `E: EnvelopePoint`s.
                                0.0, 1.0, 0.0, env_y_max) // x_min, x_max, y_min, y_max.
                .dimensions(dim[0], env_editor_height - 10.0)
                .point(env_editor_pos)
                .skew_y(env_skew_y)
                .color(demo.bg_color.invert())
                .frame(demo.frame_width)
                .frame_color(demo.bg_color.invert().plain_contrast())
                .label(&text)
                .label_color(env_label_color)
                .point_radius(6.0)
                .line_width(2.0)
                .callback(|_points: &mut Vec<Point>, _idx: usize|{})
                .draw(ui, gl);

        }); // End of matrix widget callback.

}

/// Draw a circle controlled by the XYPad.
fn draw_circle(win_w: f64,
               win_h: f64,
               gl: &mut GlGraphics,
               pos: Point,
               color: Color) {
    let Color(col) = color;
    graphics::Ellipse::new(col)
        .draw(
            [pos[0], pos[1], 30.0, 30.0],
            graphics::default_draw_state(),
            graphics::abs_transform(win_w, win_h),
            gl
        );
}
