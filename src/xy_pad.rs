use std::num::Float;
use color::Color;
use dimensions::Dimensions;
use graphics;
use graphics::{
    Context,
};
use label;
use label::FontSize;
use mouse_state::MouseState;
use opengl_graphics::Gl;
use point::Point;
use rectangle;
use rectangle::{
    Corner
};
use ui_context::{
    UIID,
    UiContext,
};
use utils::{
    clamp,
    map_range,
    val_to_string,
};
use vecmath::{
    vec2_add,
    vec2_sub,
};
use widget::Widget::XYPad;

/// Represents the state of the xy_pad widget.
#[deriving(Show, PartialEq, Clone)]
pub enum State {
    Normal,
    Highlighted,
    Clicked,
}

impl State {
    /// Return the associated Rectangle state.
    fn as_rectangle_state(&self) -> rectangle::State {
        match self {
            &State::Normal => rectangle::State::Normal,
            &State::Highlighted => rectangle::State::Highlighted,
            &State::Clicked => rectangle::State::Clicked,
        }
    }
}

widget_fns!(XYPad, State, XYPad(State::Normal))

/// Check the current state of the button.
fn get_new_state(is_over: bool,
                 prev: State,
                 mouse: MouseState) -> State {
    use mouse_state::MouseButtonState::{Down, Up};
    use self::State::{Normal, Highlighted, Clicked};
    match (is_over, prev, mouse.left) {
        (true,  Normal,  Down) => Normal,
        (true,  _,       Down) => Clicked,
        (true,  _,       Up)   => Highlighted,
        (false, Clicked, Down) => Clicked,
        _                      => Normal,
    }
}

/// Draw the crosshair.
fn draw_crosshair(
    win_w: f64,
    win_h: f64,
    graphics: &mut Gl,
    pos: Point,
    line_width: f64,
    vert_x: f64, hori_y: f64,
    pad_dim: Dimensions,
    color: Color
) {
    let context = &Context::abs(win_w, win_h);
    let Color(col) = color;
    let line = graphics::Line::new(col, 0.5 * line_width);
    line.draw([vert_x, pos[1], vert_x, pos[1] + pad_dim[1]], context, graphics);
    line.draw([pos[0], hori_y, pos[0] + pad_dim[0], hori_y], context, graphics);
}


/// A context on which the builder pattern can be implemented.
pub struct XYPadContext<'a, X, Y> {
    uic: &'a mut UiContext,
    ui_id: UIID,
    x: X, min_x: X, max_x: X,
    y: Y, min_y: Y, max_y: Y,
    line_width: f64,
    font_size: FontSize,
    pos: Point,
    dim: Dimensions,
    maybe_callback: Option<|X, Y|:'a>,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_label: Option<&'a str>,
    maybe_label_color: Option<Color>,
    maybe_label_font_size: Option<u32>,
}

impl <'a, X, Y> XYPadContext<'a, X, Y> {
    #[inline]
    pub fn line_width(self, width: f64) -> XYPadContext<'a, X, Y> {
        XYPadContext { line_width: width, ..self }
    }
    #[inline]
    pub fn value_font_size(self, size: FontSize) -> XYPadContext<'a, X, Y> {
        XYPadContext { font_size: size, ..self }
    }
}

pub trait XYPadBuilder<'a, X: Float + Copy + ToPrimitive + FromPrimitive + ToString,
                           Y: Float + Copy + ToPrimitive + FromPrimitive + ToString> {
    /// A xy_pad builder method to be implemented by the UiContext.
    fn xy_pad(&'a mut self, ui_id: UIID,
              x_val: X, x_min: X, x_max: X,
              y_val: Y, y_min: Y, y_max: Y) -> XYPadContext<'a, X, Y>;
}

impl<'a, X: Float + Copy + ToPrimitive + FromPrimitive + ToString,
         Y: Float + Copy + ToPrimitive + FromPrimitive + ToString>
XYPadBuilder<'a, X, Y> for UiContext {
    /// An xy_pad builder method to be implemented by the UiContext.
    fn xy_pad(&'a mut self, ui_id: UIID,
              x_val: X, min_x: X, max_x: X,
              y_val: Y, min_y: Y, max_y: Y) -> XYPadContext<'a, X, Y> {
        XYPadContext {
            uic: self,
            ui_id: ui_id,
            x: x_val, min_x: min_x, max_x: max_x,
            y: y_val, min_y: min_y, max_y: max_y,
            line_width: 1.0,
            font_size: 18u32,
            pos: [0.0, 0.0],
            dim: [128.0, 128.0],
            maybe_callback: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
        }
    }
}

