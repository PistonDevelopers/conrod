
use color::Color;
use label;
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
#[deriving(PartialEq, Clone)]
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
    ui_id: UIID,
    value: T,
    min: T,
    max: T,
    pos: Point<f64>,
    width: f64,
    height: f64,
    maybe_callback: Option<|T|:'a>,
    maybe_color: Option<Color>,
    maybe_frame: Option<(f64, Color)>,
    maybe_label: Option<(&'a str, u32, Color)>,
}

pub trait SliderBuilder<'a, T: Num + Copy + FromPrimitive + ToPrimitive> {
    /// A slider builder method to be implemented by the UIContext.
    fn slider(&'a mut self, ui_id: UIID,
              value: T, min: T, max: T) -> SliderContext<'a, T>;
}

impl<'a, T: Num + Copy + FromPrimitive + ToPrimitive>
SliderBuilder<'a, T> for UIContext {
    /// A button builder method to be implemented by the UIContext.
    fn slider(&'a mut self, ui_id: UIID,
              value: T, min: T, max: T) -> SliderContext<'a, T> {
        SliderContext {
            uic: self,
            ui_id: ui_id,
            value: value,
            min: min,
            max: max,
            pos: Point::new(0.0, 0.0, 0.0),
            width: 192.0,
            height: 48.0,
            maybe_callback: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_label: None,
        }
    }
}

impl_callable!(SliderContext, |T|:'a, T)
impl_colorable!(SliderContext, T)
impl_frameable!(SliderContext, T)
impl_labelable!(SliderContext, T)
impl_positionable!(SliderContext, T)
impl_shapeable!(SliderContext, T)

impl<'a, T: Num + Copy + FromPrimitive + ToPrimitive>
::draw::Drawable for SliderContext<'a, T> {
    fn draw(&mut self, gl: &mut Gl) {

        let state = *get_state(self.uic, self.ui_id);
        let mouse = self.uic.get_mouse_state();
        let is_over = rectangle::is_over(self.pos, mouse.pos, self.width, self.height);
        let new_state = get_new_state(is_over, state, mouse);

        let (frame_w, frame_c) = match self.maybe_frame {
            Some((frame_w, frame_c)) => (frame_w, frame_c), None => (0.0, Color::black()),
        };
        let frame_w2 = frame_w * 2.0;

        let is_horizontal = self.width > self.height;
        let (new_value, pad_pos, pad_w, pad_h) = if is_horizontal {
            // Horizontal.
            let p = self.pos + Point::new(frame_w, frame_w, 0.0);
            let max_w = self.width - frame_w2;
            let w = match (is_over, state, new_state) {
                (true, Highlighted, Clicked) | (_, Clicked, Clicked)  =>
                     clamp(mouse.pos.x - p.x, 0f64, max_w),
                _ => clamp(percentage(self.value, self.min, self.max) as f64 * max_w, 0f64, max_w),
            };
            let h = self.height - frame_w2;
            let new_value = value_from_perc((w / max_w) as f32, self.min, self.max);
            (new_value, p, w, h)
        } else {
            // Vertical.
            let max_h = self.height - frame_w2;
            let corner = self.pos + Point::new(frame_w, frame_w, 0.0);
            let y_max = corner.y + max_h;
            let (h, p) = match (is_over, state, new_state) {
                (true, Highlighted, Clicked) | (_, Clicked, Clicked) => {
                    let p = Point::new(corner.x, clamp(mouse.pos.y, corner.y, y_max), 0.0);
                    let h = clamp(max_h - (p.y - corner.y), 0.0, max_h);
                    (h, p)
                },
                _ => {
                    let h = clamp(percentage(self.value, self.min, self.max) as f64 * max_h, 0.0, max_h);
                    let p = Point::new(corner.x, corner.y + max_h - h, 0.0);
                    (h, p)
                },
            };
            let w = self.width - frame_w2;
            let new_value = value_from_perc((h / max_h) as f32, self.min, self.max);
            (new_value, p, w, h)
        };

        // Callback.
        match self.maybe_callback {
            Some(ref mut callback) => {
                if self.value != new_value || match (state, new_state) {
                    (Highlighted, Clicked) | (Clicked, Highlighted) => true,
                    _ => false,
                } { (*callback)(new_value) }
            }, None => (),
        }

        // Draw.
        let rect_state = new_state.as_rectangle_state();
        let color = self.maybe_color.unwrap_or(::std::default::Default::default());

        // Rectangle frame / backdrop.
        rectangle::draw(self.uic.win_w, self.uic.win_h, gl, rect_state,
                        self.pos, self.width, self.height, None, frame_c);
        // Slider rectangle.
        rectangle::draw(self.uic.win_w, self.uic.win_h, gl, rect_state,
                        pad_pos, pad_w, pad_h, None, color);

        match self.maybe_label {
            None => (),
            Some((text, size, text_color)) => {
                let is_horizontal = self.width > self.height;
                let l_pos = if is_horizontal {
                    let x = pad_pos.x + (pad_h - size as f64) / 2.0;
                    let y = pad_pos.y + (pad_h - size as f64) / 2.0;
                    Point::new(x, y, 0f64)
                } else {
                    let label_w = label::width(self.uic, size, text.as_slice());
                    let x = pad_pos.x + (pad_w - label_w) / 2.0;
                    let y = pad_pos.y + pad_h - pad_w - frame_w;
                    Point::new(x, y, 0f64)
                };
                // Draw the label.
                label::draw(gl, self.uic, l_pos, size, text_color, text.as_slice());
            },
        }

        set_state(self.uic, self.ui_id, new_state,
                  self.pos.x, self.pos.y, self.width, self.height);

    }
}

