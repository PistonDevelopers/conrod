//! The `EnvelopeEditor` widget and related items.

use {
    Color,
    Colorable,
    Direction,
    Edge,
    Borderable,
    FontSize,
    Labelable,
    NodeIndex,
    Point,
    Positionable,
    Rect,
    Scalar,
    Sizeable,
    Widget,
};
use num::Float;
use std::any::Any;
use std::default::Default;
use std::fmt::Debug;
use utils::{clamp, map_range, percentage, val_to_string};
use widget;


/// Used for editing a series of 2D Points on a cartesian (X, Y) plane within some given range.
///
/// Useful for things such as oscillator/automation envelopes or any value series represented
/// periodically.
pub struct EnvelopeEditor<'a, E:'a, F>
    where E: EnvelopePoint,
{
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

widget_style!{
    /// Styling for the EnvelopeEditor, necessary for constructing its renderable Element.
    style Style {
        /// Coloring for the EnvelopeEditor's **BorderedRectangle**.
        - color: Color { theme.shape_color }
        /// Thickness of the **BorderedRectangle**'s border.
        - border: f64 { theme.border_width }
        /// Color of the border.
        - border_color: Color { theme.border_color }
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
pub struct State {
    pressed_point: Option<usize>,
    rectangle_idx: widget::IndexSlot,
    label_idx: widget::IndexSlot,
    value_label_idx: widget::IndexSlot,
    point_path_idx: widget::IndexSlot,
    point_indices: Vec<NodeIndex>,
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
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self) -> Self::State {
        State {
            pressed_point: None,
            rectangle_idx: widget::IndexSlot::new(),
            label_idx: widget::IndexSlot::new(),
            value_label_idx: widget::IndexSlot::new(),
            point_path_idx: widget::IndexSlot::new(),
            point_indices: Vec::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the `EnvelopeEditor` in accordance to the latest input and call the given `react`
    /// function if necessary.
    fn update(self, args: widget::UpdateArgs<Self>) {
        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let EnvelopeEditor {
            env,
            skew_y_range,
            min_x, max_x,
            min_y, max_y,
            mut maybe_react,
            maybe_label,
            ..
        } = self;

        let point_radius = style.point_radius(ui.theme());
        let border = style.border(ui.theme());
        let rel_rect = Rect::from_xy_dim([0.0, 0.0], rect.dim());
        let inner_rel_rect = rel_rect.pad(border);

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

        // The index of the point that is under the given relative xy position.
        let point_under_rel_xy = |env: &[E], xy: Point| -> Option<usize> {
            for i in 0..env.len() {
                let px = env[i].get_x();
                let py = env[i].get_y();
                let x = map_x_to(px, inner_rel_rect.left(), inner_rel_rect.right());
                let y = map_y_to(py, inner_rel_rect.bottom(), inner_rel_rect.top());
                let distance = (xy[0] - x).powf(2.0)
                             + (xy[1] - y).powf(2.0);
                if distance <= point_radius.powf(2.0) {
                    return Some(i);
                }
            }
            None
        };

        // Track the currently pressed point if any.
        let mut pressed_point = state.pressed_point;

        // Handle all events that have occurred to the EnvelopeEditor since the last update.
        //
        // Check for:
        // - New points via left `Click`.
        // - Remove points via right `Click`.
        // - Dragging points via left `Drag`.
        'events: for widget_event in ui.widget_input(idx).events() {
            use event;
            use input::{self, MouseButton};

            match widget_event {

                // A left `Click` creates a new point, unless already over an existing point.
                event::Widget::Click(click) if click.button == input::MouseButton::Left => {
                    if !inner_rel_rect.is_over(click.xy) {
                        continue 'events;
                    }

                    // Find the points on either side of the click, while checking that the `Click`
                    // is not over a point.
                    let mut maybe_left = None;
                    let mut maybe_right = None;
                    for (i, p) in env.iter().enumerate() {
                        let px = p.get_x();
                        let py = p.get_y();
                        let x = map_x_to(px, inner_rel_rect.left(), inner_rel_rect.right());
                        let y = map_y_to(py, inner_rel_rect.bottom(), inner_rel_rect.top());
                        let distance = (click.xy[0] - x).powf(2.0)
                                     + (click.xy[1] - y).powf(2.0);

                        // If the click was over a point, we're done.
                        if distance <= point_radius.powf(2.0) {
                            continue 'events;
                        }

                        if x <= click.xy[0] {
                            maybe_left = Some(i);
                        } else if maybe_right.is_none() {
                            maybe_right = Some(i);
                        }
                    }

                    let new_x = map_to_x(click.xy[0], inner_rel_rect.left(), inner_rel_rect.right());
                    let new_y = map_to_y(click.xy[1], inner_rel_rect.bottom(), inner_rel_rect.top());
                    let new_point = EnvelopePoint::new(new_x, new_y);

                    let mut maybe_react = |env: &mut Vec<E>, idx: usize| {
                        if let Some(ref mut react) = maybe_react {
                            react(env, idx);
                        }
                    };

                    // Insert the point and call the reaction function if one was given.
                    match (maybe_left, maybe_right) {
                        (Some(_), None) | (None, None) => {
                            let idx = env.len();
                            env.push(new_point);
                            maybe_react(env, idx);
                        },
                        (None, Some(_)) => {
                            env.insert(0, new_point);
                            maybe_react(env, 0);
                        },
                        (Some(_), Some(idx)) => {
                            env.insert(idx, new_point);
                            maybe_react(env, idx);
                        },
                    }
                },

                // A right `Click` removes the point under the cursor.
                event::Widget::Click(click) if click.button == input::MouseButton::Right => {
                    if !inner_rel_rect.is_over(click.xy) {
                        continue 'events;
                    }

                    if let Some(idx) = point_under_rel_xy(env, click.xy) {
                        env.remove(idx);
                        if let Some(ref mut react) = maybe_react {
                            react(env, idx);
                        }
                    }
                },

                // Check to see if a point was pressed in case it is later dragged.
                event::Widget::Press(press) => {
                    if let event::Button::Mouse(MouseButton::Left, xy) = press.button {
                        // Check for a point under the cursor.
                        if let Some(idx) = point_under_rel_xy(env, xy) {
                            pressed_point = Some(idx);
                        } else if pressed_point.is_some() {
                            pressed_point = None;
                        }
                    }
                },

                // Check to see if a point was released in case it is later dragged.
                event::Widget::Release(release) => {
                    if let event::Button::Mouse(MouseButton::Left, _) = release.button {
                        pressed_point = None;
                    }
                },

                // A left `Drag` moves the `pressed_point` if there is one.
                event::Widget::Drag(drag) if drag.button == input::MouseButton::Left => {
                    if let Some(idx) = pressed_point {
                        let drag_to_x_clamped = inner_rel_rect.x.clamp_value(drag.to[0]);
                        let drag_to_y_clamped = inner_rel_rect.y.clamp_value(drag.to[1]);
                        let unbounded_x = map_to_x(drag_to_x_clamped,
                                                   inner_rel_rect.left(),
                                                   inner_rel_rect.right());
                        let (left_bound, right_bound) = get_x_bounds(env, idx);
                        let new_x = clamp(unbounded_x, left_bound, right_bound);
                        let new_y = map_to_y(drag_to_y_clamped,
                                             inner_rel_rect.bottom(),
                                             inner_rel_rect.top());
                        env[idx].set_x(new_x);
                        env[idx].set_y(new_y);
                        if let Some(ref mut react) = maybe_react {
                            react(env, idx);
                        }
                    }
                },

                _ => (),
            }
        }

        if state.pressed_point != pressed_point {
            state.update(|state| state.pressed_point = pressed_point);
        }

        let inner_rect = rect.pad(border);
        let rectangle_idx = state.rectangle_idx.get(&mut ui);
        let dim = rect.dim();
        let border = style.border(ui.theme());
        let color = style.color(ui.theme());
        let color = ui.widget_input(idx).mouse()
            .and_then(|m| if inner_rect.is_over(m.abs_xy()) { Some(color.highlighted()) }
                          else { None })
            .unwrap_or(color);
        let border_color = style.border_color(ui.theme());
        widget::BorderedRectangle::new(dim)
            .middle_of(idx)
            .graphics_for(idx)
            .color(color)
            .border(border)
            .border_color(border_color)
            .set(rectangle_idx, &mut ui);

        let label_color = style.label_color(ui.theme());
        if let Some(label) = maybe_label {
            let label_idx = state.label_idx.get(&mut ui);
            let font_size = style.label_font_size(ui.theme());
            widget::Text::new(label)
                .middle_of(rectangle_idx)
                .graphics_for(idx)
                .color(label_color)
                .font_size(font_size)
                .set(label_idx, &mut ui);
        }

        let line_color = label_color.with_alpha(1.0);
        {
            let point_path_idx = state.point_path_idx.get(&mut ui);
            let thickness = style.line_thickness(ui.theme());
            let points = env.iter().map(|point| {
                let x = map_x_to(point.get_x(), inner_rect.left(), inner_rect.right());
                let y = map_y_to(point.get_y(), inner_rect.bottom(), inner_rect.top());
                [x, y]
            });
            widget::PointPath::new(points)
                .wh(inner_rect.dim())
                .xy(inner_rect.xy())
                .graphics_for(idx)
                .parent(idx)
                .color(line_color)
                .thickness(thickness)
                .set(point_path_idx, &mut ui);
        }

        let num_point_indices = state.point_indices.len();
        let len = env.len();
        if num_point_indices < len {
            let new_indices = (num_point_indices..len).map(|_| ui.new_unique_node_index());
            state.update(|state| state.point_indices.extend(new_indices));
        }

        let iter = state.point_indices.iter().zip(env.iter()).enumerate();
        for (i, (&point_idx, point)) in iter {
            let x = map_x_to(point.get_x(), inner_rect.left(), inner_rect.right());
            let y = map_y_to(point.get_y(), inner_rect.bottom(), inner_rect.top());
            let point_color = if state.pressed_point == Some(i) {
                line_color.clicked()
            } else {
                ui.widget_input(idx).mouse()
                    .and_then(|mouse| {
                        let mouse_abs_xy = mouse.abs_xy();
                        let distance = (mouse_abs_xy[0] - x).powf(2.0)
                                     + (mouse_abs_xy[1] - y).powf(2.0);
                        if distance <= point_radius.powf(2.0) {
                            Some(line_color.highlighted())
                        } else {
                            None
                        }
                    })
                    .unwrap_or(line_color)
            };
            widget::Circle::fill(point_radius)
                .color(point_color)
                .x_y(x, y)
                .graphics_for(idx)
                .parent(idx)
                .set(point_idx, &mut ui);
        }

        // Find the closest point to the mouse.
        let maybe_closest_point = ui.widget_input(idx).mouse().and_then(|mouse| {
            let mut closest_distance = ::std::f64::MAX;
            let mut closest_point = None;
            for (i, p) in env.iter().enumerate() {
                let px = p.get_x();
                let py = p.get_y();
                let x = map_x_to(px, inner_rect.left(), inner_rect.right());
                let y = map_y_to(py, inner_rect.bottom(), inner_rect.top());
                let mouse_abs_xy = mouse.abs_xy();
                let distance = (mouse_abs_xy[0] - x).powf(2.0)
                             + (mouse_abs_xy[1] - y).powf(2.0);
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_point = Some((i, (x, y)));
                }
            }
            closest_point
        });

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
            let value_label_idx = state.value_label_idx.get(&mut ui);
            let closest_point_idx = state.point_indices[closest_idx];
            const VALUE_TEXT_PAD: f64 = 5.0; // Slight padding between the point and the text.
            widget::Text::new(&xy_string)
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

impl<'a, E, F> Borderable for EnvelopeEditor<'a, E, F>
    where
        E: EnvelopePoint
{
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
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
