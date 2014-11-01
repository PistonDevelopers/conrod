
use color::Color;
use dimensions::Dimensions;
use graphics::{
    AddRectangle,
    AddColor,
    BackEnd,
    Context,
    Draw,
    ImageSize,
};
use label;
use label::FontSize;
use point::Point;
use ui_context::UiContext;
use utils::map_range;

/// Represents the state of the Button widget.
#[deriving(PartialEq, Show)]
pub enum State {
    Normal,
    Highlighted,
    Clicked,
}

/// Draw a basic rectangle. The primary purpose
/// of this is to be used as a building block for
/// other widgets.
pub fn draw<B: BackEnd<I>, I: ImageSize>(
    win_w: f64,
    win_h: f64,
    graphics: &mut B,
    state: State,
    pos: Point,
    dim: Dimensions,
    maybe_frame: Option<(f64, Color)>,
    color: Color
) {
    let context = &Context::abs(win_w, win_h);
    if let Some((_, f_color)) = maybe_frame {
        draw_frame(context, graphics, pos, dim, f_color)
    }
    let f_width = if let Some((f_width, _)) = maybe_frame { f_width } else { 0.0 };
    draw_normal(context, graphics, state, pos, dim, f_width, color);
}

/// Draw the button border.
fn draw_frame<B: BackEnd<I>, I: ImageSize>(
    context: &Context,
    graphics: &mut B,
    pos: Point,
    dim: Dimensions,
    color: Color
) {
    let (r, g, b, a) = color.as_tuple();
    context
        .rect(pos[0], pos[1], dim[0], dim[1])
        .rgba(r, g, b, a)
        .draw(graphics);
}

/// Draw the rectangle while considering frame
/// width for position and dimensions.
fn draw_normal<B: BackEnd<I>, I: ImageSize>(
    context: &Context,
    graphics: &mut B,
    state: State,
    pos: Point,
    dim: Dimensions,
    frame_width: f64,
    color: Color
) {
    let (r, g, b, a) = match state {
        Normal => color.as_tuple(),
        Highlighted => color.highlighted().as_tuple(),
        Clicked => color.clicked().as_tuple(),
    };
    context
        .rect(pos[0] + frame_width,
              pos[1] + frame_width,
              dim[0] - frame_width * 2.0,
              dim[1] - frame_width * 2.0)
        .rgba(r, g, b, a)
        .draw(graphics);
}

/// Return whether or not the widget has been hit by a mouse_press.
#[inline]
pub fn is_over(pos: Point,
               mouse_pos: Point,
               dim: Dimensions) -> bool {
    if mouse_pos[0] > pos[0]
    && mouse_pos[1] > pos[1]
    && mouse_pos[0] < pos[0] + dim[0]
    && mouse_pos[1] < pos[1] + dim[1] { true }
    else { false }
}

/// Draw a label centered within a rect of given position and dimensions.
pub fn draw_with_centered_label<B: BackEnd<T>, T: ImageSize>(
    win_w: f64,
    win_h: f64,
    graphics: &mut B,
    uic: &mut UiContext<T>,
    state: State,
    pos: Point,
    dim: Dimensions,
    maybe_frame: Option<(f64, Color)>,
    color: Color,
    text: &str,
    font_size: FontSize,
    text_color: Color
) {
    let context = &Context::abs(win_w, win_h);
    if let Some((_, f_color)) = maybe_frame {
        draw_frame(context, graphics, pos, dim, f_color)
    }
    let f_width = if let Some((f_width, _)) = maybe_frame { f_width } else { 0.0 };
    draw_normal(context, graphics, state, pos, dim, f_width, color);
    let text_w = label::width(uic, font_size, text);
    let l_pos = [pos[0] + (dim[0] - text_w) / 2.0, pos[1] + (dim[1] - font_size as f64) / 2.0];
    label::draw(graphics, uic, l_pos, font_size, text_color, text);
}

pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Return which corner of the rectangle the given Point is within.
pub fn corner(rect_p: Point, p: Point, dim: Dimensions) -> Corner {
    let x_temp = p[0] - rect_p[0];
    let y_temp = p[1] - rect_p[1];
    let x_perc = map_range(x_temp, 0.0, dim[0], 0f64, 1.0);
    let y_perc = map_range(y_temp, dim[1], 0.0, 0f64, 1.0);
    if      x_perc <= 0.5 && y_perc <= 0.5 { BottomLeft }
    else if x_perc >  0.5 && y_perc <= 0.5 { BottomRight }
    else if x_perc <= 0.5 && y_perc >  0.5 { TopLeft }
    else                                   { TopRight }
}

