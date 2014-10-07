
use color::Color;
use graphics::{
    AddColor,
    AddLine,
    AddSquareBorder,
    Context,
    Draw,
};
use label;
use label::FontSize;
use mouse_state::{
    MouseState,
    Up,
    Down,
};
use opengl_graphics::Gl;
use point::Point;
use rectangle;
use rectangle::{
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
};
use ui_context::{
    UIID,
    UIContext,
};
use utils::{
    clamp,
    map_range,
    val_to_string,
};
use widget::XYPad;

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
            &Normal => rectangle::Normal,
            &Highlighted => rectangle::Highlighted,
            &Clicked => rectangle::Clicked,
        }
    }
}

widget_fns!(XYPad, State, XYPad(Normal))

/// Check the current state of the button.
fn get_new_state(is_over: bool,
                 prev: State,
                 mouse: MouseState) -> State {
    match (is_over, prev, mouse.left) {
        (true, Normal, Down) => Normal,
        (true, _, Down) => Clicked,
        (true, _, Up) => Highlighted,
        (false, Clicked, Down) => Clicked,
        _ => Normal,
    }
}

/// Draw the crosshair.
fn draw_crosshair(win_w: f64,
                  win_h: f64,
                  gl: &mut Gl,
                  pos: Point<f64>,
                  line_width: f64,
                  vert_x: f64, hori_y: f64,
                  pad_w: f64, pad_h: f64,
                  color: Color) {
    let context = &Context::abs(win_w, win_h);
    let (r, g, b, a) = color.as_tuple();
    context
        .line(vert_x, pos.y, vert_x, pos.y + pad_h)
        .square_border_width(line_width)
        .rgba(r, g, b, a)
        .draw(gl);
    context
        .line(pos.x, hori_y, pos.x + pad_w, hori_y)
        .square_border_width(line_width)
        .rgba(r, g, b, a)
        .draw(gl);
}


/// A context on which the builder pattern can be implemented.
pub struct XYPadContext<'a, X, Y> {
    uic: &'a mut UIContext,
    ui_id: UIID,
    x: X, min_x: X, max_x: X,
    y: Y, min_y: Y, max_y: Y,
    line_width: f64,
    font_size: FontSize,
    pos: Point<f64>,
    width: f64,
    height: f64,
    maybe_callback: Option<|X, Y|:'a>,
    maybe_color: Option<Color>,
    maybe_frame: Option<(f64, Color)>,
    maybe_label: Option<(&'a str, FontSize, Color)>,
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

pub trait XYPadBuilder<'a, X: Num + Copy + ToPrimitive + FromPrimitive + ToString,
                           Y: Num + Copy + ToPrimitive + FromPrimitive + ToString> {
    /// A xy_pad builder method to be implemented by the UIContext.
    fn xy_pad(&'a mut self, ui_id: UIID,
              x_val: X, x_min: X, x_max: X,
              y_val: Y, y_min: Y, y_max: Y) -> XYPadContext<'a, X, Y>;
}

impl<'a, X: Num + Copy + ToPrimitive + FromPrimitive + ToString,
         Y: Num + Copy + ToPrimitive + FromPrimitive + ToString>
XYPadBuilder<'a, X, Y> for UIContext {
    /// An xy_pad builder method to be implemented by the UIContext.
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
            pos: Point::new(0.0, 0.0, 0.0),
            width: 128.0,
            height: 128.0,
            maybe_callback: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_label: None,
        }
    }
}

