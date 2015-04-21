
use color::{Color, Colorable};
use frame::Frameable;
use graphics::math::Scalar;
use graphics::character::CharacterCache;
use label::{self, FontSize, Labelable};
use mouse::Mouse;
use num::{Float, NumCast};
use position::{self, Corner, Depth, Dimensions, HorizontalAlign, Point, Position, VerticalAlign};
use std::cmp::Ordering;
use ui::{UiId, Ui};
use utils::{clamp, map_range, percentage, val_to_string};
use vecmath::vec2_sub;
use widget::Kind;

/// Represents the specific elements that the EnvelopeEditor is made up of. This is used to
/// specify which element is Highlighted or Clicked when storing State.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Element {
    Rect,
    Pad,
    /// Represents an EnvelopePoint at `usize` index
    /// as well as the last mouse pos for comparison
    /// in determining new value.
    EnvPoint(usize, (f64, f64)),
    /// Represents an EnvelopePoint's `curve` value.
    CurvePoint(usize, (f64, f64)),
}

/// An enum to define which button is clicked.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
}

/// Represents the state of the xy_pad widget.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum State {
    Normal,
    Highlighted(Element),
    Clicked(Element, MouseButton),
}

impl State {
    /// Alter the widget color depending on the state.
    fn color(&self, color: Color) -> Color {
        match *self {
            State::Normal => color,
            State::Highlighted(_) => color.highlighted(),
            State::Clicked(_, _) => color.clicked(),
        }
    }
}

widget_fns!(EnvelopeEditor, State, Kind::EnvelopeEditor(State::Normal));

/// `EnvPoint` must be implemented for any type that is used as a 2D point within the
/// EnvelopeEditor.
pub trait EnvelopePoint {
    /// A value on the X-axis of the envelope.
    type X: Float + NumCast + ToString;
    /// A value on the Y-axis of the envelope.
    type Y: Float + NumCast + ToString;
    /// Return the X value.
    fn get_x(&self) -> Self::X;
    /// Return the Y value.
    fn get_y(&self) -> Self::Y;
    /// Set the X value.
    fn set_x(&mut self, _x: Self::X);
    /// Set the Y value.
    fn set_y(&mut self, _y: Self::Y);
    /// Return the bezier curve depth (-1. to 1.) for the next interpolation.
    fn get_curve(&self) -> f32 { 1.0 }
    /// Set the bezier curve depth (-1. to 1.) for the next interpolation.
    fn set_curve(&mut self, _curve: f32) {}
    /// Create a new EnvPoint.
    fn new(_x: Self::X, _y: Self::Y) -> Self;
}

impl EnvelopePoint for Point {
    type X = Scalar;
    type Y = Scalar;
    /// Return the X value.
    fn get_x(&self) -> Scalar { self[0] }
    /// Return the Y value.
    fn get_y(&self) -> Scalar { self[1] }
    /// Return the X value.
    fn set_x(&mut self, x: Scalar) { self[0] = x }
    /// Return the Y value.
    fn set_y(&mut self, y: Scalar) { self[1] = y }
    /// Create a new Envelope Point.
    fn new(x: Scalar, y: Scalar) -> Point { [x, y] }
}

/// Determine whether or not the cursor is over the EnvelopeEditor. If it is, return the element
/// under the cursor and the closest EnvPoint to the cursor.
fn is_over_and_closest(mouse_xy: Point,
                       dim: Dimensions,
                       pad_dim: Dimensions,
                       perc_env: &[(f32, f32, f32)],
                       pt_radius: Scalar) -> (Option<Element>, Option<Element>) {
    use utils::is_over_rect;
    if is_over_rect([0.0, 0.0], mouse_xy, dim) {
        if is_over_rect([0.0, 0.0], mouse_xy, pad_dim) {
            let mut closest_distance = ::std::f64::MAX;
            let mut closest_env_point = Element::Pad;
            for (i, p) in perc_env.iter().enumerate() {
                let (x, y, _) = *p;
                let half_pad_w = pad_dim[0] / 2.0;
                let half_pad_h = pad_dim[1] / 2.0;
                let p_xy = [map_range(x, 0.0, 1.0, -half_pad_w, half_pad_w),
                            map_range(y, 0.0, 1.0, -half_pad_h, half_pad_h)];
                let distance = (mouse_xy[0] - p_xy[0]).powf(2.0)
                             + (mouse_xy[1] - p_xy[1]).powf(2.0);
                if distance <= pt_radius.powf(2.0) {
                    return (Some(Element::EnvPoint(i, (p_xy[0], p_xy[1]))),
                            Some(Element::EnvPoint(i, (p_xy[0], p_xy[1]))))
                }
                else if distance < closest_distance {
                    closest_distance = distance;
                    closest_env_point = Element::EnvPoint(i, (p_xy[0], p_xy[1]));
                }
            }
            (Some(Element::Pad), Some(closest_env_point))
        } else {
            (Some(Element::Rect), Some(Element::Rect))
        }
    } else {
        (None, None)
    }
}

