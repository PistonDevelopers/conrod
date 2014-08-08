
use widget;
use widget::{
    RelativePosition,
    Widget,
    Clicked,
    Highlighted,
    Normal,
};
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
use std::default::Default;

/// Rectangle used for constructing 2D rectangular widgets.
/// Rectangle can be treated as a child widget when used
/// to construct other widgets. I.e. A `Slider` is made up
/// of two Rectangles; one for the frame, and one for the
/// slider itself.
#[deriving(Show, Clone)]
pub struct Rectangle {
    widget_data: widget::Data,
    pub width: uint,
    pub height: uint,
    pub color: Color,
    border: uint,
}

impl Rectangle {

    /// Constructor for a widget rectangle.
    pub fn new(pos: RelativePosition,
               width: uint,
               height: uint,
               color: Color,
               border: uint) -> Rectangle {
        Rectangle {
            widget_data: widget::Data::new(pos),
            width: width,
            height: height,
            color: color,
            border: border,
        }
    }

    /// Draw the button border.
    fn draw_border(&self, context: &Context, _args: &RenderArgs, gl: &mut Gl) {
        let (r, g, b, a) = (0.0, 0.0, 0.0, 1.0);
        let pos = self.get_abs_pos();
        context
            .rect(pos.x as f64, pos.y as f64, self.width as f64, self.height as f64)
            .rgba(r, g, b, a)
            .draw(gl);
    }

    /// Draw the button while considering border for dimensions and position.
    fn draw_normal(&self, context: &Context, _args: &RenderArgs, gl: &mut Gl) {
        let bw = self.border as f64;
        let hbw = bw / 2f64;
        let (r, g, b, a) = match self.get_draw_state() {
            Normal => self.color.as_tuple(),
            Highlighted => self.color.highlighted().as_tuple(),
            Clicked => self.color.clicked().as_tuple(),
        };
        let pos = self.get_abs_pos();
        context
            .rect(pos.x as f64 + hbw, pos.y as f64 + hbw, self.width as f64 - bw, self.height as f64 - bw)
            .rgba(r, g, b, a)
            .draw(gl);
    }

}

impl Default for Rectangle {
    
    /// Default constructor for a widget Rectangle.
    fn default() -> Rectangle {
        Rectangle {
            widget_data: Default::default(),
            width: 100u,
            height: 50u,
            color: Default::default(),
            border: 7u,
        }
    }

}

impl Widget for Rectangle {

    impl_get_widget_data!(widget_data)

    /// Return the dimensions as a tuple holding width and height.
    fn get_dimensions(&self) -> (uint, uint) { (self.width, self.height) }

    /// Return whether or not the widget has been hit by a mouse_press.
    fn is_over(&self, mouse_pos: Point<int>) -> bool {
        let p = mouse_pos - self.get_abs_pos();
        if p.x > 0 && p.y > 0 && p.x < self.width as int && p.y < self.height as int { true }
        else { false }
    }

    /// Draw the rectangle.
    fn draw(&mut self, args: &RenderArgs, gl: &mut Gl) {
        let context = &Context::abs(args.width as f64, args.height as f64);
        self.draw_border(context, args, gl);
        self.draw_normal(context, args, gl);
    }

}