impl_callable!(XYPadContext, |X, Y|:'a, X, Y)
impl_colorable!(XYPadContext, X, Y)
impl_frameable!(XYPadContext, X, Y)
impl_labelable!(XYPadContext, X, Y)
impl_positionable!(XYPadContext, X, Y)
impl_shapeable!(XYPadContext, X, Y)

impl<'a, X: Num + Copy + ToPrimitive + FromPrimitive + ToString,
         Y: Num + Copy + ToPrimitive + FromPrimitive + ToString>
::draw::Drawable for XYPadContext<'a, X, Y> {
    fn draw(&mut self, gl: &mut Gl) {

        // Init.
        let state = *get_state(self.uic, self.ui_id);
        let mouse = self.uic.get_mouse_state();
        let frame_w = match self.maybe_frame { Some((w, _)) => w, None => 0.0 };
        let frame_w2 = frame_w * 2.0;
        let pad_w = self.width - frame_w2;
        let pad_h = self.height - frame_w2;
        let pad_pos = self.pos + Point::new(frame_w, frame_w, 0.0);
        let is_over_pad = rectangle::is_over(pad_pos, mouse.pos, pad_w, pad_h);
        let new_state = get_new_state(is_over_pad, state, mouse);

        // Determine new values.
        let (new_x, new_y) = match (is_over_pad, new_state) {
            (_, Normal) | (_, Highlighted) => (self.x, self.y),
            (_, Clicked) => {
                let temp_x = clamp(mouse.pos.x, pad_pos.x, pad_pos.x + pad_w);
                let temp_y = clamp(mouse.pos.y, pad_pos.y, pad_pos.y + pad_h);
                (map_range(temp_x - self.pos.x, pad_w, 0.0, self.min_x, self.max_x),
                 map_range(temp_y - self.pos.y, pad_h, 0.0, self.min_y, self.max_y))
            }
        };

        // Callback if value is changed or the pad is clicked/released.
        match self.maybe_callback {
            Some(ref mut callback) => {
                if self.x != new_x || self.y != new_y { (*callback)(new_x, new_y) }
                else {
                    match (state, new_state) {
                        (Highlighted, Clicked)
                        | (Clicked, Highlighted) => (*callback)(new_x, new_y),
                        _ => (),
                    }
                }
            },
            None => (),
        }

        // Draw.
        let rect_state = new_state.as_rectangle_state();
        let color = self.maybe_color.unwrap_or(::std::default::Default::default());
        rectangle::draw(self.uic.win_w, self.uic.win_h, gl, rect_state, self.pos,
                        self.width, self.height, self.maybe_frame, color);
        let (vert_x, hori_y) = match (is_over_pad, new_state) {
            (_, Normal) | (_, Highlighted) =>
                (pad_pos.x + map_range(new_x, self.min_x, self.max_x, pad_w, 0.0),
                 pad_pos.y + map_range(new_y, self.min_y, self.max_y, pad_h, 0.0)),
            (_, Clicked) =>
                (clamp(mouse.pos.x, pad_pos.x, pad_pos.x + pad_w),
                 clamp(mouse.pos.y, pad_pos.y, pad_pos.y + pad_h)),
        };
        // Crosshair.
        draw_crosshair(self.uic.win_w, self.uic.win_h, gl, pad_pos, self.line_width,
                       vert_x, hori_y, pad_w, pad_h, color.plain_contrast());
        // Label.
        match self.maybe_label {
            None => (),
            Some((l_text, l_size, l_color)) => {
                let l_w = label::width(self.uic, l_size, l_text);
                let l_x = pad_pos.x + (pad_w - l_w) / 2.0;
                let l_y = pad_pos.y + (pad_h - l_size as f64) / 2.0;
                let l_pos = Point::new(l_x, l_y, 0.0);
                label::draw(gl, self.uic, l_pos, l_size, l_color, l_text);
            },
        }
        // xy value string.
        let x_string = val_to_string(self.x, self.max_x,
                                     self.max_x - self.min_x, self.width as uint);
        let y_string = val_to_string(self.y, self.max_y,
                                     self.max_y - self.min_y, self.height as uint);
        let xy_string = format!("{}, {}", x_string, y_string);
        let xy_string_w = label::width(self.uic, self.font_size, xy_string.as_slice());
        let xy_string_pos = {
            match rectangle::corner(pad_pos, Point::new(vert_x, hori_y, 0.0), pad_w, pad_h) {
                TopLeft => Point::new(vert_x, hori_y, 0.0),
                TopRight => Point::new(vert_x - xy_string_w, hori_y, 0.0),
                BottomLeft => Point::new(vert_x, hori_y - self.font_size as f64, 0.0),
                BottomRight => Point::new(vert_x - xy_string_w, hori_y - self.font_size as f64, 0.0),
            }
        };
        label::draw(gl, self.uic, xy_string_pos, self.font_size,
                    color.plain_contrast(), xy_string.as_slice());

        set_state(self.uic, self.ui_id, new_state,
                  self.pos.x, self.pos.y, self.width, self.height);

    }
}


