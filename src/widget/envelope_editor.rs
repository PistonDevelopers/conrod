
use {
    CharacterCache,
    Color,
    Colorable,
    Corner,
    Dimension,
    Dimensions,
    Frameable,
    FontSize,
    Labelable,
    Mouse,
    Point,
    Scalar,
    Theme,
    Ui,
    Widget,
};
use num::Float;
use std::any::Any;
use std::cmp::Ordering;
use std::default::Default;
use std::fmt::Debug;
use utils::{clamp, map_range, percentage, val_to_string};
use vecmath::vec2_sub;
use widget;


/// Used for editing a series of 2D Points on a cartesian (X, Y) plane within some given range.
/// Useful for things such as oscillator/automation envelopes or any value series represented
/// periodically.
pub struct EnvelopeEditor<'a, E:'a, F> where E: EnvelopePoint {
    common: widget::CommonBuilder,
    env: &'a mut Vec<E>,
    skew_y_range: f32,
    min_x: E::X, max_x: E::X,
    min_y: E::Y, max_y: E::Y,
    maybe_react: Option<F>,
    maybe_label: Option<&'a str>,
    style: Style,
    enabled: bool,
}

/// Styling for the EnvelopeEditor, necessary for constructing its renderable Element.
#[allow(missing_docs, missing_copy_implementations)]
#[derive(Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    pub maybe_color: Option<Color>,
    pub maybe_frame: Option<f64>,
    pub maybe_frame_color: Option<Color>,
    pub maybe_label_color: Option<Color>,
    pub maybe_label_font_size: Option<FontSize>,
    pub maybe_value_font_size: Option<FontSize>,
    pub maybe_point_radius: Option<f64>,
    pub maybe_line_width: Option<f64>,
}

/// Represents the state of the EnvelopeEditor widget.
#[derive(Clone, Debug, PartialEq)]
pub struct State<E> where E: EnvelopePoint {
    interaction: Interaction,
    env: Vec<E>,
    min_x: E::X,
    max_x: E::X,
    min_y: E::Y,
    max_y: E::Y,
    skew_y_range: f32,
    maybe_label: Option<String>,
    maybe_closest_point: Option<(usize, (f64, f64))>,
}

/// Unique kind for the widget.
pub const KIND: widget::Kind = "EnvelopeEditor";

/// Describes an interaction with the EnvelopeEditor.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Interaction {
    Normal,
    Highlighted(Elem),
    Clicked(Elem, MouseButton),
}

/// Represents the specific elements that the EnvelopeEditor is made up of. This is used to
/// specify which element is Highlighted or Clicked when storing State.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Elem {
    Rect,
    Pad,
    /// Represents an EnvelopePoint at `usize` index
    /// as well as the last mouse pos for comparison
    /// in determining new value.
    EnvPoint(usize, (f64, f64)),
    // /// Represents an EnvelopePoint's `curve` value.
    // CurvePoint(usize, (f64, f64)),
}

/// An enum to define which button is clicked.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
}


