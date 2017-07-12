//! This module is used for sharing a few items between the `all_widgets.rs`, `glutin_glium.rs` and
//! `glutin_gfx.rs` examples.
//!
//! The module contains:
//!
//! - `pub struct DemoApp` as a demonstration of some state we want to change.
//! - `pub fn gui` as a demonstration of all widgets, some of which mutate our `DemoApp`.
//! - `pub struct Ids` - a set of all `widget::Id`s used in the `gui` fn.
//!
//! By sharing these items between these examples, we can test and ensure that the different events
//! and drawing backends behave in the same manner.
#![allow(dead_code)]

extern crate rand;

use conrod;
use std;

#[cfg(feature="glium")] use conrod::backend::glium::glium;


pub const WIN_W: u32 = 600;
pub const WIN_H: u32 = 420;


/// A demonstration of some application state we want to control with a conrod GUI.
pub struct DemoApp {
    ball_xy: conrod::Point,
    ball_color: conrod::Color,
    sine_frequency: f32,
    rust_logo: conrod::image::Id,
}


impl DemoApp {

    /// Simple constructor for the `DemoApp`.
    pub fn new(rust_logo: conrod::image::Id) -> Self {
        DemoApp {
            ball_xy: [0.0, 0.0],
            ball_color: conrod::color::WHITE,
            sine_frequency: 1.0,
            rust_logo: rust_logo,
        }
    }

}


/// A set of reasonable stylistic defaults that works for the `gui` below.
pub fn theme() -> conrod::Theme {
    use conrod::position::{Align, Direction, Padding, Position, Relative};
    conrod::Theme {
        name: "Demo Theme".to_string(),
        padding: Padding::none(),
        x_position: Position::Relative(Relative::Align(Align::Start), None),
        y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
        background_color: conrod::color::DARK_CHARCOAL,
        shape_color: conrod::color::LIGHT_CHARCOAL,
        border_color: conrod::color::BLACK,
        border_width: 0.0,
        label_color: conrod::color::WHITE,
        font_id: None,
        font_size_large: 26,
        font_size_medium: 18,
        font_size_small: 12,
        widget_styling: conrod::theme::StyleMap::default(),
        mouse_drag_threshold: 0.0,
        double_click_threshold: std::time::Duration::from_millis(500),
    }
}


// Generate a unique `WidgetId` for each widget.
widget_ids! {
    pub struct Ids {

        // The scrollable canvas.
        canvas,

        // The title and introduction widgets.
        title,
        introduction,

        // Shapes.
        shapes_canvas,
        rounded_rectangle,
        shapes_left_col,
        shapes_right_col,
        shapes_title,
        line,
        point_path,
        rectangle_fill,
        rectangle_outline,
        trapezoid,
        oval_fill,
        oval_outline,
        circle,

        // Image.
        image_title,
        rust_logo,

        // Button, XyPad, Toggle.
        button_title,
        button,
        xy_pad,
        toggle,
        ball,

        // NumberDialer, PlotPath
        dialer_title,
        number_dialer,
        plot_path,

        // Scrollbar
        canvas_scrollbar,

    }
}