/// Determine and return the new state from the previous state and the mouse position.
fn get_new_state(is_over_elem: Option<Element>,
                 prev: State,
                 mouse: Mouse) -> State {
    use mouse::ButtonState::{Down, Up};
    use self::Element::{EnvPoint, CurvePoint};
    use self::MouseButton::{Left, Right};
    use self::State::{Normal, Highlighted, Clicked};
    match (is_over_elem, prev, mouse.left, mouse.right) {
        (Some(_), Normal, Down, Up) => Normal,
        (Some(elem), _, Up, Up) => Highlighted(elem),
        (Some(elem), Highlighted(_), Down, Up) => Clicked(elem, Left),
        (Some(_), Clicked(p_elem, m_button), Down, Up) |
        (Some(_), Clicked(p_elem, m_button), Up, Down) => {
            match p_elem {
                EnvPoint(idx, _) => Clicked(EnvPoint(idx, (mouse.xy[0], mouse.xy[1])), m_button),
                CurvePoint(idx, _) =>
                    Clicked(CurvePoint(idx, (mouse.xy[0], mouse.xy[1])), m_button),
                _ => Clicked(p_elem, m_button),
            }
        },
        (None, Clicked(p_elem, m_button), Down, Up) => {
            match (p_elem, m_button) {
                (EnvPoint(idx, _), Left) =>
                    Clicked(EnvPoint(idx, (mouse.xy[0], mouse.xy[1])), Left),
                (CurvePoint(idx, _), Left) =>
                    Clicked(CurvePoint(idx, (mouse.xy[0], mouse.xy[1])), Left),
                _ => Clicked(p_elem, Left),
            }
        },
        (Some(_), Highlighted(p_elem), Up, Down) => {
            match p_elem {
                EnvPoint(idx, _) => Clicked(EnvPoint(idx, (mouse.xy[0], mouse.xy[1])), Right),
                CurvePoint(idx, _) => Clicked(CurvePoint(idx, (mouse.xy[0], mouse.xy[1])), Right),
                _ => Clicked(p_elem, Right),
            }
        },
        _ => Normal,
    }
}

/// Used for editing a series of 2D Points on a cartesian (X, Y) plane within some given range.
/// Useful for things such as oscillator/automation envelopes or any value series represented
/// periodically.
pub struct EnvelopeEditor<'a, E:'a, F> where E: EnvelopePoint {
    env: &'a mut Vec<E>,
    skew_y_range: f32,
    min_x: E::X, max_x: E::X,
    min_y: E::Y, max_y: E::Y,
    pt_radius: f64,
    line_width: f64,
    value_font_size: FontSize,
    pos: Position,
    dim: Dimensions,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    depth: Depth,
    maybe_callback: Option<F>,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_label: Option<&'a str>,
    maybe_label_color: Option<Color>,
    maybe_label_font_size: Option<u32>,
}

impl<'a, E, F> EnvelopeEditor<'a, E, F> where E: EnvelopePoint {