/// `EnvPoint` must be implemented for any type that is used as a 2D point within the
/// EnvelopeEditor.
pub trait EnvelopePoint: Any + Clone + Debug + PartialEq {
    /// A value on the X-axis of the envelope.
    type X: Any + Debug + Default + Float + ToString;
    /// A value on the Y-axis of the envelope.
    type Y: Any + Debug + Default + Float + ToString;
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


impl Interaction {
    /// Alter the widget color depending on the state.
    fn color(&self, color: Color) -> Color {
        match *self {
            Interaction::Normal => color,
            Interaction::Highlighted(_) => color.highlighted(),
            Interaction::Clicked(_, _) => color.clicked(),
        }
    }
}


/// Determine and return the new state from the previous state and the mouse position.
fn get_new_interaction(is_over_elem: Option<Elem>,
                       prev: Interaction,
                       mouse: Mouse) -> Interaction {
    use mouse::ButtonPosition::{Down, Up};
    use self::Elem::{EnvPoint};//, CurvePoint};
    use self::MouseButton::{Left, Right};
    use self::Interaction::{Normal, Highlighted, Clicked};
    match (is_over_elem, prev, mouse.left.position, mouse.right.position) {
        (Some(_), Normal, Down, Up) => Normal,
        (Some(elem), _, Up, Up) => Highlighted(elem),
        (Some(elem), Highlighted(_), Down, Up) => Clicked(elem, Left),
        (Some(_), Clicked(p_elem, m_button), Down, Up) |
        (Some(_), Clicked(p_elem, m_button), Up, Down) => {
            match p_elem {
                EnvPoint(idx, _) => Clicked(EnvPoint(idx, (mouse.xy[0], mouse.xy[1])), m_button),
                // CurvePoint(idx, _) =>
                //     Clicked(CurvePoint(idx, (mouse.xy[0], mouse.xy[1])), m_button),
                _ => Clicked(p_elem, m_button),
            }
        },
        (None, Clicked(p_elem, m_button), Down, Up) => {
            match (p_elem, m_button) {
                (EnvPoint(idx, _), Left) =>
                    Clicked(EnvPoint(idx, (mouse.xy[0], mouse.xy[1])), Left),
                // (CurvePoint(idx, _), Left) =>
                //     Clicked(CurvePoint(idx, (mouse.xy[0], mouse.xy[1])), Left),
                _ => Clicked(p_elem, Left),
            }
        },
        (Some(_), Highlighted(p_elem), Up, Down) => {
            match p_elem {
                EnvPoint(idx, _) => Clicked(EnvPoint(idx, (mouse.xy[0], mouse.xy[1])), Right),
                // CurvePoint(idx, _) => Clicked(CurvePoint(idx, (mouse.xy[0], mouse.xy[1])), Right),
                _ => Clicked(p_elem, Right),
            }
        },
        _ => Normal,
    }
}


impl<'a, E, F> EnvelopeEditor<'a, E, F> where E: EnvelopePoint {

    /// Set the radius of the envelope point circle.
    #[inline]
    pub fn point_radius(mut self, radius: f64) -> EnvelopeEditor<'a, E, F> {
        self.style.maybe_point_radius = Some(radius);
        self
    }

    /// Set the width of the envelope lines.
    #[inline]
    pub fn line_width(mut self, width: f64) -> EnvelopeEditor<'a, E, F> {
        self.style.maybe_line_width = Some(width);
        self
    }

    /// Set the font size for the displayed values.
    #[inline]
    pub fn value_font_size(mut self, size: FontSize) -> EnvelopeEditor<'a, E, F> {
        self.style.maybe_value_font_size = Some(size);
        self
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
            common: widget::CommonBuilder::new(),
            env: env,
            skew_y_range: 1.0, // Default skew amount (no skew).
            min_x: min_x, max_x: max_x,
            min_y: min_y, max_y: max_y,
            maybe_react: None,
            maybe_label: None,
            style: Style::new(),
            enabled: true,
        }
    }

    /// Set the reaction for the EnvelopeEditor. 
    pub fn react(mut self, reaction: F) -> EnvelopeEditor<'a, E, F> {
        self.maybe_react = Some(reaction);
        self
    }

    /// If true, will allow user inputs.  If false, will disallow user inputs.
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
    }

}