/// Instantiate a GUI demonstrating every widget available in conrod.
pub fn gui(ui: &mut conrod::UiCell, ids: &Ids, app: &mut DemoApp) {
    use conrod::{widget, Colorable, Labelable, Positionable, Sizeable, Widget};
    use std::iter::once;

    const MARGIN: conrod::Scalar = 30.0;
    const SHAPE_GAP: conrod::Scalar = 50.0;
    const TITLE_SIZE: conrod::FontSize = 42;
    const SUBTITLE_SIZE: conrod::FontSize = 32;

    // `Canvas` is a widget that provides some basic functionality for laying out children widgets.
    // By default, its size is the size of the window. We'll use this as a background for the
    // following widgets, as well as a scrollable container for the children widgets.
    const TITLE: &'static str = "All Widgets";
    widget::Canvas::new().pad(MARGIN).scroll_kids_vertically().set(ids.canvas, ui);


    ////////////////
    ///// TEXT /////
    ////////////////


    // We'll demonstrate the `Text` primitive widget by using it to draw a title and an
    // introduction to the example.
    widget::Text::new(TITLE).font_size(TITLE_SIZE).mid_top_of(ids.canvas).set(ids.title, ui);

    const INTRODUCTION: &'static str =
        "This example aims to demonstrate all widgets that are provided by conrod.\
        \n\nThe widget that you are currently looking at is the Text widget. The Text widget \
        is one of several special \"primitive\" widget types which are used to construct \
        all other widget types. These types are \"special\" in the sense that conrod knows \
        how to render them via `conrod::render::Primitive`s.\
        \n\nScroll down to see more widgets!";
    widget::Text::new(INTRODUCTION)
        .padded_w_of(ids.canvas, MARGIN)
        .down(60.0)
        .align_middle_x_of(ids.canvas)
        .center_justify()
        .line_spacing(5.0)
        .set(ids.introduction, ui);


    ////////////////////////////
    ///// Lines and Shapes /////
    ////////////////////////////


    widget::Text::new("Lines and Shapes")
        .down(70.0)
        .align_middle_x_of(ids.canvas)
        .font_size(SUBTITLE_SIZE)
        .set(ids.shapes_title, ui);

    // Lay out the shapes in two horizontal columns.
    //
    // TODO: Have conrod provide an auto-flowing, fluid-list widget that is more adaptive for these
    // sorts of situations.
    widget::Canvas::new()
        .down(0.0)
        .align_middle_x_of(ids.canvas)
        .kid_area_w_of(ids.canvas)
        .h(360.0)
        .color(conrod::color::TRANSPARENT)
        .pad(MARGIN)
        .flow_down(&[
            (ids.shapes_left_col, widget::Canvas::new()),
            (ids.shapes_right_col, widget::Canvas::new()),
        ])
        .set(ids.shapes_canvas, ui);

    let shapes_canvas_rect = ui.rect_of(ids.shapes_canvas).unwrap();
    let w = shapes_canvas_rect.w();
    let h = shapes_canvas_rect.h() * 5.0 / 6.0;
    let radius = 10.0;
    widget::RoundedRectangle::fill([w, h], radius)
        .color(conrod::color::CHARCOAL.alpha(0.25))
        .middle_of(ids.shapes_canvas)
        .set(ids.rounded_rectangle, ui);

    let start = [-40.0, -40.0];
    let end = [40.0, 40.0];
    widget::Line::centred(start, end).mid_left_of(ids.shapes_left_col).set(ids.line, ui);

    let left = [-40.0, -40.0];
    let top = [0.0, 40.0];
    let right = [40.0, -40.0];
    let points = once(left).chain(once(top)).chain(once(right));
    widget::PointPath::centred(points).right(SHAPE_GAP).set(ids.point_path, ui);

    widget::Rectangle::fill([80.0, 80.0]).right(SHAPE_GAP).set(ids.rectangle_fill, ui);

    widget::Rectangle::outline([80.0, 80.0]).right(SHAPE_GAP).set(ids.rectangle_outline, ui);

    let bl = [-40.0, -40.0];
    let tl = [-20.0, 40.0];
    let tr = [20.0, 40.0];
    let br = [40.0, -40.0];
    let points = once(bl).chain(once(tl)).chain(once(tr)).chain(once(br));
    widget::Polygon::centred_fill(points).mid_left_of(ids.shapes_right_col).set(ids.trapezoid, ui);

    widget::Oval::fill([40.0, 80.0]).right(SHAPE_GAP + 20.0).align_middle_y().set(ids.oval_fill, ui);

    widget::Oval::outline([80.0, 40.0]).right(SHAPE_GAP + 20.0).align_middle_y().set(ids.oval_outline, ui);

    widget::Circle::fill(40.0).right(SHAPE_GAP).align_middle_y().set(ids.circle, ui);


    /////////////////
    ///// Image /////
    /////////////////


    widget::Text::new("Image")
        .down_from(ids.shapes_canvas, MARGIN)
        .align_middle_x_of(ids.canvas)
        .font_size(SUBTITLE_SIZE)
        .set(ids.image_title, ui);

    const LOGO_SIDE: conrod::Scalar = 144.0;
    widget::Image::new(app.rust_logo)
        .w_h(LOGO_SIDE, LOGO_SIDE)
        .down(60.0)
        .align_middle_x_of(ids.canvas)
        .set(ids.rust_logo, ui);


    /////////////////////////////////
    ///// Button, XYPad, Toggle /////
    /////////////////////////////////


    widget::Text::new("Button, XYPad and Toggle")
        .down_from(ids.rust_logo, 60.0)
        .align_middle_x_of(ids.canvas)
        .font_size(SUBTITLE_SIZE)
        .set(ids.button_title, ui);

    let ball_x_range = ui.kid_area_of(ids.canvas).unwrap().w();
    let ball_y_range = ui.h_of(ui.window).unwrap() * 0.5;
    let min_x = -ball_x_range / 3.0;
    let max_x = ball_x_range / 3.0;
    let min_y = -ball_y_range / 3.0;
    let max_y = ball_y_range / 3.0;
    let side = 130.0;

    for _press in widget::Button::new()
        .label("PRESS ME")
        .mid_left_with_margin_on(ids.canvas, MARGIN)
        .down_from(ids.button_title, 60.0)
        .w_h(side, side)
        .set(ids.button, ui)
    {
        let x = rand::random::<conrod::Scalar>() * (max_x - min_x) - max_x;
        let y = rand::random::<conrod::Scalar>() * (max_y - min_y) - max_y;
        app.ball_xy = [x, y];
    }

    for (x, y) in widget::XYPad::new(app.ball_xy[0], min_x, max_x,
                                     app.ball_xy[1], min_y, max_y)
        .label("BALL XY")
        .wh_of(ids.button)
        .align_middle_y_of(ids.button)
        .align_middle_x_of(ids.canvas)
        .parent(ids.canvas)
        .set(ids.xy_pad, ui)
    {
        app.ball_xy = [x, y];
    }

    let is_white = app.ball_color == conrod::color::WHITE;
    let label = if is_white { "WHITE" } else { "BLACK" };
    for is_white in widget::Toggle::new(is_white)
        .label(label)
        .label_color(if is_white { conrod::color::WHITE } else { conrod::color::LIGHT_CHARCOAL })
        .mid_right_with_margin_on(ids.canvas, MARGIN)
        .align_middle_y_of(ids.button)
        .set(ids.toggle, ui)
    {
        app.ball_color = if is_white { conrod::color::WHITE } else { conrod::color::BLACK };
    }

    let ball_x = app.ball_xy[0];
    let ball_y = app.ball_xy[1] - max_y - side * 0.5 - MARGIN;
    widget::Circle::fill(20.0)
        .color(app.ball_color)
        .x_y_relative_to(ids.xy_pad, ball_x, ball_y)
        .set(ids.ball, ui);


    //////////////////////////////////
    ///// NumberDialer, PlotPath /////
    //////////////////////////////////


    widget::Text::new("NumberDialer and PlotPath")
        .down_from(ids.xy_pad, max_y - min_y + side * 0.5 + MARGIN)
        .align_middle_x_of(ids.canvas)
        .font_size(SUBTITLE_SIZE)
        .set(ids.dialer_title, ui);

    // Use a `NumberDialer` widget to adjust the frequency of the sine wave below.
    let min = 0.5;
    let max = 200.0;
    let decimal_precision = 1;
    for new_freq in widget::NumberDialer::new(app.sine_frequency, min, max, decimal_precision)
        .down(60.0)
        .align_middle_x_of(ids.canvas)
        .w_h(160.0, 40.0)
        .label("F R E Q")
        .set(ids.number_dialer, ui)
    {
        app.sine_frequency = new_freq;
    }

    // Use the `PlotPath` widget to display a sine wave.
    let min_x = 0.0;
    let max_x = std::f32::consts::PI * 2.0 * app.sine_frequency;
    let min_y = -1.0;
    let max_y = 1.0;
    widget::PlotPath::new(min_x, max_x, min_y, max_y, f32::sin)
        .kid_area_w_of(ids.canvas)
        .h(240.0)
        .down(60.0)
        .align_middle_x_of(ids.canvas)
        .set(ids.plot_path, ui);


    /////////////////////
    ///// Scrollbar /////
    /////////////////////


    widget::Scrollbar::y_axis(ids.canvas).auto_hide(true).set(ids.canvas_scrollbar, ui);

}


