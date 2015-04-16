
use color::{Color, Colorable};
use dimensions::Dimensions;
use frame::Frameable;
use label::{self, FontSize, Labelable};
use mouse::Mouse;
use graphics::Graphics;
use graphics::character::CharacterCache;
use num::{Float, ToPrimitive, FromPrimitive};
use point::Point;
use position::Positionable;
use rectangle;
use shape::Shapeable;
use ui::{UiId, Ui};
use utils::{clamp, percentage, value_from_perc};
use vecmath::vec2_add;
use widget::Kind;

/// Represents the state of the Button widget.
#[derive(PartialEq, Clone, Copy)]
pub enum State {
    Normal,
    Highlighted,
    Clicked,
}

impl State {
    /// Return the associated Rectangle state.
    fn as_rectangle_state(&self) -> rectangle::State {
        match self {
            &State::Normal => rectangle::State::Normal,
            &State::Highlighted => rectangle::State::Highlighted,
            &State::Clicked => rectangle::State::Clicked,
        }
    }
}

widget_fns!(Slider, State, Kind::Slider(State::Normal));

/// Check the current state of the slider.
fn get_new_state(is_over: bool,
                 prev: State,
                 mouse: Mouse) -> State {
    use mouse::ButtonState::{Down, Up};
    use self::State::{Normal, Highlighted, Clicked};
    match (is_over, prev, mouse.left) {
        (true,  Normal,  Down) => Normal,
        (true,  _,       Down) => Clicked,
        (true,  _,       Up)   => Highlighted,
        (false, Clicked, Down) => Clicked,
        _ => Normal,
    }
}

/// Linear value selection. If the slider's width is greater than it's height, it will
/// automatically become a horizontal slider, otherwise it will be a vertical slider. Its callback
/// is triggered if the value is updated or if the mouse button is released while the cursor is
/// above the rectangle.
pub struct Slider<'a, T, F> {
    ui_id: UiId,
    value: T,
    min: T,
    max: T,
    pos: Point,
    dim: Dimensions,
    maybe_callback: Option<F>,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_label: Option<&'a str>,
    maybe_label_color: Option<Color>,
    maybe_label_font_size: Option<u32>,
}

impl<'a, T, F> Slider<'a, T, F> {

    /// Construct a new Slider widget.
    pub fn new(ui_id: UiId, value: T, min: T, max: T) -> Slider<'a, T, F> {
        Slider {
            ui_id: ui_id,
            value: value,
            min: min,
            max: max,
            pos: [0.0, 0.0],
            dim: [192.0, 48.0],
            maybe_callback: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
        }
    }

    /// Set the callback for the Slider. It will be triggered if the value is updated or if the
    /// mouse button is released while the cursor is above the rectangle.
    pub fn callback(mut self, cb: F) -> Slider<'a, T, F> {
        self.maybe_callback = Some(cb);
        self
    }

}

impl<'a, T, F> Colorable for Slider<'a, T, F> {
    fn color(mut self, color: Color) -> Self {
        self.maybe_color = Some(color);
        self
    }
}

