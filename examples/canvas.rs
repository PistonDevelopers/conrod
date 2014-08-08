
use std::default::Default;
use conrod::{
    Button,
    Slider,
    Point,
    Widget,
    Down,
    Right,
    Color,
    Toggle,
    WidgetData,
};

pub struct Canvas {
    widget_data: WidgetData,
    button: Button,
    toggle: Toggle,
    slider: Slider<int>,
}

impl Canvas {

    /// Create a new UI Widget Canvas.
    pub fn new() -> Canvas {
        let button: Button = Default::default();
        let toggle = Toggle::new(Right(100u), // Relative Position to previous child.
                                 75u, // Width
                                 75u, // Height
                                 6u, // Border
                                 Color::new(0.65f32, 0.3f32, 0.7f32, 1f32), // Color
                                 false); // Initial toggle state
        let slider = Slider::new(Down(175u), // Relative Position to previous child.
                                 40u, // Width
                                 200u, // Height
                                 3u, // Border
                                 Color::new(0.5f32, 0.6f32, 0.8f32, 1f32), // Color
                                 0, // Minimum slider value
                                 128, // Maximum slider value
                                 80); // Initial slider value
        let mut canvas = Canvas {
            widget_data: Default::default(),
            button: button,
            slider: slider,
            toggle: toggle,
        };
        canvas.set_abs_pos(Point::new(50i, 50, 0));
        canvas
    }

}

impl Widget for Canvas {

    impl_get_widget_data!(widget_data)

    fn get_children(&self) -> Vec<&Widget> {
        vec![&self.button as &Widget,
             &self.toggle as &Widget,
             &self.slider as &Widget]
    }

    fn get_children_mut(&mut self) -> Vec<&mut Widget> {
        vec![&mut self.button as &mut Widget,
             &mut self.toggle as &mut Widget,
             &mut self.slider as &mut Widget]
    }

}

