//!
//! A demonstration of all non-primitive widgets available in Conrod.
//!
//!
//! Don't be put off by the number of method calls, they are only for demonstration and almost all
//! of them are optional. Conrod supports `Theme`s, so if you don't give it an argument, it will
//! check the current `Theme` within the `Ui` and retrieve defaults from there.
//!

#[macro_use]
extern crate conrod_core;
extern crate conrod_glium;
#[macro_use]
extern crate conrod_winit;
extern crate find_folder;
extern crate glium;
extern crate rand; // for making a random color.

mod support;

use glium::Surface;

/// This struct holds all of the variables used to demonstrate application data being passed
/// through the widgets. If some of these seem strange, that's because they are! Most of these
/// simply represent the aesthetic state of different parts of the GUI to offer visual feedback
/// during interaction with the widgets.
struct DemoApp {
    /// Background color (for demonstration of button and sliders).
    bg_color: conrod_core::Color,
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
    /// The widget border width (we'll use this to demo Bordering
    /// and number_dialer).
    border_width: f64,
    /// Bool matrix for widget_matrix demonstration.
    bool_matrix: [[bool; 8]; 8],
    /// A vector of strings for drop_down_list demonstration.
    ddl_colors: Vec<String>,
    /// The currently selected DropDownList color.
    ddl_color: conrod_core::Color,
    /// We also need an Option<idx> to indicate whether or not an
    /// item is selected.
    selected_idx: Option<usize>,
    /// Co-ordinates for a little circle used to demonstrate the
    /// xy_pad.
    circle_pos: conrod_core::Point,
    /// Envelope for demonstration of EnvelopeEditor.
    envelopes: Vec<(Vec<conrod_core::Point>, String)>,
}

impl DemoApp {
    /// Constructor for the Demonstration Application model.
    fn new() -> DemoApp {
        DemoApp {
            bg_color: conrod_core::color::rgb(0.2, 0.35, 0.45),
            show_button: false,
            toggle_label: "OFF".to_string(),
            title_pad: 350.0,
            v_slider_height: 230.0,
            border_width: 1.0,
            bool_matrix: [
                [true, true, true, true, true, true, true, true],
                [true, false, false, false, false, false, false, true],
                [true, false, true, false, true, true, true, true],
                [true, false, true, false, true, true, true, true],
                [true, false, false, false, true, true, true, true],
                [true, true, true, true, true, true, true, true],
                [true, true, false, true, false, false, false, true],
                [true, true, true, true, true, true, true, true],
            ],
            ddl_colors: vec![
                "Black".to_string(),
                "White".to_string(),
                "Red".to_string(),
                "Green".to_string(),
                "Blue".to_string(),
            ],
            ddl_color: conrod_core::color::PURPLE,
            selected_idx: None,
            circle_pos: [-50.0, 110.0],
            envelopes: vec![
                (
                    vec![
                        [0.0, 0.0],
                        [0.1, 17000.0],
                        [0.25, 8000.0],
                        [0.5, 2000.0],
                        [1.0, 0.0],
                    ],
                    "Envelope A".to_string(),
                ),
                (
                    vec![[0.0, 0.85], [0.3, 0.2], [0.6, 0.6], [1.0, 0.0]],
                    "Envelope B".to_string(),
                ),
            ],
        }
    }
}

