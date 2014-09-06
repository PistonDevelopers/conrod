
use color::Color;
use frame::{
    Framing,
    Frame,
    NoFrame,
};
use label;
use label::{
    Labeling,
    Label,
    NoLabel,
};
use mouse_state::{
    MouseState,
    Up,
    Down,
};
use opengl_graphics::Gl;
use point::Point;
use rectangle;
use ui_context::{
    UIID,
    UIContext,
};
use utils::{
    clamp,
    percentage,
    value_from_perc,
};
use widget::Slider;

/// Represents the state of the Button widget.
#[deriving(PartialEq)]
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

widget_fns!(Slider, State, Slider(Normal))

/// Check the current state of the slider.
fn get_new_state(is_over: bool,
                 prev: State,
                 mouse: MouseState) -> State {
    match (is_over, prev, mouse) {
        (true, Normal, MouseState { left: Down, .. }) => Normal,
        (true, _, MouseState { left: Down, .. }) => Clicked,
        (true, _, MouseState { left: Up, .. }) => Highlighted,
        (false, Clicked, MouseState { left: Down, .. }) => Clicked,
        _ => Normal,
    }
}

/// A context on which the builder pattern can be implemented.
pub struct SliderContext<'a, T> {
    uic: &'a mut UIContext,
    value: T,
    min: T,
    max: T,
    new_value: T,
    is_over: bool,
    state: State,
    new_state: State,
    pos: Point<f64>,
    width: f64,
    height: f64,
    r_pos: Point<f64>,
    r_width: f64,
    r_height: f64,
    maybe_color: Option<Color>,
    maybe_frame: Option<(f64, Color)>,
    maybe_label: Option<(&'a str, u32, Color)>,
}

pub trait SliderBuilder<'a, T: Num + Copy + FromPrimitive + ToPrimitive> {
    /// A slider builder method to be implemented by the UIContext.
    fn slider(&'a mut self, ui_id: UIID, value: T, min: T, max: T,
              x: f64, y: f64, width: f64, height: f64) -> SliderContext<'a, T>;
}

impl<'a, T: Num + Copy + FromPrimitive + ToPrimitive>
SliderBuilder<'a, T> for UIContext {
    /// A button builder method to be implemented by the UIContext.
    fn slider(&'a mut self, ui_id: UIID, value: T, min: T, max: T,
              x: f64, y: f64, width: f64, height: f64) -> SliderContext<'a, T> {
        let pos = Point::new(x, y, 0.0);
        let state = *get_state(self, ui_id);
        let mouse = self.get_mouse_state();
        let is_over = rectangle::is_over(pos, mouse.pos, width, height);
        let new_state = get_new_state(is_over, state, mouse);
        set_state(self, ui_id, new_state);
        SliderContext {
            uic: self,
            value: value,
            min: min,
            max: max,
            new_value: value,
            is_over: is_over,
            state: state,
            new_state: new_state,
            pos: pos,
            width: width,
            height: height,
            r_pos: pos,
            r_width: width,
            r_height: height,
            maybe_color: None,
            maybe_frame: None,
            maybe_label: None,
        }
    }
}

impl<'a, T> ::color::Colorable<'a> for SliderContext<'a, T> {
    #[inline]
    fn color(self, r: f32, g: f32, b: f32, a: f32) -> SliderContext<'a, T> {
        SliderContext { maybe_color: Some(Color::new(r, g, b, a)), ..self }
    }
}

impl<'a, T> ::label::Labelable<'a> for SliderContext<'a, T> {
    #[inline]
    fn label(self, text: &'a str, size: u32,
             color: ::color::Color) -> SliderContext<'a, T> {
        SliderContext { maybe_label: Some((text, size, color)), ..self }
    }
}

impl<'a, T> ::position::Positionable for SliderContext<'a, T> {
    #[inline]
    fn position(self, x: f64, y: f64) -> SliderContext<'a, T> {
        SliderContext { pos: Point::new(x, y, 0.0), ..self }
    }
}

