//! The `EnvelopeEditor` widget and related items.

use {Color, Colorable, Borderable, FontSize, Labelable, Positionable, Sizeable, Widget};
use num::Float;
use position::{Direction, Edge, Point, Rect, Scalar};
use std;
use text;
use utils::{clamp, map_range, percentage, val_to_string};
use widget;


/// Used for editing a series of 2D Points on a cartesian (X, Y) plane within some given range.
///
/// Useful for things such as oscillator/automation envelopes or any value series represented
/// periodically.
#[derive(WidgetCommon_)]
pub struct EnvelopeEditor<'a, E>
    where E: EnvelopePoint + 'a,
{
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    env: &'a [E],
    /// The value skewing for the envelope's y-axis. This is useful for displaying exponential
    /// ranges such as frequency.
    pub skew_y_range: f32,
    min_x: E::X, max_x: E::X,
    min_y: E::Y, max_y: E::Y,
    maybe_label: Option<&'a str>,
    style: Style,
    enabled: bool,
}

/// Styling for the EnvelopeEditor, necessary for constructing its renderable Element.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// Coloring for the EnvelopeEditor's **BorderedRectangle**.
    #[conrod(default = "theme.shape_color")]
    pub color: Option<Color>,
    /// Thickness of the **BorderedRectangle**'s border.
    #[conrod(default = "theme.border_width")]
    pub border: Option<f64>,
    /// Color of the border.
    #[conrod(default = "theme.border_color")]
    pub border_color: Option<Color>,
    /// Color of the label.
    #[conrod(default = "theme.label_color")]
    pub label_color: Option<Color>,
    /// The font size of the **EnvelopeEditor**'s label if one was given.
    #[conrod(default = "theme.font_size_medium")]
    pub label_font_size: Option<FontSize>,
    /// The font size of the value label.
    #[conrod(default = "14")]
    pub value_font_size: Option<FontSize>,
    /// The radius of the envelope points.
    #[conrod(default = "6.0")]
    pub point_radius: Option<Scalar>,
    /// The thickness of the envelope lines.
    #[conrod(default = "2.0")]
    pub line_thickness: Option<Scalar>,
    /// The ID of the font used to display the label.
    #[conrod(default = "theme.font_id")]
    pub label_font_id: Option<Option<text::font::Id>>,
}

widget_ids! {
    struct Ids {
        rectangle,
        label,
        value_label,
        point_path,
        points[],
    }
}

/// Represents the state of the EnvelopeEditor widget.
pub struct State {
    pressed_point: Option<usize>,
    ids: Ids,
}


/// `EnvPoint` must be implemented for any type that is used as a 2D point within the
/// EnvelopeEditor.
pub trait EnvelopePoint: Clone + PartialEq {
    /// A value on the X-axis of the envelope.
    type X: Float + ToString;
    /// A value on the Y-axis of the envelope.
    type Y: Float + ToString;
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


impl<'a, E> EnvelopeEditor<'a, E>
    where E: EnvelopePoint,
{

    /// Construct an EnvelopeEditor widget.
    pub fn new(env: &'a [E], min_x: E::X, max_x: E::X, min_y: E::Y, max_y: E::Y) -> Self {
        EnvelopeEditor {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            env: env,
            skew_y_range: 1.0, // Default skew amount (no skew).
            min_x: min_x, max_x: max_x,
            min_y: min_y, max_y: max_y,
            maybe_label: None,
            enabled: true,
        }
    }

    /// Specify the font used for displaying the label.
    pub fn label_font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.label_font_id = Some(Some(font_id));
        self
    }

    builder_methods!{
        pub point_radius { style.point_radius = Some(Scalar) }
        pub line_thickness { style.line_thickness = Some(Scalar) }
        pub value_font_size { style.value_font_size = Some(FontSize) }
        pub skew_y { skew_y_range = f32 }
        pub enabled { enabled = bool }
    }

}


/// The kinds of events that may be yielded by the `EnvelopeEditor`.
#[derive(Copy, Clone, Debug)]
pub enum Event<E>
    where E: EnvelopePoint,
{
    /// Insert a new point.
    AddPoint {
        /// The index at which the point should be inserted.
        i: usize,
        /// The new point.
        point: E,
    },
    /// Remove a point.
    RemovePoint {
        /// The index of the point that should be removed.
        i: usize,
    },
    /// Move a point.
    MovePoint {
        /// The index of the point that should be moved.
        i: usize,
        /// The point's new *x* value.
        x: E::X,
        /// The point's new *y* value.
        y: E::Y,
    },
}


