
use color::{Color, Colorable};
use frame::Frameable;
use graphics::character::CharacterCache;
use label::{FontSize, Labelable};
use mouse::Mouse;
use num::{Float, NumCast, ToPrimitive};
use position::{self, Depth, Dimensions, HorizontalAlign, Position, VerticalAlign};
use ui::{UiId, Ui};
use utils::{clamp, percentage, value_from_perc};
use widget::Kind;

/// Represents the state of the Button widget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Normal,
    Highlighted,
    Clicked,
}

impl State {
    /// Return the color associated with the state.
    fn color(&self, color: Color) -> Color {
        match self {
            &State::Normal => color,
            &State::Highlighted => color.highlighted(),
            &State::Clicked => color.clicked(),
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
    value: T,
    min: T,
    max: T,
    pos: Position,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    dim: Dimensions,
    depth: Depth,
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
    pub fn new(value: T, min: T, max: T) -> Slider<'a, T, F> {
        Slider {
            value: value,
            min: min,
            max: max,
            pos: Position::default(),
            maybe_h_align: None,
            maybe_v_align: None,
            dim: [192.0, 48.0],
            depth: 0.0,
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

    /// After building the Button, use this method to set its current state into the given `Ui`.
    /// It will use this state for rendering the next time `ui.draw(graphics)` is called.
    pub fn set<C>(mut self, ui_id: UiId, ui: &mut Ui<C>)
        where
            C: CharacterCache,
            F: FnMut(T),
            T: Float + NumCast + ToPrimitive,
    {
        use elmesque::form::{collage, rect, text};
        use utils::{is_over_rect, map_range};

        let state = *get_state(ui, ui_id);
        let dim = self.dim;
        let h_align = self.maybe_h_align.unwrap_or(ui.theme.h_align);
        let v_align = self.maybe_v_align.unwrap_or(ui.theme.v_align);
        let xy = ui.get_xy(self.pos, dim, h_align, v_align);
        let mouse = ui.get_mouse_state().relative_to(xy);
        let is_over = is_over_rect([0.0, 0.0], mouse.xy, dim);
        let new_state = get_new_state(is_over, state, mouse);

        let frame_w = self.maybe_frame.unwrap_or(ui.theme.frame_width);
        let frame_w2 = frame_w * 2.0;
        let (inner_w, inner_h) = (dim[0] - frame_w2, dim[1] - frame_w2);
        let (half_inner_w, half_inner_h) = (inner_w / 2.0, inner_h / 2.0);

        let is_horizontal = dim[0] > dim[1];
        let (new_value, pad_rel_xy, pad_dim) = if is_horizontal {
            // Horizontal.
            let w = match (is_over, state, new_state) {
                (true, State::Highlighted, State::Clicked) | (_, State::Clicked, State::Clicked) => {
                    let w = map_range(mouse.xy[0], -half_inner_w, half_inner_w, 0.0, inner_w);
                    clamp(w, 0.0, inner_w)
                },
                _ => {
                    let value_percentage = percentage(self.value, self.min, self.max);
                    clamp(value_percentage as f64 * inner_w, 0.0, inner_w)
                },
            };
            let new_value = value_from_perc((w / inner_w) as f32, self.min, self.max);
            let p = [-(inner_w - w) / 2.0, 0.0];
            (new_value, p, [w, inner_h])
        } else {
            // Vertical.
            let h = match (is_over, state, new_state) {
                (true, State::Highlighted, State::Clicked) | (_, State::Clicked, State::Clicked) => {
                    let h = map_range(mouse.xy[1], -half_inner_h, half_inner_h, 0.0, inner_h);
                    clamp(h, 0.0, inner_h)
                },
                _ => {
                    let value_percentage = percentage(self.value, self.min, self.max);
                    clamp(value_percentage as f64 * inner_h, 0.0, inner_h)
                },
            };
            let new_value = value_from_perc((h / inner_h) as f32, self.min, self.max);
            let rel_xy = [0.0, -(inner_h - h) / 2.0];
            (new_value, rel_xy, [inner_w, h])
        };

        // Callback.
        match self.maybe_callback {
            Some(ref mut callback) => {
                if self.value != new_value || match (state, new_state) {
                    (State::Highlighted, State::Clicked) | (State::Clicked, State::Highlighted) => true,
                    _ => false,
                } { callback(new_value) }
            }, None => (),
        }

        // Draw.
        let frame_color = new_state.color(self.maybe_frame_color.unwrap_or(ui.theme.frame_color));
        let color = new_state.color(self.maybe_color.unwrap_or(ui.theme.shape_color));

        // Rectangle frame / backdrop Form.
        let frame_form = rect(dim[0], dim[1])
            .filled(frame_color);
        // Slider rectangle Form.
        let pad_form = rect(pad_dim[0], pad_dim[1])
            .filled(color)
            .shift(pad_rel_xy[0], pad_rel_xy[1]);

        // Label Form.
        let maybe_label_form = self.maybe_label.map(|label_text| {
            use elmesque::text::Text;
            use label;
            const TEXT_PADDING: f64 = 10.0;
            let text_color = self.maybe_label_color.unwrap_or(ui.theme.label_color);
            let size = self.maybe_label_font_size.unwrap_or(ui.theme.font_size_medium);
            let label_w = label::width(ui, size, &label_text);
            let is_horizontal = self.dim[0] > self.dim[1];
            let l_pos = if is_horizontal {
                let x = position::align_left_of(dim[0], label_w) + TEXT_PADDING;
                [x, 0.0]
            } else {
                let y = position::align_bottom_of(dim[1], size as f64) + TEXT_PADDING;
                [0.0, y]
            };
            text(Text::from_string(label_text.to_string()).color(text_color).height(size as f64))
                .shift(l_pos[0].floor(), l_pos[1].floor())
                .shift(xy[0].floor(), xy[1].floor())
        });

        // Chain the Forms and shift them into position.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(pad_form).into_iter())
            .map(|form| form.shift(xy[0], xy[1]))
            .chain(maybe_label_form.into_iter());

        // Collect the Forms into a renderable Element.
        let element = collage(dim[0] as i32, dim[1] as i32, form_chain.collect());

        // Store the slider's state in the `Ui`.
        ui.set_widget(ui_id, ::widget::Widget {
            kind: Kind::Slider(new_state),
            xy: xy,
            depth: self.depth,
            element: Some(element),
        });

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

impl<'a, T, F> position::Positionable for Slider<'a, T, F> {
    #[inline]
    fn horizontal_align(self, h_align: HorizontalAlign) -> Self {
        Slider { maybe_h_align: Some(h_align), ..self }
    }
    #[inline]
    fn vertical_align(self, v_align: VerticalAlign) -> Self {
        Slider { maybe_v_align: Some(v_align), ..self }
    }
    fn position(mut self, pos: Position) -> Self {
        self.pos = pos;
        self
    }
}

impl<'a, T, F> position::Sizeable for Slider<'a, T, F> {
    #[inline]
    fn width(self, w: f64) -> Self {
        let h = self.dim[1];
        Slider { dim: [w, h], ..self }
    }
    #[inline]
    fn height(self, h: f64) -> Self {
        let w = self.dim[0];
        Slider { dim: [w, h], ..self }
    }
}