    /// Set the radius of the envelope point circle.
    #[inline]
    pub fn point_radius(self, radius: f64) -> EnvelopeEditor<'a, E, F> {
        EnvelopeEditor { pt_radius: radius, ..self }
    }

    /// Set the width of the envelope lines.
    #[inline]
    pub fn line_width(self, width: f64) -> EnvelopeEditor<'a, E, F> {
        EnvelopeEditor { line_width: width, ..self }
    }

    /// Set the font size for the displayed values.
    #[inline]
    pub fn value_font_size(self, size: FontSize) -> EnvelopeEditor<'a, E, F> {
        EnvelopeEditor { value_font_size: size, ..self }
    }

    /// Set the value skewing for the envelope's y-axis. This is useful for displaying exponential
    /// ranges such as frequency.
    #[inline]
    pub fn skew_y(self, skew: f32) -> EnvelopeEditor<'a, E, F> {
        EnvelopeEditor { skew_y_range: skew, ..self }
    }

    /// Construct an EnvelopeEditor widget.
    pub fn new(env: &'a mut Vec<E>, min_x: E::X, max_x: E::X, min_y: E::Y, max_y: E::Y)
    -> EnvelopeEditor<'a, E, F> {
        EnvelopeEditor {
            env: env,
            skew_y_range: 1.0, // Default skew amount (no skew).
            min_x: min_x, max_x: max_x,
            min_y: min_y, max_y: max_y,
            pt_radius: 6.0, // Default envelope point radius.
            line_width: 2.0, // Default envelope line width.
            value_font_size: 14u32,
            pos: Position::default(),
            dim: [256.0, 128.0],
            maybe_h_align: None,
            maybe_v_align: None,
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

    /// Set the callback for the EnvelopeEditor. 
    pub fn callback(mut self, cb: F) -> EnvelopeEditor<'a, E, F> {
        self.maybe_callback = Some(cb);
        self
    }

    /// After building the EnvelopeEditor, use this method to set its current state into the given
    /// `Ui`. It will use this state for rendering the next time `ui.draw(graphics)` is called.
    pub fn set<C>(mut self, ui_id: UiId, ui: &mut Ui<C>)
        where
            C: CharacterCache,
            E: EnvelopePoint,
            E::X: Float,
            E::Y: Float,
            F: FnMut(&mut Vec<E>, usize),
    {
        use elmesque::form::{circle, collage, Form, line, rect, solid, text};
        use elmesque::text::Text;

        let state = *get_state(ui, ui_id);
        let dim = self.dim;
        let h_align = self.maybe_h_align.unwrap_or(ui.theme.h_align);
        let v_align = self.maybe_v_align.unwrap_or(ui.theme.v_align);
        let xy = ui.get_xy(self.pos, dim, h_align, v_align);
        let mouse = ui.get_mouse_state(ui_id).relative_to(xy);
        let skew = self.skew_y_range;
        let (min_x, max_x, min_y, max_y) = (self.min_x, self.max_x, self.min_y, self.max_y);
        let pt_radius = self.pt_radius;
        let value_font_size = self.value_font_size;
        let frame_w = self.maybe_frame.unwrap_or(ui.theme.frame_width);
        let frame_w2 = frame_w * 2.0;
        let pad_dim = vec2_sub(dim, [frame_w2; 2]);
        let half_pad_w = pad_dim[0] / 2.0;
        let half_pad_h = pad_dim[1] / 2.0;
        let line_width = self.line_width;

        // Create a vector with each EnvelopePoint value represented as a skewed weight
        // between 0.0 and 1.0.
        let perc_env: Vec<(f32, f32, f32)> = self.env.iter().map(|pt| {
            (percentage(pt.get_x(), min_x, max_x),
             percentage(pt.get_y(), min_y, max_y).powf(1.0 / skew),
             pt.get_curve())
        }).collect();

        // Check for new state.
        let (is_over_elem, is_closest_elem) =
            is_over_and_closest(mouse.xy, dim, pad_dim, &perc_env[..], pt_radius);
        let new_state = get_new_state(is_over_elem, state, mouse);

        // Construct the frame and inner rectangle Forms.
        let frame_color = self.maybe_frame_color.unwrap_or(ui.theme.frame_color);
        let frame_form = rect(dim[0], dim[1]).filled(frame_color);
        let color = new_state.color(self.maybe_color.unwrap_or(ui.theme.shape_color));
        let pressable_form = rect(pad_dim[0], pad_dim[1]).filled(color);

        // Construct the label Form.
        let maybe_label_form = self.maybe_label.map(|l_text| {
            let l_color = self.maybe_label_color.unwrap_or(ui.theme.label_color);
            let l_size = self.maybe_label_font_size.unwrap_or(ui.theme.font_size_medium);
            text(Text::from_string(l_text.to_string()).color(l_color).height(l_size as f64))
        });

        // Draw the envelope lines.
        let line_color = color.plain_contrast();
        let envelope_line_forms = perc_env.windows(2).map(|window| {
            let ((x_a, y_a, _), (x_b, y_b, _)) = (window[0], window[1]);
            let p_a = [map_range(x_a, 0.0, 1.0, -half_pad_w, half_pad_w),
                       map_range(y_a, 0.0, 1.0, -half_pad_h, half_pad_h)];
            let p_b = [map_range(x_b, 0.0, 1.0, -half_pad_w, half_pad_w),
                       map_range(y_b, 0.0, 1.0, -half_pad_h, half_pad_h)];
            let style = solid(line_color).width(line_width);
            line(style, p_a[0], p_a[1], p_b[0], p_b[1])
        });

        // Determine the left and right X bounds for a point.
        let get_x_bounds = |envelope_perc: &[(f32, f32, f32)], idx: usize| -> (f32, f32) {
            let right_bound = if envelope_perc.len() > 0 && envelope_perc.len() - 1 > idx {
                envelope_perc[idx + 1].0 // X value of point on right.
            } else { 1.0 };
            let left_bound = if envelope_perc.len() > 0 && idx > 0 {
                envelope_perc[idx - 1].0 // X value of point on left.
            } else { 0.0 };
            (left_bound, right_bound)
        };

        // Draw the closest envelope point and it's label. Return the idx if it is currently clicked.
        let (maybe_closest_point_form, is_clicked_env_point) = match (state, new_state) {

            (_, State::Clicked(elem, _)) | (_, State::Highlighted(elem)) => {
                use std::iter::Chain;
                use std::option::IntoIter;

                // Construct a Form for an envelope point and it's value in text form.
                let env_pt_form = |ui: &mut Ui<C>, env: &[E], idx: usize, p_pos: Point|
                                                -> Chain<IntoIter<Form>, IntoIter<Form>> {
                    let x_range = max_x - min_x;
                    let y_range = max_y - min_y;
                    let x_px_range = pad_dim[0] as usize;
                    let y_px_range = pad_dim[1] as usize;
                    let x_string = val_to_string(env[idx].get_x(), max_x, x_range, x_px_range);
                    let y_string = val_to_string(env[idx].get_y(), max_y, y_range, y_px_range);
                    let xy_string = format!("{}, {}", x_string, y_string);
                    const PAD: f64 = 5.0; // Slight padding between the crosshair and the text.
                    let w = label::width(ui, value_font_size, &xy_string);
                    let h = value_font_size as f64;
                    let x_shift = w / 2.0 + PAD;
                    let y_shift = h / 2.0 + PAD;
                    let (text_x, text_y) = match position::corner(p_pos, pad_dim) {
                        Corner::TopLeft => (x_shift, -y_shift),
                        Corner::TopRight => (-x_shift, -y_shift),
                        Corner::BottomLeft => (x_shift, y_shift),
                        Corner::BottomRight => (-x_shift, y_shift),
                    };
                    let color = color.plain_contrast();
                    let circle_form = circle(pt_radius).filled(color)
                        .shift(p_pos[0].floor(), p_pos[1].floor());
                    let text_form = text(Text::from_string(xy_string).color(color).height(h))
                        .shift(p_pos[0], p_pos[1])
                        .shift(text_x.floor(), text_y.floor());
                    Some(circle_form).into_iter().chain(Some(text_form).into_iter())
                };

                match elem {
                    // If a point is clicked, draw that point.
                    Element::EnvPoint(idx, (x, y)) => {
                        let (left_x_bound, right_x_bound) = get_x_bounds(&perc_env[..], idx);
                        let left_pixel_bound = map_range(left_x_bound, 0.0, 1.0, -half_pad_w, half_pad_w);
                        let right_pixel_bound = map_range(right_x_bound, 0.0, 1.0, -half_pad_w, half_pad_w);
                        let p_pos_x_clamped = clamp(x, left_pixel_bound, right_pixel_bound);
                        let p_pos_y_clamped = clamp(y, -half_pad_h, half_pad_h);
                        let p_pos_clamped = [p_pos_x_clamped, p_pos_y_clamped];
                        let point_form = env_pt_form(ui, &self.env[..], idx, p_pos_clamped);
                        (Some(point_form), Some(idx))
                    },
                    // Otherwise, draw the closest point if there is one.
                    Element::Pad => match is_closest_elem {
                        Some(Element::EnvPoint(closest_idx, (x, y))) => {
                            let point_form = env_pt_form(ui, &self.env[..], closest_idx, [x, y]);
                            (Some(point_form), None)
                        },
                        _ => (None, None),
                    },
                    _ => (None, None),
                }

            },
            _ => (None, None),

        };

        // Determine new values.
        let get_new_value = |perc_envelope: &[(f32, f32, f32)], idx: usize| -> (E::X, E::Y) {
            let mouse_x_clamped = clamp(mouse.xy[0], -half_pad_w, half_pad_w);
            let mouse_y_clamped = clamp(mouse.xy[1], -half_pad_h, half_pad_h);
            let new_x_perc = percentage(mouse_x_clamped, -half_pad_w, half_pad_w);
            let new_y_perc = percentage(mouse_y_clamped, -half_pad_h, half_pad_h).powf(skew);
            let (left_bound, right_bound) = get_x_bounds(perc_envelope, idx);
            (map_range(if new_x_perc > right_bound { right_bound }
                       else if new_x_perc < left_bound { left_bound }
                       else { new_x_perc }, 0.0, 1.0, min_x, max_x),
             map_range(new_y_perc, 0.0, 1.0, min_y, max_y))
        };

        // If a point is currently clicked, check for callback and value setting conditions.
        match is_clicked_env_point {

            Some(idx) => {

                // Call the `callback` closure if mouse was released
                // on one of the DropDownMenu items.
                match (state, new_state) {
                    (State::Clicked(_, m_button), State::Highlighted(_)) |
                    (State::Clicked(_, m_button), State::Normal) => {
                        match m_button {
                            MouseButton::Left => {
                                // Adjust the point and trigger the callback.
                                let (new_x, new_y) = get_new_value(&perc_env[..], idx);
                                self.env[idx].set_x(new_x);
                                self.env[idx].set_y(new_y);
                                match self.maybe_callback {
                                    Some(ref mut callback) => callback(self.env, idx),
                                    None => (),
                                }
                            },
                            MouseButton::Right => {
                                // Delete the point and trigger the callback.
                                self.env.remove(idx);
                                match self.maybe_callback {
                                    Some(ref mut callback) => callback(self.env, idx),
                                    None => (),
                                }
                            },
                        }
                    },

                    (State::Clicked(_, prev_m_button), State::Clicked(_, m_button)) => {
                        match (prev_m_button, m_button) {
                            (MouseButton::Left, MouseButton::Left) => {
                                let (new_x, new_y) = get_new_value(&perc_env[..], idx);
                                let current_x = self.env[idx].get_x();
                                let current_y = self.env[idx].get_y();
                                if new_x != current_x || new_y != current_y {
                                    // Adjust the point and trigger the callback.
                                    self.env[idx].set_x(new_x);
                                    self.env[idx].set_y(new_y);
                                    match self.maybe_callback {
                                        Some(ref mut callback) => callback(self.env, idx),
                                        None => (),
                                    }
                                }
                            }, _ => (),
                        }
                    }, _ => (),

                }

            },

            None => {

                // Check if a there are no points. If so and the mouse was clicked, add a point.
                if self.env.len() == 0 {
                    match (state, new_state) {
                        (State::Clicked(elem, m_button), State::Highlighted(_)) => {
                            match (elem, m_button) {
                                (Element::Pad, MouseButton::Left) => {
                                    let (new_x, new_y) = get_new_value(&perc_env[..], 0);
                                    let new_point = EnvelopePoint::new(new_x, new_y);
                                    self.env.push(new_point);
                                }, _ => (),
                            }
                        }, _ => (),
                    }
                }

                else {
                    // Check if a new point should be created.
                    match (state, new_state) {
                        (State::Clicked(elem, m_button), State::Highlighted(_)) => {
                            match (elem, m_button) {
                                (Element::Pad, MouseButton::Left) => {
                                    let (new_x, new_y) = {
                                        let mouse_x = clamp(mouse.xy[0], -half_pad_w, half_pad_w);
                                        let mouse_y = clamp(mouse.xy[1], -half_pad_h, half_pad_h);
                                        let new_x_perc = percentage(mouse_x, -half_pad_w, half_pad_w);
                                        let new_y_perc = percentage(mouse_y, -half_pad_h, half_pad_h).powf(skew);
                                        (map_range(new_x_perc, 0.0, 1.0, min_x, max_x),
                                         map_range(new_y_perc, 0.0, 1.0, min_y, max_y))
                                    };
                                    let new_point = EnvelopePoint::new(new_x, new_y);
                                    self.env.push(new_point);
                                    self.env.sort_by(|a, b| if a.get_x() > b.get_x() { Ordering::Greater }
                                                            else if a.get_x() < b.get_x() { Ordering::Less }
                                                            else { Ordering::Equal });
                                }, _ => (),
                            }
                        }, _ => (),
                    }
                }

            },

        }

        // Group the different Forms into a single form.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(pressable_form).into_iter())
            .chain(maybe_label_form.into_iter())
            .chain(envelope_line_forms);
        let forms = match maybe_closest_point_form {
            Some(closest_point_form) => form_chain
                .chain(closest_point_form)
                .map(|form| form.shift(xy[0].floor(), xy[1].floor()))
                .collect(),
            None => form_chain
                .map(|form| form.shift(xy[0].floor(), xy[1].floor()))
                .collect(),
        };

        // Turn the form into a renderable element.
        let element = collage(dim[0] as i32, dim[1] as i32, forms);

        // Store the EnvelopeEditor's new state in the Ui.
        ui.set_widget(ui_id, ::widget::Widget {
            kind: Kind::EnvelopeEditor(new_state),
            xy: xy,
            depth: self.depth,
            element: Some(element),
        });

    }

}

