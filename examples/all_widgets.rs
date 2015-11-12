//!
//!
//! A demonstration of all widgets available in Conrod.
//!
//!
//! Don't be put off by the number of method calls, they are only for demonstration and almost all
//! of them are optional. Conrod supports `Theme`s, so if you don't give it an argument, it will
//! check the current `Theme` within the `Ui` and retrieve defaults from there.
//!
//!


#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate vecmath;

use conrod::{
    Button,
    Color,
    Colorable,
    DropDownList,
    EnvelopeEditor,
    Frameable,
    Label,
    Labelable,
    NumberDialer,
    Point,
    Positionable,
    Slider,
    Sizeable,
    Split,
    TextBox,
    Theme,
    Toggle,
    Widget,
    WidgetMatrix,
    XYPad,
};
use conrod::color::{self, rgb, white, black, red, green, blue, purple};
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use opengl_graphics::glyph_cache::GlyphCache;
use piston::event_loop::{Events, EventLoop};
use piston::input::{RenderEvent};
use piston::window::{WindowSettings, Size};


type Ui = conrod::Ui<GlyphCache<'static>>;

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
    title_pad: f64,
    /// The height of the vertical sliders (we will play with this
    /// using a number_dialer).
    v_slider_height: f64,
    /// The widget frame width (we'll use this to demo Framing
    /// and number_dialer).
    frame_width: f64,
    /// Bool matrix for widget_matrix demonstration.
    bool_matrix: [[bool; 8]; 8],
    /// A vector of strings for drop_down_list demonstration.
    ddl_colors: Vec<String>,
    /// The currently selected DropDownList color.
    ddl_color: Color,
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

    /// Constructor for the Demonstration Application model.
    fn new() -> DemoApp {
        DemoApp {
            bg_color: rgb(0.2, 0.35, 0.45),
            show_button: false,
            toggle_label: "OFF".to_string(),
            title_pad: 350.0,
            v_slider_height: 230.0,
            frame_width: 1.0,
            bool_matrix: [ [true, true, true, true, true, true, true, true],
                           [true, false, false, false, false, false, false, true],
                           [true, false, true, false, true, true, true, true],
                           [true, false, true, false, true, true, true, true],
                           [true, false, false, false, true, true, true, true],
                           [true, true, true, true, true, true, true, true],
                           [true, true, false, true, false, false, false, true],
                           [true, true, true, true, true, true, true, true] ],
            ddl_colors: vec!["Black".to_string(),
                              "White".to_string(),
                              "Red".to_string(),
                              "Green".to_string(),
                              "Blue".to_string()],
            ddl_color: purple(),
            selected_idx: None,
            circle_pos: [560.0, 310.0],
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
    let opengl = OpenGL::V3_2;
    let window: GlutinWindow =
        WindowSettings::new(
            "Hello Conrod".to_string(),
            Size { width: 1100, height: 550 }
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .samples(4)
        .build()
        .unwrap();
    let event_iter = window.events().ups(60).max_fps(60);
    let mut gl = GlGraphics::new(opengl);

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    let theme = Theme::default();
    let glyph_cache = GlyphCache::new(&font_path).unwrap();
    let mut ui = Ui::new(glyph_cache, theme);
    let mut demo = DemoApp::new();

    for event in event_iter {
        ui.handle_event(&event);
        if let Some(args) = event.render_args() {
            gl.draw(args.viewport(), |c, gl| {

                // We'll set all our widgets in a single function called `set_widgets`.
                // At the moment conrod requires that we set our widgets in the Render loop,
                // however soon we'll add support so that you can set your Widgets at any arbitrary
                // update rate.
                set_widgets(&mut ui, &mut demo);

                // Draw our Ui!
                //
                // The `draw_if_changed` method only re-draws the GUI if some `Widget`'s `Element`
                // representation has changed. Normally, a `Widget`'s `Element` should only change
                // if a Widget was interacted with in some way, however this is up to the `Widget`
                // designer's discretion.
                //
                // If instead you need to re-draw your conrod GUI every frame, use `Ui::draw`.
                ui.draw_if_changed(c, gl);

                // Draw the circle that's controlled by our XYPad.
                graphics::Ellipse::new(demo.ddl_color.to_fsa())
                    .draw([demo.circle_pos[0], demo.circle_pos[1], 30.0, 30.0],
                          &c.draw_state, c.transform, gl);
            });
        }
    }
}



/// Set all `Widget`s within the User Interface.
///
/// The first time this gets called, each `Widget`'s `State` will be initialised and cached within
/// the `Ui` at their given indices. Every other time this get called, the `Widget`s will avoid any
/// allocations by updating the pre-existing cached state. A new graphical `Element` is only
/// retrieved from a `Widget` in the case that it's `State` has changed in some way.
fn set_widgets(ui: &mut Ui, demo: &mut DemoApp) {

    // Normally, `Split`s can be used to describe the layout of `Canvas`ses within a window (see
    // the canvas.rs example for a demonstration of this). However, when only one `Split` is used
    // (as in this case) a single `Canvas` will simply fill the screen.
    // We can use this `Canvas` as a parent Widget upon which we can place other widgets.
    Split::new(CANVAS).frame(demo.frame_width).color(demo.bg_color).scrolling(true).set(ui);

    // Calculate x and y coords for title (temporary until `Canvas`es are implemented, see #380).
    let title_x = demo.title_pad - (ui.win_w / 2.0) + 185.0;
    let title_y = (ui.win_h / 2.0) - 50.0;

    // Label example.
    Label::new("Widget Demonstration")
        .xy(title_x, title_y)
        .font_size(32)
        .color(demo.bg_color.plain_contrast())
        .parent(Some(CANVAS))
        .set(TITLE, ui);

    if demo.show_button {

        // Button widget example button.
        Button::new()
            .dimensions(200.0, 50.0)
            .xy(140.0 - (ui.win_w / 2.0), title_y - 70.0)
            .rgb(0.4, 0.75, 0.6)
            .frame(demo.frame_width)
            .label("PRESS")
            .react(|| demo.bg_color = color::random())
            .set(BUTTON, ui)

    }

    // Horizontal slider example.
    else {

        // Create the label for the slider.
        let pad = demo.title_pad as i16;
        let pad_string = pad.to_string();
        let label = {
            let mut text = "Padding: ".to_string();
            text.push_str(&pad_string);
            text
        };

        // Slider widget example slider(value, min, max).
        Slider::new(pad as f32, 30.0, 700.0)
            .dimensions(200.0, 50.0)
            .xy(140.0 - (ui.win_w / 2.0), title_y - 70.0)
            .rgb(0.5, 0.3, 0.6)
            .frame(demo.frame_width)
            .label(&label)
            .label_color(white())
            .react(|new_pad: f32| demo.title_pad = new_pad as f64)
            .set(TITLE_PAD_SLIDER, ui);

    }

    // Clone the label toggle to be drawn.
    let label = demo.toggle_label.clone();

    // Keep track of the currently shown widget.
    let shown_widget = if demo.show_button { BUTTON } else { TITLE_PAD_SLIDER };

    // Toggle widget example toggle(value).
    Toggle::new(demo.show_button)
        .dimensions(75.0, 75.0)
        .down(20.0)
        .rgb(0.6, 0.25, 0.75)
        .frame(demo.frame_width)
        .label(&label)
        .label_color(white())
        .react(|value| {
            demo.show_button = value;
            demo.toggle_label = match value {
                true => "ON".to_string(),
                false => "OFF".to_string()
            }
        })
        .set(TOGGLE, ui);

    // Let's draw a slider for each color element.
    // 0 => red, 1 => green, 2 => blue.
    for i in 0..3 {

        // We'll color the slider similarly to the color element which it will control.
        let color = match i {
            0 => rgb(0.75, 0.3, 0.3),
            1 => rgb(0.3, 0.75, 0.3),
            _ => rgb(0.3, 0.3, 0.75),
        };

        // Grab the value of the color element.
        let value = match i {
            0 => demo.bg_color.red(),
            1 => demo.bg_color.green(),
            _ => demo.bg_color.blue(),
        };

        // Create the label to be drawn with the slider.
        let mut label = value.to_string();
        if label.len() > 4 { label.truncate(4); }

        // Slider widget examples. slider(value, min, max)
        if i == 0 { Slider::new(value, 0.0, 1.0).down(25.0) }
        else      { Slider::new(value, 0.0, 1.0).right(20.0) }
            .dimensions(40.0, demo.v_slider_height)
            .color(color)
            .frame(demo.frame_width)
            .label(&label)
            .label_color(white())
            .react(|color| match i {
                0 => demo.bg_color.set_red(color),
                1 => demo.bg_color.set_green(color),
                _ => demo.bg_color.set_blue(color),
            })
            .set(COLOR_SLIDER + i, ui);

    }

    // Number Dialer widget example. (value, min, max, precision)
    NumberDialer::new(demo.v_slider_height, 25.0, 250.0, 1)
        .dimensions(260.0, 60.0)
        .right_from(shown_widget, 30.0)
        .color(demo.bg_color.invert())
        .frame(demo.frame_width)
        .label("Height (px)")
        .label_color(demo.bg_color.invert().plain_contrast())
        .react(|new_height| demo.v_slider_height = new_height)
        .set(SLIDER_HEIGHT, ui);

    // Number Dialer widget example. (value, min, max, precision)
    NumberDialer::new(demo.frame_width, 0.0, 15.0, 2)
        .dimensions(260.0, 60.0)
        .down(20.0)
        .color(demo.bg_color.invert().plain_contrast())
        .frame(demo.frame_width)
        .frame_color(demo.bg_color.plain_contrast())
        .label("Frame Width (px)")
        .label_color(demo.bg_color.plain_contrast())
        .react(|new_width| demo.frame_width = new_width)
        .set(FRAME_WIDTH, ui);

    // A demonstration using widget_matrix to easily draw
    // a matrix of any kind of widget.
    let (cols, rows) = (8, 8);
    WidgetMatrix::new(cols, rows)
        .down(20.0)
        .dimensions(260.0, 260.0) // matrix width and height.
        .each_widget(|_n, col: usize, row: usize| { // called for every matrix elem.

            // Color effect for fun.
            let (r, g, b, a) = (
                0.5 + (col as f32 / cols as f32) / 2.0,
                0.75,
                1.0 - (row as f32 / rows as f32) / 2.0,
                1.0
            );

            // Now return the widget we want to set in each element position.
            // You can return any type that implements `Widget`.
            // The returned widget will automatically be positioned and sized to the matrix
            // element's rectangle.
            let elem = &mut demo.bool_matrix[col][row];
            Toggle::new(*elem)
                .rgba(r, g, b, a)
                .frame(demo.frame_width)
                .react(move |new_val: bool| *elem = new_val)
        })
        .set(TOGGLE_MATRIX, ui);

    // A demonstration using a DropDownList to select its own color.
    let mut ddl_color = demo.ddl_color;
    DropDownList::new(&mut demo.ddl_colors, &mut demo.selected_idx)
        .dimensions(150.0, 40.0)
        .right_from(SLIDER_HEIGHT, 30.0) // Position right from widget 6 by 50 pixels.
        .max_visible_items(3)
        .color(ddl_color)
        .frame(demo.frame_width)
        .frame_color(ddl_color.plain_contrast())
        .label("Colors")
        .label_color(ddl_color.plain_contrast())
        .react(|selected_idx: &mut Option<usize>, new_idx, string: &str| {
            *selected_idx = Some(new_idx);
            ddl_color = match string {
                "Black" => black(),
                "White" => white(),
                "Red"   => red(),
                "Green" => green(),
                "Blue"  => blue(),
                _       => purple(),
            };
        })
        .set(COLOR_SELECT, ui);
    demo.ddl_color = ddl_color;

    // Draw an xy_pad.
    XYPad::new(demo.circle_pos[0], 550.0, 700.0, // x range.
               demo.circle_pos[1], 320.0, 170.0) // y range.
        .dimensions(150.0, 150.0)
        .right_from(TOGGLE_MATRIX, 30.0)
        .align_bottom() // Align to the bottom of the last TOGGLE_MATRIX element.
        .color(ddl_color)
        .frame(demo.frame_width)
        .frame_color(white())
        .label("Circle Position")
        .label_color(ddl_color.plain_contrast().alpha(0.5))
        .line_width(2.0)
        .react(|new_x, new_y| {
            demo.circle_pos[0] = new_x;
            demo.circle_pos[1] = new_y;
        })
        .set(CIRCLE_POSITION, ui);

    // Draw two TextBox and EnvelopeEditor pairs to the right of the DropDownList flowing downward.
    for i in 0..2 {

        let &mut (ref mut env, ref mut text) = &mut demo.envelopes[i];

        // Draw a TextBox. text_box(&mut String, FontSize)
        if i == 0 { TextBox::new(text).right_from(COLOR_SELECT, 30.0) }
        else      { TextBox::new(text) }
            .font_size(20)
            .dimensions(320.0, 40.0)
            .frame(demo.frame_width)
            .frame_color(demo.bg_color.invert().plain_contrast())
            .color(demo.bg_color.invert())
            .react(|_string: &mut String|{})
            .set(ENVELOPE_EDITOR + (i * 2), ui);

        let env_y_max = match i { 0 => 20_000.0, _ => 1.0 };
        let env_skew_y = match i { 0 => 3.0, _ => 1.0 };

        // Draw an EnvelopeEditor. (Vec<Point>, x_min, x_max, y_min, y_max).
        EnvelopeEditor::new(env, 0.0, 1.0, 0.0, env_y_max)
            .down(10.0)
            .dimensions(320.0, 150.0)
            .skew_y(env_skew_y)
            .color(demo.bg_color.invert())
            .frame(demo.frame_width)
            .frame_color(demo.bg_color.invert().plain_contrast())
            .label(&text)
            .label_color(demo.bg_color.invert().plain_contrast().alpha(0.5))
            .point_radius(6.0)
            .line_width(2.0)
            .react(|_points: &mut Vec<Point>, _idx: usize|{})
            .set(ENVELOPE_EDITOR + (i * 2) + 1, ui);

    }

}


// In conrod, each widget must have its own unique identifier so that the `Ui` can keep track of
// its state between updates.
// To make this easier, conrod provides the `widget_ids` macro, which generates a unique `WidgetId`
// for each identifier given in the list.
// The `with n` syntax reserves `n` number of WidgetIds for that identifier, rather than just one.
// This is often useful when you need to use an identifier in some kind of loop (i.e. like within
// the use of `WidgetMatrix` as above).
widget_ids! {
    CANVAS,
    TITLE,
    BUTTON,
    TITLE_PAD_SLIDER,
    TOGGLE,
    COLOR_SLIDER with 3,
    SLIDER_HEIGHT,
    FRAME_WIDTH,
    TOGGLE_MATRIX,
    COLOR_SELECT,
    CIRCLE_POSITION,
    ENVELOPE_EDITOR with 4
}
