
use {
    Backend,
    Circle,
    Color,
    Colorable,
    Direction,
    Edge,
    Frameable,
    FramedRectangle,
    FontSize,
    IndexSlot,
    Labelable,
    Mouse,
    NodeIndex,
    Point,
    PointPath,
    Positionable,
    Scalar,
    Sizeable,
    Text,
    Widget,
};
use num::Float;
use std::any::Any;
use std::cmp::Ordering;
use std::default::Default;
use std::fmt::Debug;
use utils::{clamp, map_range, percentage, val_to_string};
use widget;


/// Used for editing a series of 2D Points on a cartesian (X, Y) plane within some given range.
///
/// Useful for things such as oscillator/automation envelopes or any value series represented
/// periodically.
pub struct EnvelopeEditor<'a, E:'a, F> where E: EnvelopePoint {
    common: widget::CommonBuilder,
    env: &'a mut Vec<E>,
    /// The value skewing for the envelope's y-axis. This is useful for displaying exponential
    /// ranges such as frequency.
    pub skew_y_range: f32,
    min_x: E::X, max_x: E::X,
    min_y: E::Y, max_y: E::Y,
    maybe_react: Option<F>,
    maybe_label: Option<&'a str>,
    style: Style,
    enabled: bool,
}

/// Unique kind for the widget.
pub const KIND: widget::Kind = "EnvelopeEditor";

widget_style!{
    KIND;
    /// Styling for the EnvelopeEditor, necessary for constructing its renderable Element.
    style Style {
        /// Coloring for the EnvelopeEditor's **FramedRectangle**.
        - color: Color { theme.shape_color }
        /// Thickness of the **FramedRectangle**'s frame.
        - frame: f64 { theme.frame_width }
        /// Color of the frame.
        - frame_color: Color { theme.frame_color }
        /// Color of the label.
        - label_color: Color { theme.label_color }
        /// The font size of the **EnvelopeEditor**'s label if one was given.
        - label_font_size: FontSize { theme.font_size_medium }
        /// The font size of the value label.
        - value_font_size: FontSize { 14 }
        /// The radius of the envelope points.
        - point_radius: Scalar { 6.0 }
        /// The thickness of the envelope lines.
        - line_thickness: Scalar { 2.0 }
    }
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
    rectangle_idx: IndexSlot,
    label_idx: IndexSlot,
    value_label_idx: IndexSlot,
    point_path_idx: IndexSlot,
    point_indices: Vec<NodeIndex>,
}

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

    builder_methods!{
        pub point_radius { style.point_radius = Some(Scalar) }
        pub line_thickness { style.line_thickness = Some(Scalar) }
        pub value_font_size { style.value_font_size = Some(FontSize) }
        pub skew_y { skew_y_range = f32 }
        pub react { maybe_react = Some(F) }
        pub enabled { enabled = bool }
    }

}