impl<'a, E, F> Colorable for EnvelopeEditor<'a, E, F>
    where
        E: EnvelopePoint
{
    fn color(mut self, color: Color) -> Self {
        self.maybe_color = Some(color);
        self
    }
}

impl<'a, E, F> Frameable for EnvelopeEditor<'a, E, F>
    where
        E: EnvelopePoint
{
    fn frame(mut self, width: f64) -> Self {
        self.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, E, F> Labelable<'a> for EnvelopeEditor<'a, E, F>
    where
        E: EnvelopePoint
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

impl<'a, E, F> position::Positionable for EnvelopeEditor<'a, E, F>
    where
        E: EnvelopePoint
{
    fn position(mut self, pos: Position) -> Self {
        self.pos = pos;
        self
    }
    #[inline]
    fn horizontal_align(self, h_align: HorizontalAlign) -> Self {
        EnvelopeEditor { maybe_h_align: Some(h_align), ..self }
    }
    #[inline]
    fn vertical_align(self, v_align: VerticalAlign) -> Self {
        EnvelopeEditor { maybe_v_align: Some(v_align), ..self }
    }
}

impl<'a, E, F> position::Sizeable for EnvelopeEditor<'a, E, F>
    where
        E: EnvelopePoint
{
    #[inline]
    fn width(self, w: f64) -> Self {
        let h = self.dim[1];
        EnvelopeEditor { dim: [w, h], ..self }
    }
    #[inline]
    fn height(self, h: f64) -> Self {
        let w = self.dim[0];
        EnvelopeEditor { dim: [w, h], ..self }
    }
}