/// In most of the examples the `glutin` crate is used for providing the window context and
/// events while the `glium` crate is used for displaying `conrod::render::Primitives` to the
/// screen.
/// 
/// This `Iterator`-like type simplifies some of the boilerplate involved in setting up a
/// glutin+glium event loop that works efficiently with conrod.
#[cfg(feature="glium")]
pub struct EventLoop {
    ui_needs_update: bool,
    last_update: std::time::Instant,
}

#[cfg(feature="glium")]
impl EventLoop {

    pub fn new() -> Self {
        EventLoop {
            last_update: std::time::Instant::now(),
            ui_needs_update: true,
        }
    }

    /// Produce an iterator yielding all available events.
    pub fn next(&mut self, events_loop: &mut glium::glutin::EventsLoop) -> Vec<glium::glutin::Event> {
        // We don't want to loop any faster than 60 FPS, so wait until it has been at least 16ms
        // since the last yield.
        let last_update = self.last_update;
        let sixteen_ms = std::time::Duration::from_millis(16);
        let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }

        // Collect all pending events.
        let mut events = Vec::new();
        events_loop.poll_events(|event| events.push(event));

        // If there are no events and the `Ui` does not need updating, wait for the next event.
        if events.is_empty() && !self.ui_needs_update {
            events_loop.run_forever(|event| {
                events.push(event);
                glium::glutin::ControlFlow::Break
            });
        }

        self.ui_needs_update = false;
        self.last_update = std::time::Instant::now();

        events
    }

    /// Notifies the event loop that the `Ui` requires another update whether or not there are any
    /// pending events.
    ///
    /// This is primarily used on the occasion that some part of the `Ui` is still animating and
    /// requires further updates to do so.
    pub fn needs_update(&mut self) {
        self.ui_needs_update = true;
    }

}