impl<'a, T: Num + Copy + FromPrimitive + ToPrimitive>
::frame::Frameable<'a> for SliderContext<'a, T> {
    #[inline]
    fn frame(self, width: f64, color: ::color::Color) -> SliderContext<'a, T> {
        let mouse = self.uic.get_mouse_state();
        let is_horizontal = self.width > self.height;
        let (r_pos, r_width, r_height) = if is_horizontal {
            // Horizontal.
            let p = self.pos + Point::new(width, width, 0f64);
            let max_width = self.width - (width * 2f64);
            let w = match (self.is_over, self.state, self.new_state) {
                (true, Highlighted, Clicked) | (_, Clicked, Clicked)  => {
                    clamp(mouse.pos.x - p.x, 0f64, max_width)
                },
                _ => clamp(percentage(self.value, self.min, self.max) as f64 * max_width,
                           0f64, max_width),
            };
            let h = self.height - (width * 2.0);
            (p, w, h)
        } else {
            // Vertical.
            let corner = self.pos + Point::new(width, width, 0.0);
            let max_height = self.height - (width * 2.0);
            let y_max = corner.y + max_height;
            let (h, p) = match (self.is_over, self.state, self.new_state) {
                (true, Highlighted, Clicked) | (_, Clicked, Clicked) => {
                    let p = Point::new(corner.x, clamp(mouse.pos.y, corner.y, y_max), 0.0);
                    let h = clamp(max_height - (p.y - corner.y), 0.0, max_height);
                    (h, p)
                },
                _ => {
                    let h = clamp(percentage(self.value, self.min, self.max) as f64 * max_height,
                                  0f64, max_height);
                    let p = Point::new(corner.x, corner.y + max_height - h, 0f64);
                    (h, p)
                },
            };
            let w = self.width - (width * 2f64);
            (p, w, h)
        };
        SliderContext {
            r_pos: r_pos, r_width: r_width, r_height: r_height,
            maybe_frame: Some((width, color)), ..self
        }
    }
}

impl<'a, T: Num + Copy + FromPrimitive + ToPrimitive>
::callback::Callable<|T|:'a> for SliderContext<'a, T> {
    #[inline]
    fn callback(self, callback: |T|) -> SliderContext<'a, T> {
        let mouse = self.uic.get_mouse_state();
        let frame_w = match self.maybe_frame { Some((frame_w, _)) => frame_w, None => 0.0 };
        let is_horizontal = self.width > self.height;
        let new_value = if is_horizontal {
            // Horizontal.
            let max_w = self.width - (frame_w * 2.0);
            value_from_perc((self.r_width / max_w) as f32, self.min, self.max)
        } else {
            // Vertical.
            let max_h = self.height - (frame_w * 2.0);
            value_from_perc((self.r_height / max_h) as f32, self.min, self.max)
        };
        if self.value != new_value || match (self.state, self.new_state) {
            (Highlighted, Clicked) | (Clicked, Highlighted) => true,
            _ => false,
        } { callback(new_value) };
        SliderContext { new_value: new_value, ..self }
    }
}

impl<'a, T> ::draw::Drawable for SliderContext<'a, T> {
    fn draw(&mut self, gl: &mut Gl) {
        let rect_state = self.new_state.as_rectangle_state();
        let color: Color = match self.maybe_color {
            None => ::std::default::Default::default(),
            Some(color) => color,
        };
        let (frame_w, frame_c) = match self.maybe_frame {
            Some((frame_w, frame_c)) => (frame_w, frame_c), None => (0.0, Color::black()),
        };
        // Rectangle frame / backdrop.
        rectangle::draw(self.uic.win_w, self.uic.win_h, gl, rect_state,
                        self.pos, self.width, self.height, None, frame_c);
        // Slider rectangle.
        rectangle::draw(self.uic.win_w, self.uic.win_h, gl, rect_state,
                        self.r_pos, self.r_width, self.r_height, None, color);
        match self.maybe_label {
            None => (),
            Some((text, size, text_color)) => {
                let is_horizontal = self.width > self.height;
                let l_pos = if is_horizontal {
                    let x = self.r_pos.x + (self.r_height - size as f64) / 2.0;
                    let y = self.r_pos.y + (self.r_height - size as f64) / 2.0;
                    Point::new(x, y, 0f64)
                } else {
                    let label_w = label::width(self.uic, size, text.as_slice());
                    let x = self.r_pos.x + (self.r_width - label_w) / 2.0;
                    let y = self.r_pos.y + self.r_height - self.r_width - frame_w;
                    Point::new(x, y, 0f64)
                };
                // Draw the label.
                label::draw(gl, self.uic, l_pos, size, text_color, text.as_slice());
            },
        }
    }
}