impl<'a, T, F> Frameable for Slider<'a, T, F> {
    fn frame(mut self, width: f64) -> Self {
        self.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, T, F> Labelable<'a> for Slider<'a, T, F>
{
    fn label(mut self, text: &'a str) -> Self {
        self.maybe_label = Some(text);
        self
    }

    fn label_color(mut self, color: Color) -> Self {
        self.maybe_label_color = Some(color);
        self
    }

    fn label_font_size(mut self, size: FontSize) -> Self {
        self.maybe_label_font_size = Some(size);
        self
    }
}

impl<'a, T, F> Positionable for Slider<'a, T, F> {
    fn point(mut self, pos: Point) -> Self {
        self.pos = pos;
        self
    }
}

impl<'a, T, F> Shapeable for Slider<'a, T, F> {
    fn get_dim(&self) -> Dimensions { self.dim }
    fn dim(mut self, dim: Dimensions) -> Self { self.dim = dim; self }
}

impl<'a, T, F> ::draw::Drawable for Slider<'a, T, F>
    where
        T: Float + FromPrimitive + ToPrimitive,
        F: FnMut(T) + 'a
{

    fn draw<B, C>(&mut self, ui: &mut Ui<C>, graphics: &mut B)
        where
            B: Graphics<Texture = <C as CharacterCache>::Texture>,
            C: CharacterCache
    {

        let state = *get_state(ui, self.ui_id);
        let mouse = ui.get_mouse_state();
        let is_over = rectangle::is_over(self.pos, mouse.pos, self.dim);
        let new_state = get_new_state(is_over, state, mouse);

        let frame_w = self.maybe_frame.unwrap_or(ui.theme.frame_width);
        let frame_w2 = frame_w * 2.0;
        let frame_color = self.maybe_frame_color.unwrap_or(ui.theme.frame_color);

        let is_horizontal = self.dim[0] > self.dim[1];
        let (new_value, pad_pos, pad_dim) = if is_horizontal {
            // Horizontal.
            let p = vec2_add(self.pos, [frame_w, frame_w]);
            let max_w = self.dim[0] - frame_w2;
            let w = match (is_over, state, new_state) {
                (true, State::Highlighted, State::Clicked) | (_, State::Clicked, State::Clicked)  =>
                     clamp(mouse.pos[0] - p[0], 0f64, max_w),
                _ => clamp(percentage(self.value, self.min, self.max) as f64 * max_w, 0f64, max_w),
            };
            let h = self.dim[1] - frame_w2;
            let new_value = value_from_perc((w / max_w) as f32, self.min, self.max);
            (new_value, p, [w, h])
        } else {
            // Vertical.
            let max_h = self.dim[1] - frame_w2;
            let corner = vec2_add(self.pos, [frame_w, frame_w]);
            let y_max = corner[1] + max_h;
            let (h, p) = match (is_over, state, new_state) {
                (true, State::Highlighted, State::Clicked) | (_, State::Clicked, State::Clicked) => {
                    let p = [corner[0], clamp(mouse.pos[1], corner[1], y_max)];
                    let h = clamp(max_h - (p[1] - corner[1]), 0.0, max_h);
                    (h, p)
                },
                _ => {
                    let h = clamp(percentage(self.value, self.min, self.max) as f64 * max_h, 0.0, max_h);
                    let p = [corner[0], corner[1] + max_h - h];
                    (h, p)
                },
            };
            let w = self.dim[0] - frame_w2;
            let new_value = value_from_perc((h / max_h) as f32, self.min, self.max);
            (new_value, p, [w, h])
        };

        // Callback.
        match self.maybe_callback {
            Some(ref mut callback) => {
                if self.value != new_value || match (state, new_state) {
                    (State::Highlighted, State::Clicked) | (State::Clicked, State::Highlighted) => true,
                    _ => false,
                } { (*callback)(new_value) }
            }, None => (),
        }

        // Draw.
        let rect_state = new_state.as_rectangle_state();
        let color = self.maybe_color.unwrap_or(ui.theme.shape_color);

        // Rectangle frame / backdrop.
        rectangle::draw(ui.win_w, ui.win_h, graphics, rect_state,
                        self.pos, self.dim, None, frame_color);
        // Slider rectangle.
        rectangle::draw(ui.win_w, ui.win_h, graphics, rect_state,
                        pad_pos, pad_dim, None, color);

        // If there's a label, draw it.
        if let Some(text) = self.maybe_label {
            let text_color = self.maybe_label_color.unwrap_or(ui.theme.label_color);
            let size = self.maybe_label_font_size.unwrap_or(ui.theme.font_size_medium);
            let is_horizontal = self.dim[0] > self.dim[1];
            let l_pos = if is_horizontal {
                let x = pad_pos[0] + (pad_dim[1] - size as f64) / 2.0;
                let y = pad_pos[1] + (pad_dim[1] - size as f64) / 2.0;
                [x, y]
            } else {
                let label_w = label::width(ui, size, &text);
                let x = pad_pos[0] + (pad_dim[0] - label_w) / 2.0;
                let y = pad_pos[1] + pad_dim[1] - pad_dim[0] - frame_w;
                [x, y]
            };
            // Draw the label.
            ui.draw_text(graphics, l_pos, size, text_color, &text);
        }

        set_state(ui, self.ui_id, Kind::Slider(new_state), self.pos, self.dim);

    }
}
