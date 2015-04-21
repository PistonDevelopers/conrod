
use color::{Color, Colorable};
use frame::Frameable;
use graphics::character::CharacterCache;
use label::{self, FontSize, Labelable};
use mouse::Mouse;
use num::{Float, NumCast};
use position::{self, Corner, Depth, Dimensions, HorizontalAlign, Position, VerticalAlign};
use ui::{UiId, Ui};
use utils::{clamp, map_range, val_to_string};
use vecmath::vec2_sub;
use widget::Kind;

/// Represents the state of the xy_pad widget.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum State {
    Normal,
    Highlighted,
    Clicked,
}

impl State {
    /// The color associated with the current state.
    fn color(&self, color: Color) -> Color {
        match self {
            &State::Normal => color,
            &State::Highlighted => color.highlighted(),
            &State::Clicked => color.clicked(),
        }
    }
}

widget_fns!(XYPad, State, Kind::XYPad(State::Normal));

/// Check the current state of the button.
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
        _                      => Normal,
    }
}

/// Used for displaying and controlling a 2D point on a cartesian plane within a given range.
/// Its reaction is triggered when the value is updated or if the mouse button is released while
/// the cursor is above the rectangle.
pub struct XYPad<'a, X, Y, F> {
    x: X, min_x: X, max_x: X,
    y: Y, min_y: Y, max_y: Y,
    line_width: f64,
    value_font_size: FontSize,
    pos: Position,
    dim: Dimensions,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    depth: Depth,
    maybe_react: Option<F>,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_label: Option<&'a str>,
    maybe_label_color: Option<Color>,
    maybe_label_font_size: Option<u32>,
}

impl<'a, X, Y, F> XYPad<'a, X, Y, F> {

