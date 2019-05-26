
use conrod_core::{
    Colorable,
    Widget,
    Labelable,
    Positionable,
    Rect,
    Sizeable,
    Ui,
    UiCell,
    widget,
};

use layout::*;

widget_ids! {
    pub struct Ids {
        button_title,
        button,
        xy_pad,
        toggle,
        ball,
    }
}

pub struct GuiState {
    ball_xy: conrod_core::Point,
    ball_color: conrod_core::Color,
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            ball_xy: [0.0, 0.0],
            ball_color: conrod_core::color::WHITE,
        }
    }
}

pub struct Gui {
    ids: Ids,
}

impl Gui {
    pub fn new(ui: &mut Ui) -> Self {
        Self {
            ids: Ids::new(ui.widget_id_generator()),
        }
    }

    /// Returns id of widget that the next Gui should be down_from
    pub fn update(&self, ui: &mut UiCell, state: &mut GuiState, canvas: widget::Id, last: widget::Id, rect: &Rect, side: f64) -> widget::Id {
        let ids = &self.ids;

        widget::Text::new("Button, XYPad and Toggle")
            .down_from(last, 60.0)
            .align_middle_x_of(canvas)
            .font_size(SUBTITLE_SIZE)
            .set(ids.button_title, ui);

        for _press in widget::Button::new()
            .label("PRESS ME")
            .mid_left_with_margin_on(canvas, MARGIN)
            .down_from(ids.button_title, 60.0)
            .w_h(side, side)
            .set(ids.button, ui)
        {
            let x = rand::random::<conrod_core::Scalar>() * (rect.x.end - rect.x.start) - rect.x.end;
            let y = rand::random::<conrod_core::Scalar>() * (rect.y.end - rect.y.start) - rect.y.end;
            state.ball_xy = [x, y];
        }

        for (x, y) in widget::XYPad::new(state.ball_xy[0], rect.x.start, rect.x.end,
                                        state.ball_xy[1], rect.y.start, rect.y.end)
            .label("BALL XY")
            .wh_of(ids.button)
            .align_middle_y_of(ids.button)
            .align_middle_x_of(canvas)
            .parent(canvas)
            .set(ids.xy_pad, ui)
        {
            state.ball_xy = [x, y];
        }

        let is_white = state.ball_color == conrod_core::color::WHITE;
        let label = if is_white { "WHITE" } else { "BLACK" };
        for is_white in widget::Toggle::new(is_white)
            .label(label)
            .label_color(if is_white { conrod_core::color::WHITE } else { conrod_core::color::LIGHT_CHARCOAL })
            .mid_right_with_margin_on(canvas, MARGIN)
            .align_middle_y_of(ids.button)
            .set(ids.toggle, ui)
        {
            state.ball_color = if is_white { conrod_core::color::WHITE } else { conrod_core::color::BLACK };
        }

        let ball_x = state.ball_xy[0];
        let ball_y = state.ball_xy[1] - rect.y.end - side * 0.5 - MARGIN;
        widget::Circle::fill(20.0)
            .color(state.ball_color)
            .x_y_relative_to(ids.xy_pad, ball_x, ball_y)
            .set(ids.ball, ui);

        ids.xy_pad // Return id of widget that the next Gui should be down_from
    }
}
