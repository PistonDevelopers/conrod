
use piston::{
    MouseReleaseArgs,
};
use piston::mouse;
use widget;
use widget::{
    Widget,
    RelativePosition,
    Highlighted,
    Clicked,
};
use std::default::Default;
use graphics::AddRectangle;
use color::Color;
use point::Point;
use rectangle::Rectangle;

/// A simple `Toggle` type widget which will always
/// remain in one of two states: `true` (on) or
/// `false` (off).
#[deriving(Show, Clone)]
pub struct Toggle {
    widget_data: widget::Data,
    rect: Rectangle,
    color: Color,
    value: bool,
}

impl Toggle {

    /// Constructor for Button widget.
    pub fn new(pos: RelativePosition,
               width: uint,
               height: uint,
               border: uint,
               color: Color,
               value: bool) -> Toggle {
        let r_color = match value {
            true => color,
            false => Color::new(color.r * 0.1f32, color.g * 0.1f32, color.b * 0.1f32, color.a),
        };
        Toggle {
            widget_data: widget::Data::new(pos),
            rect: Rectangle::new(pos, width, height, r_color, border),
            color: color,
            value: value,
        }
    }

}

impl Default for Toggle {

    /// Default constructor for Button widget.
    fn default() -> Toggle {
        Toggle {
            widget_data: Default::default(),
            rect: Default::default(),
            color: Default::default(),
            value: false,
        }
    }

}

impl Widget for Toggle {

    impl_get_widget_data!(widget_data)

    /// Return the dimensions as a tuple holding width and height.
    fn get_dimensions(&self) -> (uint, uint) { self.rect.get_dimensions() }

    /// Return a reference to the rectangle as a widget child.
    fn get_children(&self) -> Vec<&Widget> {
        vec![&self.rect as &Widget]
    }

    /// Return all children widgets.
    fn get_children_mut(&mut self) -> Vec<&mut Widget> {
        vec![&mut self.rect as &mut Widget]
    }

    /// Return whether or not the widget has been hit by a mouse_press.
    fn is_over(&self, mouse_pos: Point<int>) -> bool {
        self.rect.is_over(mouse_pos)
    }

    /// Mouse released.
    fn mouse_release_update_draw_state(&mut self, args: &MouseReleaseArgs) {
        match (args.button, self.get_draw_state()) {
            (mouse::Left, Clicked) => {
                // Toggle value.
                self.value = if self.value == true { false } else { true };
                // Toggle rectangle color.
                self.rect.color = match self.value {
                    true => self.color,
                    false => Color::new(self.color.r * 0.1f32, self.color.g * 0.1f32, self.color.b * 0.1f32, self.color.a),
                };
                self.set_draw_state(Highlighted);
            },
            _ => (),
        }
    }

}

