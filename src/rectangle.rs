
use piston::{
    RenderArgs,
};
use opengl_graphics::Gl;
use graphics::{
    Context,
    AddRectangle,
    AddColor,
    Draw,
};
use point::Point;
use color::Color;

widget_state!(RectangleState, RectangleState {
    Normal -> 0,
    Highlighted -> 1,
    Clicked -> 2
})

/// Draw a basic rectangle. The primary purpose
/// of this is to be used as a building block for
/// other widgets.
pub fn draw(args: &RenderArgs,
            gl: &mut Gl,
            state: RectangleState,
            pos: Point<f64>,
            width: f64,
            height: f64,
            border: f64,
            color: Color) {
    let context = &Context::abs(args.width as f64, args.height as f64);
    draw_border(context, gl, pos, width, height);
    draw_normal(context, gl, state, pos, width, height, border, color);
}

/// Draw the button border.
fn draw_border(context: &Context,
               gl: &mut Gl,
               pos: Point<f64>,
               width: f64,
               height: f64) {
    let (r, g, b, a) = (0.0, 0.0, 0.0, 1.0);
    context
        .rect(pos.x, pos.y, width, height)
        .rgba(r, g, b, a)
        .draw(gl);
}

/// Draw the button while considering border for dimensions and position.
fn draw_normal(context: &Context,
               gl: &mut Gl,
               state: RectangleState,
               pos: Point<f64>,
               width: f64,
               height: f64,
               border: f64,
               color: Color) {
    let half_border = border / 2f64;
    let (r, g, b, a) = match state {
        Normal => color.as_tuple(),
        Highlighted => color.highlighted().as_tuple(),
        Clicked => color.clicked().as_tuple(),
    };
    context
        .rect(pos.x + half_border, pos.y + half_border, width - border, height - border)
        .rgba(r, g, b, a)
        .draw(gl);
}

/// Return whether or not the widget has been hit by a mouse_press.
pub fn is_over(pos: Point<f64>,
               mouse_pos: Point<f64>,
               width: f64,
               height: f64) -> bool {
    let p = mouse_pos - pos;
    if p.x > 0f64 && p.y > 0f64 && p.x < width && p.y < height { true }
    else { false }
}