fn main() {
    const WIDTH: u32 = 1100;
    const HEIGHT: u32 = 560;

    // Build the window.
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let window = glium::glutin::window::WindowBuilder::new()
        .with_title("Widget Demonstration")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &event_loop).unwrap();

    // construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Identifiers used for instantiating our widgets.
    let mut ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // A type used for converting `conrod_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod_glium::Renderer::new(&display).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();

    // Our demonstration app that we'll control with our GUI.
    let mut app = DemoApp::new();

    // Poll events from the window.
    support::run_loop(display, event_loop, move |request, display| {
        match request {
            support::Request::Event {
                event,
                should_update_ui,
                should_exit,
            } => {
                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = support::convert_event(&event, &display.gl_window().window()) {
                    ui.handle_event(event);
                    *should_update_ui = true;
                }

                match event {
                    glium::glutin::event::Event::WindowEvent { event, .. } => match event {
                        // Break from the loop upon `Escape`.
                        glium::glutin::event::WindowEvent::CloseRequested
                        | glium::glutin::event::WindowEvent::KeyboardInput {
                            input:
                                glium::glutin::event::KeyboardInput {
                                    virtual_keycode:
                                        Some(glium::glutin::event::VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *should_exit = true,
                        _ => {}
                    },
                    _ => {}
                }
            }
            support::Request::SetUi { needs_redraw } => {
                // We'll set all our widgets in a single function called `set_widgets`.
                let mut ui = ui.set_widgets();
                set_widgets(&mut ui, &mut app, &mut ids);

                *needs_redraw = ui.has_changed();
            }
            support::Request::Redraw => {
                // Render the `Ui` and then display it on the screen.
                let primitives = ui.draw();

                renderer.fill(display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    })
}

// In conrod, each widget must have its own unique identifier so that the `Ui` can keep track of
// its state between updates.
//
// To make this easier, conrod provides the `widget_ids` macro. This macro generates a new type
// with a unique `widget::Id` field for each identifier given in the list. See the `widget_ids!`
// documentation for more details.
widget_ids! {
    struct Ids {
        canvas,
        canvas_x_scrollbar,
        canvas_y_scrollbar,
        title,
        button,
        title_pad_slider,
        toggle,
        red_slider,
        green_slider,
        blue_slider,
        slider_height,
        border_width,
        toggle_matrix,
        color_select,
        circle_position,
        circle,
        text_box_a,
        text_box_b,
        envelope_editor_a,
        envelope_editor_b,
    }
}

/// Set all `Widget`s within the User Interface.
///
/// The first time this gets called, each `Widget`'s `State` will be initialised and cached within
/// the `Ui` at their given indices. Every other time this get called, the `Widget`s will avoid any
/// allocations by updating the pre-existing cached state. A new graphical `Element` is only
/// retrieved from a `Widget` in the case that it's `State` has changed in some way.
fn set_widgets(ui: &mut conrod_core::UiCell, app: &mut DemoApp, ids: &mut Ids) {
    use conrod_core::{
        color, widget, Borderable, Colorable, Labelable, Positionable, Sizeable, Widget,
    };

    // We can use this `Canvas` as a parent Widget upon which we can place other widgets.
    widget::Canvas::new()
        .border(app.border_width)
        .pad(30.0)
        .color(app.bg_color)
        .scroll_kids()
        .set(ids.canvas, ui);
    widget::Scrollbar::x_axis(ids.canvas)
        .auto_hide(true)
        .set(ids.canvas_y_scrollbar, ui);
    widget::Scrollbar::y_axis(ids.canvas)
        .auto_hide(true)
        .set(ids.canvas_x_scrollbar, ui);

    // Text example.
    widget::Text::new("Widget Demonstration")
        .top_left_with_margins_on(ids.canvas, 0.0, app.title_pad)
        .font_size(32)
        .color(app.bg_color.plain_contrast())
        .set(ids.title, ui);

    if app.show_button {
        // Button widget example button.
        if widget::Button::new()
            .w_h(200.0, 50.0)
            .mid_left_of(ids.canvas)
            .down_from(ids.title, 45.0)
            .rgb(0.4, 0.75, 0.6)
            .border(app.border_width)
            .label("PRESS")
            .set(ids.button, ui)
            .was_clicked()
        {
            app.bg_color = color::rgb(rand::random(), rand::random(), rand::random())
        }
    }
    // Horizontal slider example.
    else {
        // Create the label for the slider.
        let label = format!("Padding: {}", app.title_pad as i16);

        // Slider widget example slider(value, min, max).
        if let Some(new_pad) = widget::Slider::new(app.title_pad, 0.0, 670.0)
            .w_h(200.0, 50.0)
            .mid_left_of(ids.canvas)
            .down_from(ids.title, 45.0)
            .rgb(0.5, 0.3, 0.6)
            .border(app.border_width)
            .label(&label)
            .label_color(color::WHITE)
            .set(ids.title_pad_slider, ui)
        {
            app.title_pad = new_pad;
        }
    }

    // Keep track of the currently shown widget.
    let shown_widget = if app.show_button {
        ids.button
    } else {
        ids.title_pad_slider
    };

    // Toggle widget example.
    if let Some(value) = widget::Toggle::new(app.show_button)
        .w_h(75.0, 75.0)
        .down(20.0)
        .rgb(0.6, 0.25, 0.75)
        .border(app.border_width)
        .label(&app.toggle_label)
        .label_color(color::WHITE)
        .set(ids.toggle, ui)
        .last()
    {
        app.show_button = value;
        app.toggle_label = match value {
            true => "ON".to_string(),
            false => "OFF".to_string(),
        }
    }

    macro_rules! color_slider {
        ($slider_id:ident, $bg_color:ident, $color:expr, $set_color:ident, $position:ident) => {{
            let value = app.bg_color.$bg_color();
            let label = format!("{:.*}", 2, value);
            for color in widget::Slider::new(value, 0.0, 1.0)
                .$position(25.0)
                .w_h(40.0, app.v_slider_height)
                .color($color)
                .border(app.border_width)
                .label(&label)
                .label_color(color::WHITE)
                .set(ids.$slider_id, ui)
            {
                app.bg_color.$set_color(color);
            }
        }};
    }

    color_slider!(red_slider, red, color::rgb(0.75, 0.3, 0.3), set_red, down);
    color_slider!(
        green_slider,
        green,
        color::rgb(0.3, 0.75, 0.3),
        set_green,
        right
    );
    color_slider!(
        blue_slider,
        blue,
        color::rgb(0.3, 0.3, 0.75),
        set_blue,
        right
    );

    // Number Dialer widget example. (value, min, max, precision)
    for new_height in widget::NumberDialer::new(app.v_slider_height, 25.0, 250.0, 1)
        .w_h(260.0, 60.0)
        .right_from(shown_widget, 30.0)
        .color(app.bg_color.invert())
        .border(app.border_width)
        .label("Height (px)")
        .label_color(app.bg_color.invert().plain_contrast())
        .set(ids.slider_height, ui)
    {
        app.v_slider_height = new_height;
    }

    // Number Dialer widget example. (value, min, max, precision)
    for new_width in widget::NumberDialer::new(app.border_width, 0.0, 15.0, 2)
        .w_h(260.0, 60.0)
        .down(20.0)
        .color(app.bg_color.plain_contrast().invert())
        .border(app.border_width)
        .border_color(app.bg_color.plain_contrast())
        .label("Border Width (px)")
        .label_color(app.bg_color.plain_contrast())
        .set(ids.border_width, ui)
    {
        app.border_width = new_width;
    }

    // A demonstration using widget_matrix to easily draw a matrix of any kind of widget.
    let (cols, rows) = (8, 8);
    let mut elements = widget::Matrix::new(cols, rows)
        .down(20.0)
        .w_h(260.0, 260.0)
        .set(ids.toggle_matrix, ui);

    // The `Matrix` widget returns an `Elements`, which can be used similar to an `Iterator`.
    while let Some(elem) = elements.next(ui) {
        let (col, row) = (elem.col, elem.row);

        // Color effect for fun.
        let (r, g, b, a) = (
            0.5 + (elem.col as f32 / cols as f32) / 2.0,
            0.75,
            1.0 - (elem.row as f32 / rows as f32) / 2.0,
            1.0,
        );

        // We can use `Element`s to instantiate any kind of widget we like.
        // The `Element` does all of the positioning and sizing work for us.
        // Here, we use the `Element` to `set` a `Toggle` widget for us.
        let toggle = widget::Toggle::new(app.bool_matrix[col][row])
            .rgba(r, g, b, a)
            .border(app.border_width);
        if let Some(new_value) = elem.set(toggle, ui).last() {
            app.bool_matrix[col][row] = new_value;
        }
    }

    // A demonstration using a DropDownList to select its own color.
    for selected_idx in widget::DropDownList::new(&app.ddl_colors, app.selected_idx)
        .w_h(150.0, 40.0)
        .right_from(ids.slider_height, 30.0) // Position right from widget 6 by 50 pixels.
        .max_visible_items(3)
        .color(app.ddl_color)
        .border(app.border_width)
        .border_color(app.ddl_color.plain_contrast())
        .label("Colors")
        .label_color(app.ddl_color.plain_contrast())
        .scrollbar_on_top()
        .set(ids.color_select, ui)
    {
        app.selected_idx = Some(selected_idx);
        app.ddl_color = match &app.ddl_colors[selected_idx][..] {
            "Black" => color::BLACK,
            "White" => color::WHITE,
            "Red" => color::RED,
            "Green" => color::GREEN,
            "Blue" => color::BLUE,
            _ => color::PURPLE,
        }
    }

    // Draw an xy_pad.
    for (x, y) in widget::XYPad::new(
        app.circle_pos[0],
        -75.0,
        75.0, // x range.
        app.circle_pos[1],
        95.0,
        245.0,
    ) // y range.
    .w_h(150.0, 150.0)
    .right_from(ids.toggle_matrix, 30.0)
    .align_bottom_of(ids.toggle_matrix) // Align to the bottom of the last toggle_matrix element.
    .color(app.ddl_color)
    .border(app.border_width)
    .border_color(color::WHITE)
    .label("Circle Position")
    .label_color(app.ddl_color.plain_contrast().alpha(0.5))
    .line_thickness(2.0)
    .set(ids.circle_position, ui)
    {
        app.circle_pos[0] = x;
        app.circle_pos[1] = y;
    }

    // Draw a circle at the app's circle_pos.
    widget::Circle::fill(15.0)
        .xy_relative_to(ids.circle_position, app.circle_pos)
        .color(app.ddl_color)
        .set(ids.circle, ui);

    // Draw two TextBox and EnvelopeEditor pairs to the right of the DropDownList flowing downward.
    for i in 0..2 {
        let &mut (ref mut env, ref mut text) = &mut app.envelopes[i];
        let (text_box, env_editor, env_y_max, env_skew_y) = match i {
            0 => (ids.text_box_a, ids.envelope_editor_a, 20_000.0, 3.0),
            1 => (ids.text_box_b, ids.envelope_editor_b, 1.0, 1.0),
            _ => unreachable!(),
        };

        // A text box in which we can mutate a single line of text, and trigger reactions via the
        // `Enter`/`Return` key.
        for event in widget::TextBox::new(text)
            .and_if(i == 0, |text| text.right_from(ids.color_select, 30.0))
            .font_size(20)
            .w_h(320.0, 40.0)
            .border(app.border_width)
            .border_color(app.bg_color.invert().plain_contrast())
            .color(app.bg_color.invert())
            .set(text_box, ui)
        {
            match event {
                widget::text_box::Event::Enter => println!("TextBox {}: {:?}", i, text),
                widget::text_box::Event::Update(string) => *text = string,
            }
        }

        // Draw an EnvelopeEditor. (&[Point], x_min, x_max, y_min, y_max).
        for event in widget::EnvelopeEditor::new(env, 0.0, 1.0, 0.0, env_y_max)
            .down(10.0)
            .w_h(320.0, 150.0)
            .skew_y(env_skew_y)
            .color(app.bg_color.invert())
            .border(app.border_width)
            .border_color(app.bg_color.invert().plain_contrast())
            .label(&text)
            .label_color(app.bg_color.invert().plain_contrast().alpha(0.5))
            .point_radius(6.0)
            .line_thickness(2.0)
            .set(env_editor, ui)
        {
            event.update(env);
        }
    }
}