impl_callable!(XYPadContext, |X, Y|:'a, X, Y)
impl_colorable!(XYPadContext, X, Y)
impl_frameable!(XYPadContext, X, Y)
impl_labelable!(XYPadContext, X, Y)
impl_positionable!(XYPadContext, X, Y)
impl_shapeable!(XYPadContext, X, Y)

impl<'a, X: Float + Copy + ToPrimitive + FromPrimitive + ToString,
         Y: Float + Copy + ToPrimitive + FromPrimitive + ToString>
::draw::Drawable for XYPadContext<'a, X, Y> {
    fn draw(&mut self, graphics: &mut Gl) {

        // Init.
        let state = *get_state(self.uic, self.ui_id);
        let mouse = self.uic.get_mouse_state();
        let frame_w = self.maybe_frame.unwrap_or(self.uic.theme.frame_width);
        let frame_w2 = frame_w * 2.0;
        let maybe_frame = match frame_w > 0.0 {
            true => Some((frame_w, self.maybe_frame_color.unwrap_or(self.uic.theme.frame_color))),
            false => None,
        };
        let pad_dim = vec2_sub(self.dim, [frame_w2, ..2]);
        let pad_pos = vec2_add(self.pos, [frame_w, frame_w]);
        let is_over_pad = rectangle::is_over(pad_pos, mouse.pos, pad_dim);
        let new_state = get_new_state(is_over_pad, state, mouse);

        // Determine new values.
        let (new_x, new_y) = match (is_over_pad, new_state) {
            (_, State::Normal) | (_, State::Highlighted) => (self.x, self.y),
            (_, State::Clicked) => {
                let temp_x = clamp(mouse.pos[0], pad_pos[0], pad_pos[0] + pad_dim[0]);
                let temp_y = clamp(mouse.pos[1], pad_pos[1], pad_pos[1] + pad_dim[1]);
                (map_range(temp_x - self.pos[0], pad_dim[0], 0.0, self.min_x, self.max_x),
                 map_range(temp_y - self.pos[1], pad_dim[1], 0.0, self.min_y, self.max_y))
            }
        };

        // Callback if value is changed or the pad is clicked/released.
        match self.maybe_callback {
            Some(ref mut callback) => {
                if self.x != new_x || self.y != new_y { (*callback)(new_x, new_y) }
                else {
                    match (state, new_state) {
                        (State::Highlighted, State::Clicked)
                        | (State::Clicked, State::Highlighted) => (*callback)(new_x, new_y),
                        _ => (),
                    }
                }
            },
            None => (),
        }

        // Draw.
        let rect_state = new_state.as_rectangle_state();
        let color = self.maybe_color.unwrap_or(self.uic.theme.shape_color);
        rectangle::draw(self.uic.win_w, self.uic.win_h, graphics, rect_state, self.pos,
                        self.dim, maybe_frame, color);
        let (vert_x, hori_y) = match (is_over_pad, new_state) {
            (_, State::Normal) | (_, State::Highlighted) =>
                (pad_pos[0] + map_range(new_x, self.min_x, self.max_x, pad_dim[0], 0.0),
                 pad_pos[1] + map_range(new_y, self.min_y, self.max_y, pad_dim[1], 0.0)),
            (_, State::Clicked) =>
                (clamp(mouse.pos[0], pad_pos[0], pad_pos[0] + pad_dim[0]),
                 clamp(mouse.pos[1], pad_pos[1], pad_pos[1] + pad_dim[1])),
        };
        // Crosshair.
        draw_crosshair(self.uic.win_w, self.uic.win_h, graphics, pad_pos, self.line_width,
                       vert_x, hori_y, pad_dim, color.plain_contrast());
        // Label.
        if let Some(l_text) = self.maybe_label {
            let l_color = self.maybe_label_color.unwrap_or(self.uic.theme.label_color);
            let l_size = self.maybe_label_font_size.unwrap_or(self.uic.theme.font_size_medium);
            let l_w = label::width(self.uic, l_size, l_text);
            let l_x = pad_pos[0] + (pad_dim[0] - l_w) / 2.0;
            let l_y = pad_pos[1] + (pad_dim[1] - l_size as f64) / 2.0;
            let l_pos = [l_x, l_y];
            label::draw(graphics, self.uic, l_pos, l_size, l_color, l_text);
        }
        // xy value string.
        let x_string = val_to_string(self.x, self.max_x,
                                     self.max_x - self.min_x, self.dim[0] as uint);
        let y_string = val_to_string(self.y, self.max_y,
                                     self.max_y - self.min_y, self.dim[1] as uint);
        let xy_string = format!("{}, {}", x_string, y_string);
        let xy_string_w = label::width(self.uic, self.font_size, xy_string.as_slice());
        let xy_string_pos = {
            match rectangle::corner(pad_pos, [vert_x, hori_y], pad_dim) {
                Corner::TopLeft => [vert_x, hori_y],
                Corner::TopRight => [vert_x - xy_string_w, hori_y],
                Corner::BottomLeft => [vert_x, hori_y - self.font_size as f64],
                Corner::BottomRight => [vert_x - xy_string_w, hori_y - self.font_size as f64],
            }
        };
        label::draw(graphics, self.uic, xy_string_pos, self.font_size,
                    color.plain_contrast(), xy_string.as_slice());

        set_state(self.uic, self.ui_id, new_state, self.pos, self.dim);

    }
}