impl<'a, E, F> Widget for EnvelopeEditor<'a, E, F>
    where E: EnvelopePoint,
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
            rectangle_idx: IndexSlot::new(),
            label_idx: IndexSlot::new(),
            value_label_idx: IndexSlot::new(),
            point_path_idx: IndexSlot::new(),
            point_indices: Vec::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the EnvelopeEditor's cached state.
    fn update<B: Backend>(self, args: widget::UpdateArgs<Self, B>) {
        use self::Interaction::{Clicked, Highlighted, Normal};

        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let EnvelopeEditor {
            env,
            skew_y_range,
            min_x, max_x,
            min_y, max_y,
            mut maybe_react,
            maybe_label,
            enabled,
            ..
        } = self;


        let maybe_mouse = ui.input(idx).maybe_mouse;
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
            (false, _) | (true, None) => Normal,
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
            (Highlighted(_), Clicked(_, _)) => { ui.capture_mouse(idx); },
            (Clicked(_, _), Highlighted(_)) |
            (Clicked(_, _), Normal)         => { ui.uncapture_mouse(idx); },
            _ => (),
        }

        // Draw the closest envelope point and it's label. Return the idx if it is currently clicked.
        let is_clicked_env_point = match new_interaction {
            Clicked(elem, _) | Highlighted(elem) => {
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

                // Call the `react` closure if mouse was released on one of the DropDownMenu items.
                match (state.view().interaction, new_interaction) {
                    (Clicked(_, m_button), Highlighted(_)) |
                    (Clicked(_, m_button), Normal) => {
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
                                if let Highlighted(ref mut elem) = new_interaction {
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
                    (Clicked(_, prev_m_button), Clicked(_, m_button)) => {
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
                    if let (Clicked(elem, m_button), Highlighted(_)) =
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
                    if let (Clicked(elem, m_button), Highlighted(_)) =
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
            Clicked(Elem::EnvPoint(idx, p), _)  |
            Highlighted(Elem::EnvPoint(idx, p)) => Some((idx, p)),
            Clicked(_, _) | Highlighted(_) =>
                maybe_mouse.and_then(|mouse| match closest_elem(&env, mouse.xy) {
                    Elem::EnvPoint(idx, p) => Some((idx, p)),
                    _ => None,
                }),
            _ => None,
        };

        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
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

        let rectangle_idx = state.view().rectangle_idx.get(&mut ui);
        let dim = rect.dim();
        let frame = style.frame(ui.theme());
        let color = new_interaction.color(style.color(ui.theme()));
        let frame_color = style.frame_color(ui.theme());
        FramedRectangle::new(dim)
            .middle_of(idx)
            .graphics_for(idx)
            .color(color)
            .frame(frame)
            .frame_color(frame_color)
            .set(rectangle_idx, &mut ui);

        let label_color = style.label_color(ui.theme());
        if let Some(label) = maybe_label {
            let label_idx = state.view().label_idx.get(&mut ui);
            let font_size = style.label_font_size(ui.theme());
            Text::new(label)
                .middle_of(rectangle_idx)
                .graphics_for(idx)
                .color(label_color)
                .font_size(font_size)
                .set(label_idx, &mut ui);
        }

        let line_color = label_color.with_alpha(1.0);
        {
            let point_path_idx = state.view().point_path_idx.get(&mut ui);
            let thickness = style.line_thickness(ui.theme());
            let points = env.iter().map(|point| {
                let x = map_x_to(point.get_x(), inner_rect.left(), inner_rect.right());
                let y = map_y_to(point.get_y(), inner_rect.bottom(), inner_rect.top());
                [x, y]
            });
            PointPath::new(points)
                .wh(inner_rect.dim())
                .xy(inner_rect.xy())
                .graphics_for(idx)
                .parent(idx)
                .color(line_color)
                .thickness(thickness)
                .set(point_path_idx, &mut ui);
        }

        let num_point_indices = state.view().point_indices.len();
        let len = env.len();
        if num_point_indices < len {
            let new_indices = (num_point_indices..len).map(|_| ui.new_unique_node_index());
            state.update(|state| state.point_indices.extend(new_indices));
        }

        let iter = state.view().point_indices.iter().zip(env.iter()).enumerate();
        for (i, (&point_idx, point)) in iter {
            let x = map_x_to(point.get_x(), inner_rect.left(), inner_rect.right());
            let y = map_y_to(point.get_y(), inner_rect.bottom(), inner_rect.top());
            let point_color = match new_interaction {
                Clicked(Elem::EnvPoint(idx, _), MouseButton::Left) if idx == i =>
                    line_color.clicked(),
                Highlighted(Elem::EnvPoint(idx, _)) if idx == i =>
                    line_color.highlighted(),
                _ => line_color,
            };
            Circle::fill(point_radius)
                .color(point_color)
                .x_y(x, y)
                .graphics_for(idx)
                .parent(idx)
                .set(point_idx, &mut ui);
        }

        if let Some((closest_idx, (x, y))) = maybe_closest_point {
            let x_range = max_x - min_x;
            let y_range = max_y - min_y;
            let x_px_range = inner_rect.w() as usize;
            let y_px_range = inner_rect.h() as usize;
            let x_string = val_to_string(env[closest_idx].get_x(), max_x, x_range, x_px_range);
            let y_string = val_to_string(env[closest_idx].get_y(), max_y, y_range, y_px_range);
            let xy_string = format!("{}, {}", x_string, y_string);
            let x_direction = match inner_rect.x.closest_edge(x) {
                Edge::End => Direction::Backwards,
                Edge::Start => Direction::Forwards,
            };
            let y_direction = match inner_rect.y.closest_edge(y) {
                Edge::End => Direction::Backwards,
                Edge::Start => Direction::Forwards,
            };
            let value_font_size = style.value_font_size(ui.theme());
            let value_label_idx = state.view().value_label_idx.get(&mut ui);
            let closest_point_idx = state.view().point_indices[closest_idx];
            const VALUE_TEXT_PAD: f64 = 5.0; // Slight padding between the point and the text.
            Text::new(&xy_string)
                .x_direction_from(closest_point_idx, x_direction, VALUE_TEXT_PAD)
                .y_direction_from(closest_point_idx, y_direction, VALUE_TEXT_PAD)
                .color(line_color)
                .graphics_for(idx)
                .parent(idx)
                .font_size(value_font_size)
                .set(value_label_idx, &mut ui);
        }

    }

}


impl<'a, E, F> Colorable for EnvelopeEditor<'a, E, F>
    where
        E: EnvelopePoint
{
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, E, F> Frameable for EnvelopeEditor<'a, E, F>
    where
        E: EnvelopePoint
{
    builder_methods!{
        frame { style.frame = Some(Scalar) }
        frame_color { style.frame_color = Some(Color) }
    }
}

impl<'a, E, F> Labelable<'a> for EnvelopeEditor<'a, E, F>
    where
        E: EnvelopePoint
{
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
