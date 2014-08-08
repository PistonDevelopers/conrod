
use widget;
use widget::{
    Clicked,
    Highlighted,
    Normal,
    Widget,
    RelativePosition,
    Relative,
};
use piston::{
    MouseMoveArgs,
    MousePressArgs,
};
use rectangle::Rectangle;
use point::Point;
use color::Color;
use utils::clamp;
use std::default::Default;

/// A generic slider user-interface widget. It will
/// automatically convert itself between horizontal
/// and vertical depending on the dimensions given.
#[deriving(Show, Clone)]
pub struct Slider<T> {
    widget_data: widget::Data,
    frame: Rectangle,
    rect: Rectangle,
    border: uint,
    min: T,
    max: T,
    value: T,
    last_pos: Point<int>,
}

impl<T: Num + Copy + FromPrimitive + ToPrimitive> Slider<T> {

    /// Constructor for a Slider widget.
    pub fn new(pos: RelativePosition,
               width: uint,
               height: uint,
               border: uint,
               color: Color,
               min: T,
               max: T,
               value: T) -> Slider<T> {
        let perc = Slider::percentage(value, min, max);
        let slider_rect = match width >= height {
            true => { // Horizontal Slider...
                Rectangle::new(Relative(Point::new(border as int, border as int, 0)),
                               ((width - 2u * border) as f32 * perc) as uint,
                               height - 2u * border, color, 0u)
            },
            false => { // Vertical Slider...
                let perc_alt = ::std::num::abs(perc - 1f32);
                let y = border as int + (perc_alt * (height - 2u * border) as f32) as int;
                Rectangle::new(Relative(Point::new(border as int, y, 0)),
                               width - 2u * border,
                               ((height - 2u * border) as f32 * perc) as uint,
                               color, 0u)
            },
        };
        Slider {
            widget_data: widget::Data::new(pos),
            frame: Rectangle::new(Relative(Point::new(0, 0, 0)), width, height, Color::black(), 0u),
            rect: slider_rect,
            border: border,
            min: min,
            max: max,
            value: value,
            last_pos: Default::default(),
        }
    }

    /// Get value percentage between max and min.
    fn percentage(value: T, min: T, max: T) -> f32 {
        let v = value.to_f32().unwrap();
        let mn = min.to_f32().unwrap();
        let mx = max.to_f32().unwrap();
        (v - mn) / (mx - mn)
    }

    /// Adjust the slider to the given point.
    fn adjust_slider(&mut self, p: Point<int>) {
        match self.get_draw_state() {
            Clicked => {
                match self.frame.width > self.frame.height {
                    true => {
                        // Horizontal Slider.
                        let width = (p - self.get_abs_pos()).x - self.border as int;
                        let frame_width = self.frame.width;
                        let border = self.border;
                        self.rect.width = clamp(width, 0, frame_width as int - border as int * 2) as uint;
                        self.adjust_value(width as f32 / (frame_width as int - border as int * 2) as f32);
                    },
                    false => {
                        // Vertical Slider.
                        let y_min = self.get_abs_pos().y + self.border as int;
                        let y_max = self.get_abs_pos().y + self.frame.height as int - self.border as int;
                        let new_y = clamp(p.y, y_min, y_max);
                        let height = (self.get_abs_pos().y + self.frame.height as int - self.border as int) - new_y;
                        self.rect.height = height as uint;
                        let x = self.rect.get_abs_pos().x;
                        self.rect.set_abs_pos(Point::new(x, new_y, 0));
                        self.adjust_value((new_y - y_min) as f32 / (y_max - y_min) as f32);
                    }
                }
            },
            _ => (),
        }
    }

    /// Adjust the value to the given percentage.
    fn adjust_value(&mut self, perc: f32) {
        self.value = FromPrimitive::from_f32((self.max - self.min).to_f32().unwrap() * perc).unwrap();
    }

}

impl<T: Num + Copy + FromPrimitive + ToPrimitive> Widget for Slider<T> {

    impl_get_widget_data!(widget_data)

    /// Return the dimensions as a tuple holding width and height.
    fn get_dimensions(&self) -> (uint, uint) { self.frame.get_dimensions() }

    /// Return a reference to the rectangle as a widget child.
    fn get_children(&self) -> Vec<&Widget> {
        vec![&self.frame as &Widget, &self.rect as &Widget]
    }

    /// Return all children widgets.
    fn get_children_mut(&mut self) -> Vec<&mut Widget> {
        vec![&mut self.frame as &mut Widget, &mut self.rect as &mut Widget]
    }

    /// Return whether or not the widget has been hit by a mouse_press.
    fn is_over(&self, mouse_pos: Point<int>) -> bool {
        self.frame.is_over(mouse_pos)
    }

    /// Mouse move event.
    fn mouse_move(&mut self, args: &MouseMoveArgs) {
        self.mouse_move_update_draw_state(args);
        self.mouse_move_children(args);
        let p = Point::new(args.x as int, args.y as int, 0);
        self.adjust_slider(p);
        match self.get_draw_state() {
            Highlighted | Clicked => { self.last_pos = p; },
            Normal => (),
        };
    }

    /// Mouse Press event.
    fn mouse_press(&mut self, args: &MousePressArgs) {
        self.mouse_press_update_draw_state(args);
        self.mouse_press_children(args);
        match self.get_draw_state() {
            Highlighted | Clicked => {
                let p = self.last_pos;
                self.adjust_slider(p)
            },
            _ => ()
        }
    }

}

