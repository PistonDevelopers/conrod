
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

/// Button type widget.
#[deriving(Show, Clone)]
pub struct Button {
    widget_data: widget::Data,
    rect: Rectangle,
}

impl Button {

    /// Constructor for Button widget.
    pub fn new(pos: RelativePosition, width: uint, height: uint, color: Color, border: uint) -> Button {
        Button {
            widget_data: widget::Data::new(pos),
            rect: Rectangle::new(pos, width, height, color, border),
        }
    }

}

impl Default for Button {

    /// Default constructor for Button widget.
    fn default() -> Button {
        Button {
            widget_data: Default::default(),
            rect: Default::default(),
        }
    }

}

impl Widget for Button {

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
                // TRIGGER EVENT HERE.
                self.set_draw_state(Highlighted);
            },
            _ => (),
        }
    }

}

