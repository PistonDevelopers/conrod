
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
use frame::{
    Framing,
    Frame,
    NoFrame,
};
use label;
use label::FontSize;
use ui_context::UIContext;

/// Represents the state of the Button widget.
#[deriving(Show, PartialEq)]
pub enum State {
    Normal,
    Highlighted,
    Clicked,
}

/// Draw a basic rectangle. The primary purpose
/// of this is to be used as a building block for
/// other widgets.
pub fn draw(args: &RenderArgs,
            gl: &mut Gl,
            state: State,
            pos: Point<f64>,
            width: f64,
            height: f64,
            frame: Framing,
            color: Color) {
    let context = &Context::abs(args.width as f64, args.height as f64);
    match frame {
        Frame(_, f_color) => draw_frame(context, gl, pos, width, height, f_color),
        NoFrame => (),
    }
    draw_normal(context, gl, state, pos, width, height, frame, color);
}

/// Draw the button border.
fn draw_frame(context: &Context,
              gl: &mut Gl,
              pos: Point<f64>,
              width: f64,
              height: f64,
              color: Color) {
    let (r, g, b, a) = color.as_tuple();
    context
        .rect(pos.x, pos.y, width, height)
        .rgba(r, g, b, a)
        .draw(gl);
}

/// Draw the rectangle while considering frame
/// width for position and dimensions.
fn draw_normal(context: &Context,
               gl: &mut Gl,
               state: State,
               pos: Point<f64>,
               width: f64,
               height: f64,
               frame: Framing,
               color: Color) {
    let (r, g, b, a) = match state {
        Normal => color.as_tuple(),
        Highlighted => color.highlighted().as_tuple(),
        Clicked => color.clicked().as_tuple(),
    };
    let frame_w = match frame {
        Frame(frame_w, _) => frame_w,
        _ => 0.0,
    };
    context
        .rect(pos.x + frame_w,
              pos.y + frame_w,
              width - frame_w * 2.0,
              height - frame_w * 2.0)
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

/// Draw a label centered within a rect of given position and dimensions.
pub fn draw_with_centered_label(args: &RenderArgs,
                                gl: &mut Gl,
                                uic: &mut UIContext,
                                state: State,
                                pos: Point<f64>,
                                width: f64,
                                height: f64,
                                frame: Framing,
                                color: Color,
                                text: &str,
                                font_size: FontSize,
                                text_color: Color) {
    let context = &Context::abs(args.width as f64, args.height as f64);
    match frame {
        Frame(_, f_color) => draw_frame(context, gl, pos, width, height, f_color),
        NoFrame => (),
    }
    draw_normal(context, gl, state, pos, width, height, frame, color);
    let text_w = label::width(uic, font_size, text);
    let x = pos.x + (width - text_w) / 2.0;
    let y = pos.y + (height - font_size as f64) / 2.0;
    let l_pos = Point::new(x, y, 0.0);
    label::draw(args, gl, uic, l_pos, font_size, text_color, text);
}


