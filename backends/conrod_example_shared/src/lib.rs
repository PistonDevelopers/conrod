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
mod shapes;
mod image;
mod button_xy_pad_toggle;
mod number_dialer_plotpath;

use layout::*;

use conrod_core::{widget, Positionable, Rect, Sizeable, Ui, UiCell, Widget};

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
        // Scrollbar
        canvas_scrollbar,
    }
}

pub struct Gui {
    ids: Ids,
    shapes: shapes::Gui,
    image: image::Gui,
    button_xy_pad_toggle: button_xy_pad_toggle::Gui,
    number_dialer_plotpath: number_dialer_plotpath::Gui,
}

impl Gui {
    pub fn new(ui: &mut Ui) -> Self {
        Self {
            ids: Ids::new(ui.widget_id_generator()),
            shapes: shapes::Gui::new(ui),
            image: image::Gui::new(ui),
            button_xy_pad_toggle: button_xy_pad_toggle::Gui::new(ui),
            number_dialer_plotpath: number_dialer_plotpath::Gui::new(ui),
        }
    }

    /// Instantiate a GUI demonstrating every widget available in conrod.
    pub fn update(&self, ui: &mut UiCell, app: &mut DemoApp) {
        let ids = &self.ids;
        let canvas = ids.canvas;

        // `Canvas` is a widget that provides some basic functionality for laying out children widgets.
        // By default, its size is the size of the window. We'll use this as a background for the
        // following widgets, as well as a scrollable container for the children widgets.
        widget::Canvas::new().pad(MARGIN).scroll_kids_vertically().set(canvas, ui);

        self.update_text(ui);

        let last = self.shapes.update(ui, canvas);

        let last = self.image.update(ui, app.rust_logo, canvas, last);

        let ball_x_range = ui.kid_area_of(canvas).unwrap().w();
        let ball_y_range = ui.h_of(ui.window).unwrap() * 0.5;
        let rect = Rect::from_xy_dim([0.0, 0.0], [ball_x_range * 2.0 / 3.0, ball_y_range * 2.0 / 3.0]);
        let side = 130.0;
        
        let last = self.button_xy_pad_toggle.update(ui, &mut app.button_xy_pad_toggle, canvas, last, &rect, side);
        
        let space = rect.y.end - rect.y.start + side * 0.5 + MARGIN;
        self.number_dialer_plotpath.update(ui, &mut app.sine_frequency, canvas, last, space);

        /////////////////////
        ///// Scrollbar /////
        /////////////////////

        widget::Scrollbar::y_axis(canvas).auto_hide(true).set(ids.canvas_scrollbar, ui);
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

}