impl<E> Event<E>
    where E: EnvelopePoint,
{

    /// Update the given `envelope` in accordance with the `Event`.
    pub fn update(self, envelope: &mut Vec<E>) {
        match self {

            Event::AddPoint { i, point } => {
                if i <= envelope.len() {
                    envelope.insert(i, point);
                }
            },

            Event::RemovePoint { i } => {
                if i < envelope.len() {
                    envelope.remove(i);
                }
            },

            Event::MovePoint { i, x, y } => {
                let maybe_left = if i == 0 { None } else { envelope.get(i - 1).map(|p| p.get_x()) };
                let maybe_right = envelope.get(i + 1).map(|p| p.get_x());
                if let Some(p) = envelope.get_mut(i) {
                    let mut set_clamped = |min_x, max_x| {
                        let x = if x < min_x { min_x } else if x > max_x { max_x } else { x };
                        p.set_x(x);
                        p.set_y(y);
                    };
                    match (maybe_left, maybe_right) {
                        (None, None) => set_clamped(x, x),
                        (Some(min), None) => set_clamped(min, x),
                        (None, Some(max)) => set_clamped(x, max),
                        (Some(min), Some(max)) => set_clamped(min, max),
                    }
                }
            },

        }
    }

}


impl<'a, E> Widget for EnvelopeEditor<'a, E>
    where E: EnvelopePoint,
{
    type State = State;
    type Style = Style;
    type Event = Vec<Event<E>>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            pressed_point: None,
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the `EnvelopeEditor` in accordance to the latest input and call the given `react`
    /// function if necessary.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, rect, style, mut ui, .. } = args;
        let EnvelopeEditor {
            env,
            skew_y_range,
            min_x, max_x,
            min_y, max_y,
            maybe_label,
            ..
        } = self;

        let mut env = std::borrow::Cow::Borrowed(env);

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
        let mut events = Vec::new();
        'events: for widget_event in ui.widget_input(id).events() {
            use event;
            use input::{self, MouseButton};

            match widget_event {

                // Upon mouse press, check for:
                //
                // - Point insertion
                // - Point removal
                // - The beggining of a point drag
                event::Widget::Press(press) => match press.button {

                    // Left mouse press:
                    //
                    // If the mouse is currently over an existing point, we want to begin
                    // dragging it.
                    //
                    // Otherwise, if the mouse is not over an existing point, we want to insert
                    // a new point and begin dragging it.
                    event::Button::Mouse(MouseButton::Left, xy) => {

                        // In this loop, we find the points on either side of the mouse to
                        // determine the insertion index, while checking if we need to break early
                        // in the case that the mouse is already over a point.
                        let mut maybe_left = None;
                        let mut maybe_right = None;
                        for (i, p) in env.iter().enumerate() {
                            let px = p.get_x();
                            let py = p.get_y();
                            let x = map_x_to(px, inner_rel_rect.left(), inner_rel_rect.right());
                            let y = map_y_to(py, inner_rel_rect.bottom(), inner_rel_rect.top());
                            let distance = (xy[0] - x).powf(2.0)
                                         + (xy[1] - y).powf(2.0);

                            // If the press was over a point, begin dragging it and we're done.
                            if distance <= point_radius.powf(2.0) {
                                pressed_point = Some(i);
                                continue 'events;
                            }

                            if x <= xy[0] {
                                maybe_left = Some(i);
                            } else if maybe_right.is_none() {
                                maybe_right = Some(i);
                                break;
                            }
                        }

                        // We only want to insert a point if the mouse is in the inner rectangle
                        // and not on the border.
                        if !inner_rel_rect.is_over(xy) {
                            continue 'events;
                        }

                        let new_x = map_to_x(xy[0], inner_rel_rect.left(), inner_rel_rect.right());
                        let new_y = map_to_y(xy[1], inner_rel_rect.bottom(), inner_rel_rect.top());
                        let new_point = EnvelopePoint::new(new_x, new_y);

                        // Insert the point and push an `AddPoint` event.
                        match (maybe_left, maybe_right) {
                            (Some(_), None) | (None, None) => {
                                let idx = env.len();
                                let event = Event::AddPoint { i: idx, point: new_point };
                                pressed_point = Some(idx);
                                events.push(event);
                            },
                            (None, Some(_)) => {
                                let idx = 0;
                                let event = Event::AddPoint { i: idx, point: new_point };
                                pressed_point = Some(idx);
                                events.push(event);
                            },
                            (Some(_), Some(idx)) => {
                                let event = Event::AddPoint { i: idx, point: new_point };
                                pressed_point = Some(idx);
                                events.push(event);
                            },
                        }
                    },

                    // If the right mouse button was pressed over a point that is not currently
                    // being dragged, remove the point.
                    event::Button::Mouse(MouseButton::Right, xy) => {
                        if pressed_point.is_some() || !inner_rel_rect.is_over(xy) {
                            continue 'events;
                        }

                        if let Some(idx) = point_under_rel_xy(&env, xy) {
                            let event = Event::RemovePoint { i: idx };
                            events.push(event);
                        }
                    },

                    _ => (),
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
                        let (left_bound, right_bound) = get_x_bounds(&env, idx);
                        let new_x = clamp(unbounded_x, left_bound, right_bound);
                        let new_y = map_to_y(drag_to_y_clamped,
                                             inner_rel_rect.bottom(),
                                             inner_rel_rect.top());
                        let event = Event::MovePoint { i: idx, x: new_x, y: new_y };
                        events.push(event);
                    }
                },

                _ => (),
            }
        }

        if state.pressed_point != pressed_point {
            state.update(|state| state.pressed_point = pressed_point);
        }

        // Ensure that the local version of the `env` is up to date for drawing.
        for event in &events {
            event.clone().update(env.to_mut());
        }

        let inner_rect = rect.pad(border);
        let dim = rect.dim();
        let border = style.border(ui.theme());
        let color = style.color(ui.theme());
        let color = ui.widget_input(id).mouse()
            .and_then(|m| if inner_rect.is_over(m.abs_xy()) { Some(color.highlighted()) }
                          else { None })
            .unwrap_or(color);
        let border_color = style.border_color(ui.theme());
        widget::BorderedRectangle::new(dim)
            .middle_of(id)
            .graphics_for(id)
            .color(color)
            .border(border)
            .border_color(border_color)
            .set(state.ids.rectangle, ui);

        let font_id = style.label_font_id(&ui.theme).or(ui.fonts.ids().next());
        let label_color = style.label_color(&ui.theme);
        if let Some(label) = maybe_label {
            let font_size = style.label_font_size(&ui.theme);
            widget::Text::new(label)
                .and_then(font_id, widget::Text::font_id)
                .middle_of(state.ids.rectangle)
                .graphics_for(id)
                .color(label_color)
                .font_size(font_size)
                .set(state.ids.label, ui);
        }

        let line_color = label_color.with_alpha(1.0);
        {
            let thickness = style.line_thickness(ui.theme());
            let points = env.iter().map(|point| {
                let x = map_x_to(point.get_x(), inner_rect.left(), inner_rect.right());
                let y = map_y_to(point.get_y(), inner_rect.bottom(), inner_rect.top());
                [x, y]
            });
            widget::PointPath::new(points)
                .wh(inner_rect.dim())
                .xy(inner_rect.xy())
                .graphics_for(id)
                .parent(id)
                .color(line_color)
                .thickness(thickness)
                .set(state.ids.point_path, ui);
        }

        // Ensure we have at least as many point widgets as there are points in the env.
        if state.ids.points.len() < env.len() {
            state.update(|state| state.ids.points.resize(env.len(), &mut ui.widget_id_generator()));
        }

        let iter = state.ids.points.iter().zip(env.iter()).enumerate();
        for (i, (&point_id, point)) in iter {
            let x = map_x_to(point.get_x(), inner_rect.left(), inner_rect.right());
            let y = map_y_to(point.get_y(), inner_rect.bottom(), inner_rect.top());
            let point_color = if state.pressed_point == Some(i) {
                line_color.clicked()
            } else {
                ui.widget_input(id).mouse()
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
                .graphics_for(id)
                .parent(id)
                .set(point_id, &mut ui);
        }

        // Find the closest point to the mouse.
        let maybe_closest_point = ui.widget_input(id).mouse().and_then(|mouse| {
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
            let closest_point_id = state.ids.points[closest_idx];
            const VALUE_TEXT_PAD: f64 = 5.0; // Slight padding between the point and the text.
            widget::Text::new(&xy_string)
                .and_then(font_id, widget::Text::font_id)
                .x_direction_from(closest_point_id, x_direction, VALUE_TEXT_PAD)
                .y_direction_from(closest_point_id, y_direction, VALUE_TEXT_PAD)
                .color(line_color)
                .graphics_for(id)
                .parent(id)
                .font_size(value_font_size)
                .set(state.ids.value_label, ui);
        }

        events
    }

}


impl<'a, E> Colorable for EnvelopeEditor<'a, E>
    where E: EnvelopePoint
{
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, E> Borderable for EnvelopeEditor<'a, E>
    where E: EnvelopePoint
{
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a, E> Labelable<'a> for EnvelopeEditor<'a, E>
    where E: EnvelopePoint
{
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
