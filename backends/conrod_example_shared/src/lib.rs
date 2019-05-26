//! This crate is used for sharing a few items between the conrod examples.
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

#[macro_use] extern crate conrod_core;
extern crate rand;

mod layout;
mod button_xy_pad_toggle;

use layout::*;

use conrod_core::{widget, Colorable, Labelable, Positionable, Rect, Sizeable, Ui, UiCell, Widget};
use std::iter::once;

pub const WIN_W: u32 = 600;
pub const WIN_H: u32 = 420;

/// A demonstration of some application state we want to control with a conrod GUI.
pub struct DemoApp {
    button_xy_pad_toggle: button_xy_pad_toggle::GuiState,
    sine_frequency: f32,
    rust_logo: conrod_core::image::Id,
}


impl DemoApp {
    /// Simple constructor for the `DemoApp`.
    pub fn new(rust_logo: conrod_core::image::Id) -> Self {
        DemoApp {
            button_xy_pad_toggle: button_xy_pad_toggle::GuiState::new(),
            sine_frequency: 1.0,
            rust_logo: rust_logo,
        }
    }
}

/// A set of reasonable stylistic defaults that works for the `gui` below.
pub fn theme() -> conrod_core::Theme {
    use conrod_core::position::{Align, Direction, Padding, Position, Relative};
    conrod_core::Theme {
        name: "Demo Theme".to_string(),
        padding: Padding::none(),
        x_position: Position::Relative(Relative::Align(Align::Start), None),
        y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
        background_color: conrod_core::color::DARK_CHARCOAL,
        shape_color: conrod_core::color::LIGHT_CHARCOAL,
        border_color: conrod_core::color::BLACK,
        border_width: 0.0,
        label_color: conrod_core::color::WHITE,
        font_id: None,
        font_size_large: 26,
        font_size_medium: 18,
        font_size_small: 12,
        widget_styling: conrod_core::theme::StyleMap::default(),
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
        // NumberDialer, PlotPath
        dialer_title,
        number_dialer,
        plot_path,
        // Scrollbar
        canvas_scrollbar,
    }
}

pub struct Gui {
    ids: Ids,
    button_xy_pad_toggle: button_xy_pad_toggle::Gui,
}

impl Gui {
    pub fn new(ui: &mut Ui) -> Self {
        Self {
            ids: Ids::new(ui.widget_id_generator()),
            button_xy_pad_toggle: button_xy_pad_toggle::Gui::new(ui),
        }
    }

    /// Instantiate a GUI demonstrating every widget available in conrod.
    pub fn update(&self, ui: &mut UiCell, app: &mut DemoApp) {
        let ids = &self.ids;

        // `Canvas` is a widget that provides some basic functionality for laying out children widgets.
        // By default, its size is the size of the window. We'll use this as a background for the
        // following widgets, as well as a scrollable container for the children widgets.
        widget::Canvas::new().pad(MARGIN).scroll_kids_vertically().set(ids.canvas, ui);

        self.update_text(ui);

        self.update_lines_and_shapes(ui);

        self.update_image(ui, app);

        let ball_x_range = ui.kid_area_of(ids.canvas).unwrap().w();
        let ball_y_range = ui.h_of(ui.window).unwrap() * 0.5;
        let rect = Rect::from_xy_dim([0.0, 0.0], [ball_x_range * 2.0 / 3.0, ball_y_range * 2.0 / 3.0]);
        let side = 130.0;
        
        self.button_xy_pad_toggle.update(ui, &mut app.button_xy_pad_toggle, ids.canvas, &rect, side);

        self.update_number_dialer_plotpath(ui, app, &rect, side);

        /////////////////////
        ///// Scrollbar /////
        /////////////////////

        widget::Scrollbar::y_axis(ids.canvas).auto_hide(true).set(ids.canvas_scrollbar, ui);
    }

    fn update_text(&self, ui: &mut conrod_core::UiCell){
        let ids = &self.ids;

        // We'll demonstrate the `Text` primitive widget by using it to draw a title and an
        // introduction to the example.
        const TITLE: &'static str = "All Widgets";
        widget::Text::new(TITLE).font_size(TITLE_SIZE).mid_top_of(ids.canvas).set(ids.title, ui);

        const INTRODUCTION: &'static str =
            "This example aims to demonstrate all widgets that are provided by conrod.\
            \n\nThe widget that you are currently looking at is the Text widget. The Text widget \
            is one of several special \"primitive\" widget types which are used to construct \
            all other widget types. These types are \"special\" in the sense that conrod knows \
            how to render them via `conrod_core::render::Primitive`s.\
            \n\nScroll down to see more widgets!";
        widget::Text::new(INTRODUCTION)
            .padded_w_of(ids.canvas, MARGIN)
            .down(60.0)
            .align_middle_x_of(ids.canvas)
            .center_justify()
            .line_spacing(5.0)
            .set(ids.introduction, ui);
    }

    fn update_lines_and_shapes(&self, ui: &mut conrod_core::UiCell){
        let ids = &self.ids;

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
            .color(conrod_core::color::TRANSPARENT)
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
            .color(conrod_core::color::CHARCOAL.alpha(0.25))
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
    }

    fn update_image(&self, ui: &mut conrod_core::UiCell, app: &mut DemoApp){
        let ids = &self.ids;

        widget::Text::new("Image")
            .down_from(ids.shapes_canvas, MARGIN)
            .align_middle_x_of(ids.canvas)
            .font_size(SUBTITLE_SIZE)
            .set(ids.image_title, ui);

        const LOGO_SIDE: conrod_core::Scalar = 144.0;
        widget::Image::new(app.rust_logo)
            .w_h(LOGO_SIDE, LOGO_SIDE)
            .down(60.0)
            .align_middle_x_of(ids.canvas)
            .set(ids.rust_logo, ui);
    }

    fn update_number_dialer_plotpath(&self, ui: &mut conrod_core::UiCell, app: &mut DemoApp,
        rect: &Rect, side: f64)
    {
        let ids = &self.ids;

        widget::Text::new("NumberDialer and PlotPath")
            .down(rect.y.end - rect.y.start + side * 0.5 + MARGIN)
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
    }

}