    /// Construct a new XYPad widget.
    pub fn new(x_val: X, min_x: X, max_x: X, y_val: Y, min_y: Y, max_y: Y) -> XYPad<'a, X, Y, F> {
        XYPad {
            x: x_val, min_x: min_x, max_x: max_x,
            y: y_val, min_y: min_y, max_y: max_y,
            line_width: 1.0,
            value_font_size: 14u32,
            pos: Position::default(),
            dim: [128.0, 128.0],
            maybe_h_align: None,
            maybe_v_align: None,
            depth: 0.0,
            maybe_react: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
        }
    }

    /// Set the width of the XYPad's crosshair lines.
    #[inline]
    pub fn line_width(self, width: f64) -> XYPad<'a, X, Y, F> {
        XYPad { line_width: width, ..self }
    }

    /// Set the font size for the displayed crosshair value.
    #[inline]
    pub fn value_font_size(self, size: FontSize) -> XYPad<'a, X, Y, F> {
        XYPad { value_font_size: size, ..self }
    }

    /// Set the reaction for the XYPad. It will be triggered when the value is updated or if the
    /// mouse button is released while the cursor is above the rectangle.
    pub fn react(mut self, reaction: F) -> Self {
        self.maybe_react = Some(reaction);
        self
    }

    /// After building the Button, use this method to set its current state into the given `Ui`.
    /// It will use this state for rendering the next time `ui.draw(graphics)` is called.
    pub fn set<C>(mut self, ui_id: UiId, ui: &mut Ui<C>)
        where
            C: CharacterCache,
            F: FnMut(X, Y),
            X: Float + NumCast + ToString,
            Y: Float + NumCast + ToString,
    {
        use elmesque::form::{collage, line, rect, solid, text};
        use elmesque::text::Text;
        use utils::is_over_rect;

        let state = *get_state(ui, ui_id);
        let dim = self.dim;
        let h_align = self.maybe_h_align.unwrap_or(ui.theme.h_align);
        let v_align = self.maybe_v_align.unwrap_or(ui.theme.v_align);
        let xy = ui.get_xy(self.pos, dim, h_align, v_align);
        let mouse = ui.get_mouse_state(ui_id).relative_to(xy);
        let frame_w = self.maybe_frame.unwrap_or(ui.theme.frame_width);
        let frame_w2 = frame_w * 2.0;
        let pad_dim = vec2_sub(dim, [frame_w2; 2]);
        let is_over_pad = is_over_rect([0.0, 0.0], mouse.xy, pad_dim);
        let new_state = get_new_state(is_over_pad, state, mouse);
        let half_pad_w = pad_dim[0] / 2.0;
        let half_pad_h = pad_dim[1] / 2.0;

        // Determine new values from the mouse position over the pad.
        let (new_x, new_y) = match (is_over_pad, new_state) {
            (_, State::Normal) | (_, State::Highlighted) => (self.x, self.y),
            (_, State::Clicked) => {
                let temp_x = clamp(mouse.xy[0], -half_pad_w, half_pad_w);
                let temp_y = clamp(mouse.xy[1], -half_pad_h, half_pad_h);
                (map_range(temp_x, -half_pad_w, half_pad_w, self.min_x, self.max_x),
                 map_range(temp_y, -half_pad_h, half_pad_h, self.min_y, self.max_y))
            }
        };

        // React if value is changed or the pad is clicked/released.
        if let Some(ref mut react) = self.maybe_react {
            if self.x != new_x || self.y != new_y { react(new_x, new_y) }
            else {
                match (state, new_state) {
                    (State::Highlighted, State::Clicked) |
                    (State::Clicked, State::Highlighted) => react(new_x, new_y),
                    _ => (),
                }
            }
        }

        // Construct the frame and inner rectangle Forms.
        let color = new_state.color(self.maybe_color.unwrap_or(ui.theme.shape_color));
        let frame_color = self.maybe_frame_color.unwrap_or(ui.theme.frame_color);
        let frame_form = rect(dim[0], dim[1]).filled(frame_color);
        let pressable_form = rect(pad_dim[0], pad_dim[1]).filled(color);

        // Construct the label Form.
        let maybe_label_form = self.maybe_label.map(|l_text| {
            let l_color = self.maybe_label_color.unwrap_or(ui.theme.label_color);
            let l_size = self.maybe_label_font_size.unwrap_or(ui.theme.font_size_medium);
            text(Text::from_string(l_text.to_string()).color(l_color).height(l_size as f64))
        });

        // Construct the crosshair line Forms.
        let (ch_x, ch_y) = match (is_over_pad, new_state) {
            (_, State::Normal) | (_, State::Highlighted) =>
                (map_range(new_x, self.min_x, self.max_x, -half_pad_w, half_pad_w).floor(),
                 map_range(new_y, self.min_y, self.max_y, -half_pad_h, half_pad_h).floor()),
            (_, State::Clicked) =>
                (clamp(mouse.xy[0], -half_pad_w, half_pad_w).floor(),
                 clamp(mouse.xy[1], -half_pad_h, half_pad_h).floor()),
        };
        let line_style = solid(color.plain_contrast()).width(self.line_width);
        let vert_form = line(line_style.clone(), 0.0, -half_pad_h, 0.0, half_pad_h).shift_x(ch_x);
        let hori_form = line(line_style, -half_pad_w, 0.0, half_pad_w, 0.0).shift_y(ch_y);

        // Construct the value string Form.
        let x_string = val_to_string(self.x, self.max_x, self.max_x - self.min_x, dim[0] as usize);
        let y_string = val_to_string(self.y, self.max_y, self.max_y - self.min_y, dim[1] as usize);
        let value_string = format!("{}, {}", x_string, y_string);
        let value_text_form = {
            const PAD: f64 = 5.0; // Slight padding between the crosshair and the text.
            let w = label::width(ui, self.value_font_size, &value_string);
            let h = self.value_font_size as f64;
            let x_shift = w / 2.0 + PAD;
            let y_shift = h / 2.0 + PAD;
            let (value_text_x, value_text_y) = match position::corner([ch_x, ch_y], pad_dim) {
                Corner::TopLeft => (x_shift, -y_shift),
                Corner::TopRight => (-x_shift, -y_shift),
                Corner::BottomLeft => (x_shift, y_shift),
                Corner::BottomRight => (-x_shift, y_shift),
            };
            text(Text::from_string(value_string).color(color.plain_contrast()).height(h))
                .shift(ch_x, ch_y)
                .shift(value_text_x.floor(), value_text_y.floor())
        };

        // Chain the Forms and shift them into position.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(pressable_form).into_iter())
            .chain(maybe_label_form.into_iter())
            .chain(Some(vert_form).into_iter())
            .chain(Some(hori_form).into_iter())
            .chain(Some(value_text_form).into_iter())
            .map(|form| form.shift(xy[0].round(), xy[1].round()));

        // Turn the form into a renderable Element.
        let element = collage(dim[0] as i32, dim[1] as i32, form_chain.collect());

        // Store the XYPad's new state in the Ui.
        ui.set_widget(ui_id, ::widget::Widget {
            kind: Kind::XYPad(new_state),
            xy: xy,
            depth: self.depth,
            element: Some(element),
        });

    }

}

impl<'a, X, Y, F> Colorable for XYPad<'a, X, Y, F> {
    fn color(mut self, color: Color) -> Self {
        self.maybe_color = Some(color);
        self
    }
}

impl<'a, X, Y, F> Frameable for XYPad<'a, X, Y, F> {
    fn frame(mut self, width: f64) -> Self {
        self.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, X, Y, F> Labelable<'a> for XYPad<'a, X, Y, F>
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

impl<'a, X, Y, F> position::Positionable for XYPad<'a, X, Y, F> {
    fn position(mut self, pos: Position) -> Self {
        self.pos = pos;
        self
    }
    #[inline]
    fn horizontal_align(self, h_align: HorizontalAlign) -> Self {
        XYPad { maybe_h_align: Some(h_align), ..self }
    }
    #[inline]
    fn vertical_align(self, v_align: VerticalAlign) -> Self {
        XYPad { maybe_v_align: Some(v_align), ..self }
    }
}

impl<'a, X, Y, F> position::Sizeable for XYPad<'a, X, Y, F> {
    #[inline]
    fn width(self, w: f64) -> Self {
        let h = self.dim[1];
        XYPad { dim: [w, h], ..self }
    }
    #[inline]
    fn height(self, h: f64) -> Self {
        let w = self.dim[0];
        XYPad { dim: [w, h], ..self }
    }
}