impl<'a, E, F> Widget for EnvelopeEditor<'a, E, F>
    where
        E: EnvelopePoint,
        E::X: Any,
        E::Y: Any,
        F: FnMut(&mut Vec<E>, usize),
{
    type State = State<E>;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> widget::Kind {
        KIND
    }

    fn init_state(&self) -> State<E> {
        State {
            interaction: Interaction::Normal,
            env: Vec::new(),
            min_x: self.min_x,
            max_x: self.max_x,
            min_y: self.min_y,
            max_y: self.max_y,
            skew_y_range: self.skew_y_range,
            maybe_label: None,
            maybe_closest_point: None,
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn default_x_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        widget::default_x_dimension(self, ui).unwrap_or(Dimension::Absolute(256.0))
    }

    fn default_y_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        widget::default_y_dimension(self, ui).unwrap_or(Dimension::Absolute(128.0))
    }

    /// Update the state of the EnvelopeEditor's cached state.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {

        let widget::UpdateArgs { state, rect, style, mut ui, .. } = args;
        let EnvelopeEditor {
            common,
            env,
            skew_y_range,
            min_x, max_x,
            min_y, max_y,
            mut maybe_react,
            maybe_label,
            style,
            enabled,
        } = self;


        let maybe_mouse = ui.input().maybe_mouse;
        let skew = skew_y_range;

        let point_radius = style.point_radius(ui.theme());
        let frame = style.frame(ui.theme());
        let inner_rect = rect.pad(frame);

        // Converts some envelope point's `x` value to a value in the given `Scalar` range.
        let map_x_to = |x: E::X, start: Scalar, end: Scalar| -> Scalar {
            map_range(x, min_x, max_x, start, end)
        };
        // Converts some envelope point's `y` value to a value in the given `Scalar` range.
        let map_y_to = |y: E::Y, start: Scalar, end: Scalar| -> Scalar {
            let skewed_perc = percentage(y, min_y, max_y).powf(1.0 / skew_y_range);
            map_range(skewed_perc, 0.0, 1.0, start, end)
        };

        // Converts some `Scalar` value in the given range to an `x` value for an envelope point.
        let map_to_x = |value: Scalar, start: Scalar, end: Scalar| -> E::X {
            map_range(value, start, end, min_x, max_x)
        };
        // Converts some `Scalar` value in the given range to an `y` value for an envelope point.
        let map_to_y = |value: Scalar, start: Scalar, end: Scalar| -> E::Y {
            let unskewed_perc = percentage(value, start, end).powf(skew_y_range);
            map_range(unskewed_perc, 0.0, 1.0, min_y, max_y)
        };

        // Determine the left and right X bounds for a point.
        let get_x_bounds = |env: &[E], idx: usize| -> (E::X, E::X) {
            let len = env.len();
            let right_bound = if len > 0 && len - 1 > idx { env[idx + 1].get_x() } else { max_x };
            let left_bound = if len > 0 && idx > 0 { env[idx - 1].get_x() } else { min_x };
            (left_bound, right_bound)
        };

        // Check for a new interaction.
        //
        // The reason we create the new interaction as mutable is because we may need to shift back
        // an index in the event that a point is removed.
        let mut new_interaction = match (enabled, maybe_mouse) {
            (false, _) | (true, None) => Interaction::Normal,
            (true, Some(mouse)) => {

                // Determine whether or not the cursor is over the EnvelopeEditor. If it is, return
                // the element under the cursor and the closest EnvPoint to the cursor.
                let is_over_elem = |env: &[E]| if rect.is_over(mouse.xy) {
                    if inner_rect.is_over(mouse.xy) {
                        for (i, p) in env.iter().enumerate() {
                            let px = p.get_x();
                            let py = p.get_y();
                            let x = map_x_to(px, inner_rect.left(), inner_rect.right());
                            let y = map_y_to(py, inner_rect.bottom(), inner_rect.top());
                            let distance = (mouse.xy[0] - x).powf(2.0)
                                         + (mouse.xy[1] - y).powf(2.0);
                            if distance <= point_radius.powf(2.0) {
                                return Some(Elem::EnvPoint(i, (x, y)));
                            }
                        }
                        Some(Elem::Pad)
                    } else {
                        Some(Elem::Rect)
                    }
                } else {
                    None
                };

                get_new_interaction(is_over_elem(&env), state.view().interaction, mouse)
            },
        };

        // Capture the mouse if clicked or uncapture the mouse if released.
        match (state.view().interaction, new_interaction) {
            (Interaction::Highlighted(_), Interaction::Clicked(_, _)) => { ui.capture_mouse(); },
            (Interaction::Clicked(_, _), Interaction::Highlighted(_)) |
            (Interaction::Clicked(_, _), Interaction::Normal)         => { ui.uncapture_mouse(); },
            _ => (),
        }

        // Draw the closest envelope point and it's label. Return the idx if it is currently clicked.
        let is_clicked_env_point = match new_interaction {
            Interaction::Clicked(elem, _) | Interaction::Highlighted(elem) => {
                if let Elem::EnvPoint(idx, _) = elem { Some(idx) } else { None }
            },
            _ => None,
        };

        // If some new mouse state was given...
        if let Some(mouse) = maybe_mouse {

            // Determine new values.
            let get_new_value = |env: &[E], idx: usize| -> (E::X, E::Y) {
                let mouse_x_clamped = inner_rect.x.clamp_value(mouse.xy[0]);
                let mouse_y_clamped = inner_rect.y.clamp_value(mouse.xy[1]);
                let unbounded_x = map_to_x(mouse_x_clamped, inner_rect.left(), inner_rect.right());
                let (left_bound, right_bound) = get_x_bounds(&env, idx);
                let new_x = clamp(unbounded_x, left_bound, right_bound);
                let new_y = map_to_y(mouse_y_clamped, inner_rect.bottom(), inner_rect.top());
                (new_x, new_y)
            };

            // If a point is currently clicked, check for react and value setting conditions.
            if let Some(idx) = is_clicked_env_point {

                // Call the `react` closure if mouse was released
                // on one of the DropDownMenu items.
                match (state.view().interaction, new_interaction) {
                    (Interaction::Clicked(_, m_button), Interaction::Highlighted(_)) |
                    (Interaction::Clicked(_, m_button), Interaction::Normal) => {
                        match m_button {
                            MouseButton::Left => {
                                // Adjust the point and trigger the reaction.
                                let (new_x, new_y) = get_new_value(&env, idx);
                                env[idx].set_x(new_x);
                                env[idx].set_y(new_y);
                                if let Some(ref mut react) = maybe_react {
                                    react(env, idx);
                                }
                            },
                            MouseButton::Right => {
                                // Delete the point and trigger the reaction.
                                env.remove(idx);
                                // Check for whether or not the highlighted index is out of range
                                // now that a point has been removed from the envelope.
                                if let Interaction::Highlighted(ref mut elem) = new_interaction {
                                    if env.is_empty() {
                                        *elem = Elem::Pad;
                                    } else if let Elem::EnvPoint(p_idx, p) = *elem {
                                        if p_idx >= env.len() {
                                            *elem = Elem::EnvPoint(env.len() - 1, p);
                                        }
                                    }
                                }
                                if let Some(ref mut react) = maybe_react {
                                    react(env, idx);
                                }
                            },
                        }
                    },
                    (Interaction::Clicked(_, prev_m_button), Interaction::Clicked(_, m_button)) => {
                        if let (MouseButton::Left, MouseButton::Left) = (prev_m_button, m_button) {
                            let (new_x, new_y) = get_new_value(&env, idx);
                            let current_x = env[idx].get_x();
                            let current_y = env[idx].get_y();
                            if new_x != current_x || new_y != current_y {
                                // Adjust the point and trigger the reaction.
                                env[idx].set_x(new_x);
                                env[idx].set_y(new_y);
                                if let Some(ref mut react) = maybe_react {
                                    react(env, idx);
                                }
                            }
                        }
                    },
                    _ => (),

                }

            } else {

                // Check if a there are no points. If so and the mouse was clicked, add a point.
                if env.len() == 0 {
                    if let (Interaction::Clicked(elem, m_button), Interaction::Highlighted(_)) =
                        (state.view().interaction, new_interaction) {
                        if let (Elem::Pad, MouseButton::Left) = (elem, m_button) {
                            let (new_x, new_y) = get_new_value(&env, 0);
                            let new_point = EnvelopePoint::new(new_x, new_y);
                            env.push(new_point);
                            if let Some(ref mut react) = maybe_react {
                                react(env, 0);
                            }
                        }
                    }
                }

                else {
                    // Check if a new point should be created.
                    if let (Interaction::Clicked(elem, m_button), Interaction::Highlighted(_)) =
                        (state.view().interaction, new_interaction) {
                        if let (Elem::Pad, MouseButton::Left) = (elem, m_button) {
                            let clamped_mouse_x = inner_rect.x.clamp_value(mouse.xy[0]);
                            let clamped_mouse_y = inner_rect.y.clamp_value(mouse.xy[1]);
                            let (left, right, bottom, top) = inner_rect.l_r_b_t();
                            let new_x = map_to_x(clamped_mouse_x, left, right);
                            let new_y = map_to_y(clamped_mouse_y, bottom, top);
                            let new_point = EnvelopePoint::new(new_x, new_y);
                            env.push(new_point);
                            env.sort_by(|a, b| if a.get_x() > b.get_x() { Ordering::Greater }
                                               else if a.get_x() < b.get_x() { Ordering::Less }
                                               else { Ordering::Equal });
                            if let Some(ref mut react) = maybe_react {
                                let idx = env.iter().enumerate().find(|&(_, point)| {
                                    point.get_x() == new_x && point.get_y() == new_y
                                }).map(|(idx, _)| idx).unwrap();
                                react(env, idx)
                            }
                        }
                    }
                }

            }

        }

        // A function for finding the closest element to the cursor.
        let closest_elem = |env: &[E], target: Point| {
            let mut closest_distance = ::std::f64::MAX;
            let mut closest_elem = Elem::Pad;
            for (i, p) in env.iter().enumerate() {
                let px = p.get_x();
                let py = p.get_y();
                let x = map_x_to(px, inner_rect.left(), inner_rect.right());
                let y = map_y_to(py, inner_rect.bottom(), inner_rect.top());
                let xy = [x, y];
                let distance = (target[0] - x).powf(2.0)
                             + (target[1] - y).powf(2.0);
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_elem = Elem::EnvPoint(i, (x, y));
                }
            }
            closest_elem
        };

        // Determine the closest point to the cursor.
        let maybe_closest_point = match new_interaction {
            Interaction::Clicked(Elem::EnvPoint(idx, p), _)  |
            Interaction::Highlighted(Elem::EnvPoint(idx, p)) => Some((idx, p)),
            Interaction::Clicked(_, _) | Interaction::Highlighted(_) =>
                maybe_mouse.and_then(|mouse| match closest_elem(&env, mouse.xy) {
                    Elem::EnvPoint(idx, p) => Some((idx, p)),
                    _ => None,
                }),
            _ => None,
        };

        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        if state.view().maybe_closest_point != maybe_closest_point {
            state.update(|state| state.maybe_closest_point = maybe_closest_point);
        }

        if &state.view().env[..] != &env[..] {
            state.update(|state| state.env = env.clone());
        }

        let bounds_have_changed = {
            let view = state.view();
            view.min_x != min_x || view.max_x != max_x || view.min_y != min_y || view.max_y != max_y
        };

        if bounds_have_changed {
            state.update(|state| {
                state.min_x = min_x;
                state.max_x = max_x;
                state.min_y = min_y;
                state.max_y = max_y;
            });
        }

        if state.view().skew_y_range != skew {
            state.update(|state| state.skew_y_range = skew);
        }

        if state.view().maybe_label.as_ref().map(|label| &label[..]) != maybe_label {
            state.update(|state| {
                state.maybe_label = maybe_label.as_ref().map(|label| label.to_string());
            })
        }
    }

    // /// Construct an Element from the given EnvelopeEditor State.
    // fn draw<C: CharacterCache>(args: widget::DrawArgs<Self, C>) -> Element {
    //     use elmesque::form::{self, circle, collage, Form, line, solid, text};
    //     use elmesque::text::Text;

    //     let widget::DrawArgs { rect, state, style, theme, glyph_cache, .. } = args;
    //     let (xy, dim) = rect.xy_dim();
    //     let frame = style.frame(theme);
    //     let pad_dim = vec2_sub(dim, [frame * 2.0; 2]);
    //     let (half_pad_w, half_pad_h) = (pad_dim[0] / 2.0, pad_dim[1] / 2.0);
    //     let skew = state.skew_y_range;
    //     let (min_x, max_x, min_y, max_y) = (state.min_x, state.max_x, state.min_y, state.max_y);

    //     // Construct the frame and inner rectangle Forms.
    //     let value_font_size = style.value_font_size(theme);
    //     let frame_color = style.frame_color(theme);
    //     let frame_form = form::rect(dim[0], dim[1]).filled(frame_color);
    //     let color = state.interaction.color(style.color(theme));
    //     let pressable_form = form::rect(pad_dim[0], pad_dim[1]).filled(color);

    //     // Construct the label Form.
    //     let maybe_label_form = state.maybe_label.as_ref().map(|l_text| {
    //         let l_color = style.label_color(theme);
    //         let l_size = style.label_font_size(theme);
    //         text(Text::from_string(l_text.clone()).color(l_color).height(l_size as f64))
    //     });

    //     // Create a vector with each EnvelopePoint value represented as a skewed weight
    //     // between 0.0 and 1.0.
    //     let perc_env: Vec<(f32, f32, f32)> = state.env.iter().map(|pt| {
    //         (percentage(pt.get_x(), min_x, max_x),
    //          percentage(pt.get_y(), min_y, max_y).powf(1.0 / skew),
    //          pt.get_curve())
    //     }).collect();

    //     // Draw the envelope lines.
    //     let line_color = color.plain_contrast();
    //     let line_width = style.line_width(theme);
    //     let envelope_line_forms = perc_env.windows(2).map(|window| {
    //         let ((x_a, y_a, _), (x_b, y_b, _)) = (window[0], window[1]);
    //         let p_a = [map_range(x_a, 0.0, 1.0, -half_pad_w, half_pad_w),
    //                    map_range(y_a, 0.0, 1.0, -half_pad_h, half_pad_h)];
    //         let p_b = [map_range(x_b, 0.0, 1.0, -half_pad_w, half_pad_w),
    //                    map_range(y_b, 0.0, 1.0, -half_pad_h, half_pad_h)];
    //         let style = solid(line_color).width(line_width);
    //         line(style, p_a[0], p_a[1], p_b[0], p_b[1])
    //     });

    //     // Draw the closest envelope point and it's label. Return the idx if it is currently clicked.
    //     let maybe_closest_point_form = match state.interaction {

    //         Interaction::Clicked(elem, _) | Interaction::Highlighted(elem) => {
    //             use std::iter::Chain;
    //             use std::option::IntoIter;

    //             // Construct a Form for an envelope point and it's value in text form.
    //             let env_pt_form = |env: &[E], idx: usize, p_pos: Point|
    //                                             -> Chain<IntoIter<Form>, IntoIter<Form>> {
    //                 let x_range = max_x - min_x;
    //                 let y_range = max_y - min_y;
    //                 let x_px_range = pad_dim[0] as usize;
    //                 let y_px_range = pad_dim[1] as usize;
    //                 let x_string = val_to_string(env[idx].get_x(), max_x, x_range, x_px_range);
    //                 let y_string = val_to_string(env[idx].get_y(), max_y, y_range, y_px_range);
    //                 let xy_string = format!("{}, {}", x_string, y_string);
    //                 const PAD: f64 = 5.0; // Slight padding between the crosshair and the text.
    //                 let w = glyph_cache.width(value_font_size, &xy_string);
    //                 let h = value_font_size as f64;
    //                 let x_shift = w / 2.0 + PAD;
    //                 let y_shift = h / 2.0 + PAD;
    //                 let (text_x, text_y) = match position::corner(p_pos, pad_dim) {
    //                     Corner::TopLeft => (x_shift, -y_shift),
    //                     Corner::TopRight => (-x_shift, -y_shift),
    //                     Corner::BottomLeft => (x_shift, y_shift),
    //                     Corner::BottomRight => (-x_shift, y_shift),
    //                 };
    //                 let color = color.plain_contrast();
    //                 let point_radius = style.point_radius(theme);
    //                 let circle_form = circle(point_radius).filled(color)
    //                     .shift(p_pos[0].floor(), p_pos[1].floor());
    //                 let text_form = text(Text::from_string(xy_string).color(color).height(h))
    //                     .shift(p_pos[0], p_pos[1])
    //                     .shift(text_x.floor(), text_y.floor());
    //                 Some(circle_form).into_iter().chain(Some(text_form))
    //             };

    //             match elem {
    //                 // If a point is clicked, draw that point.
    //                 Elem::EnvPoint(idx, (x, y)) => {
    //                     let (left_x_bound, right_x_bound) = get_x_bounds(&env, idx);
    //                     let left_pixel_bound = map_range(left_x_bound, 0.0, 1.0, -half_pad_w, half_pad_w);
    //                     let right_pixel_bound = map_range(right_x_bound, 0.0, 1.0, -half_pad_w, half_pad_w);
    //                     let p_pos_x_clamped = clamp(x, left_pixel_bound, right_pixel_bound);
    //                     let p_pos_y_clamped = clamp(y, -half_pad_h, half_pad_h);
    //                     let p_pos_clamped = [p_pos_x_clamped, p_pos_y_clamped];
    //                     let point_form = env_pt_form(&state.env[..], idx, p_pos_clamped);
    //                     Some(point_form)
    //                 },
    //                 // Otherwise, draw the closest point if there is one.
    //                 Elem::Pad => if let Some((closest_idx, (x, y))) = state.maybe_closest_point {
    //                     Some(env_pt_form(&state.env[..], closest_idx, [x, y]))
    //                 } else {
    //                     None
    //                 },
    //                 _ => None,
    //             }

    //         },
    //         _ => None,

    //     };

    //     // Group the different Forms into a single form.
    //     let form_chain = Some(frame_form).into_iter()
    //         .chain(Some(pressable_form))
    //         .chain(maybe_label_form)
    //         .chain(envelope_line_forms);
    //     let forms = match maybe_closest_point_form {
    //         Some(closest_point_form) => form_chain
    //             .chain(closest_point_form)
    //             .map(|form| form.shift(xy[0].floor(), xy[1].floor()))
    //             .collect(),
    //         None => form_chain
    //             .map(|form| form.shift(xy[0].floor(), xy[1].floor()))
    //             .collect(),
    //     };

    //     // Turn the form into a renderable element.
    //     collage(dim[0] as i32, dim[1] as i32, forms)

    // }

}


