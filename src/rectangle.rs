
use color::Color;
use dimensions::Dimensions;
use graphics::{self, DrawState, Graphics};
use graphics::math::Matrix2d;
use graphics::character::CharacterCache;
use label::{self, FontSize};
use point::Point;
use ui::Ui;
use utils::map_range;

/// Represents the state of the Button widget.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum State {
    Normal,
    Highlighted,
    Clicked,
}

/// Draw a basic rectangle. The primary purpose
/// of this is to be used as a building block for
/// other widgets.
pub fn draw<B: Graphics>(
    win_w: f64,
    win_h: f64,
    graphics: &mut B,
    state: State,
    pos: Point,
    dim: Dimensions,
    maybe_frame: Option<(f64, Color)>,
    color: Color
) {
    let draw_state = graphics::default_draw_state();
    let transform = graphics::abs_transform(win_w, win_h);
    if let Some((_, f_color)) = maybe_frame {
        draw_frame(draw_state, transform, graphics, pos, dim, f_color)
    }
    let f_width = if let Some((f_width, _)) = maybe_frame { f_width } else { 0.0 };
    draw_normal(draw_state, transform, graphics, state, pos, dim, f_width, color);
}

/// Draw the button border.
fn draw_frame<B: Graphics>(
    draw_state: &DrawState,
    transform: Matrix2d,
    graphics: &mut B,
    pos: Point,
    dim: Dimensions,
    color: Color
) {
    graphics::Rectangle::new(color.to_fsa())
        .draw(
            [pos[0], pos[1], dim[0], dim[1]],
            draw_state,
            transform,
            graphics
        );
}

/// Draw the rectangle while considering frame
/// width for position and dimensions.
fn draw_normal<B: Graphics>(
    draw_state: &DrawState,
    transform: Matrix2d,
    graphics: &mut B,
    state: State,
    pos: Point,
    dim: Dimensions,
    frame_width: f64,
    color: Color
) {
    let color = match state {
        State::Normal => color,
        State::Highlighted => color.highlighted(),
        State::Clicked => color.clicked(),
    };
    graphics::Rectangle::new(color.to_fsa())
        .draw([pos[0] + frame_width,
            pos[1] + frame_width,
            dim[0] - frame_width * 2.0,
            dim[1] - frame_width * 2.0],
        draw_state,
        transform,
        graphics);
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
pub fn draw_with_centered_label<B, C>(
    win_w: f64,
    win_h: f64,
    graphics: &mut B,
    ui: &mut Ui<C>,
    state: State,
    pos: Point,
    dim: Dimensions,
    maybe_frame: Option<(f64, Color)>,
    color: Color,
    text: &str,
    font_size: FontSize,
    text_color: Color
)
    where
        B: Graphics<Texture = <C as CharacterCache>::Texture>,
        C: CharacterCache
{
    let draw_state = graphics::default_draw_state();
    let transform = graphics::abs_transform(win_w, win_h);
    if let Some((_, f_color)) = maybe_frame {
        draw_frame(draw_state, transform, graphics, pos, dim, f_color)
    }
    let f_width = if let Some((f_width, _)) = maybe_frame { f_width } else { 0.0 };
    draw_normal(draw_state, transform, graphics, state, pos, dim, f_width, color);
    let text_w = label::width(ui, font_size, text);
    let l_pos = [pos[0] + (dim[0] - text_w) / 2.0, pos[1] + (dim[1] - font_size as f64) / 2.0];
    ui.draw_text(graphics, l_pos, font_size, text_color, text);
}

#[derive(Copy, Clone)]
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
    if      x_perc <= 0.5 && y_perc <= 0.5 { Corner::BottomLeft }
    else if x_perc >  0.5 && y_perc <= 0.5 { Corner::BottomRight }
    else if x_perc <= 0.5 && y_perc >  0.5 { Corner::TopLeft }
    else                                   { Corner::TopRight }
}