impl Style {

    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
            maybe_value_font_size: None,
            maybe_point_radius: None,
            maybe_line_width: None,
        }
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label Color for an Element.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn label_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_label_font_size.unwrap_or(theme.font_size_medium)
        })).unwrap_or(theme.font_size_medium)
    }

    /// Get the value font size for an Element.
    pub fn value_font_size(&self, theme: &Theme) -> FontSize {
        const DEFAULT_VALUE_FONT_SIZE: u32 = 14;
        self.maybe_value_font_size.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_value_font_size.unwrap_or(DEFAULT_VALUE_FONT_SIZE)
        })).unwrap_or(DEFAULT_VALUE_FONT_SIZE)
    }

    /// Get the point radius size for an Element.
    pub fn point_radius(&self, theme: &Theme) -> f64 {
        const DEFAULT_POINT_RADIUS: f64 = 6.0;
        self.maybe_point_radius.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_point_radius.unwrap_or(DEFAULT_POINT_RADIUS)
        })).unwrap_or(DEFAULT_POINT_RADIUS)
    }

    /// Get the point radius size for an Element.
    pub fn line_width(&self, theme: &Theme) -> f64 {
        const DEFAULT_LINE_WIDTH: f64 = 2.0;
        self.maybe_line_width.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_line_width.unwrap_or(DEFAULT_LINE_WIDTH)
        })).unwrap_or(DEFAULT_LINE_WIDTH)
    }

}


impl<'a, E, F> Colorable for EnvelopeEditor<'a, E, F>
    where
        E: EnvelopePoint
{
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

impl<'a, E, F> Frameable for EnvelopeEditor<'a, E, F>
    where
        E: EnvelopePoint
{
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
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
        self.style.maybe_label_color = Some(color);
        self
    }

    fn label_font_size(mut self, size: FontSize) -> Self {
        self.style.maybe_label_font_size = Some(size);
        self
    }
}

