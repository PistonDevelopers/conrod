use quack::{ GetFrom, SetAt, Get };
use std::cmp::Ordering;
use std::num::Float;
use std::num::ToPrimitive;
use std::num::FromPrimitive;
use color::Color;
use internal;
use graphics;
use graphics::{
    Context,
};
use label;
use internal::FontSize;
use mouse::Mouse;
use opengl_graphics::Gl;
use point::Point;
use rectangle;
use rectangle::{
    Corner
};
use ui_context::{
    UIID,
    UiContext,
};
use utils::{
    clamp,
    map_range,
    percentage,
    val_to_string,
};
use widget::Widget;
use vecmath::{
    vec2_add,
    vec2_sub
};
use Label;
use Text;
use Position;
use Dimensions;
use Frame;
use FrameColor;
use SkewY;
use PointRadius;
use LineWidth;

/// Represents the specific elements that the
/// EnvelopeEditor is made up of. This is used to
/// specify which element is Highlighted or Clicked
/// when storing State.
#[derive(Show, PartialEq, Clone, Copy)]
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
#[derive(Show, PartialEq, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
}

/// Represents the state of the xy_pad widget.
#[derive(Show, PartialEq, Clone, Copy)]
pub enum State {
    Normal,
    Highlighted(Element),
    Clicked(Element, MouseButton),
}

impl State {
    /// Return the associated Rectangle state.
    fn as_rectangle_state(&self) -> rectangle::State {
        match self {
            &State::Normal => rectangle::State::Normal,
            &State::Highlighted(_) => rectangle::State::Highlighted,
            &State::Clicked(_, _) => rectangle::State::Clicked,
        }
    }
}

widget_fns!(EnvelopeEditor, State, Widget::EnvelopeEditor(State::Normal));

/// `EnvPoint` MUST be implemented for any type that is
/// contained within the Envelope.
pub trait EnvelopePoint
<X: Float + Copy + ToPrimitive + FromPrimitive + PartialOrd + ToString,
 Y: Float + Copy + ToPrimitive + FromPrimitive + PartialOrd + ToString> {
    /// Return the X value.
    fn get_x(&self) -> X;
    /// Return the Y value.
    fn get_y(&self) -> Y;
    /// Set the X value.
    fn set_x(&mut self, _x: X);
    /// Set the Y value.
    fn set_y(&mut self, _y: Y);
    /// Return the bezier curve depth (-1. to 1.) for the next interpolation.
    fn get_curve(&self) -> f32 { 1.0 }
    /// Set the bezier curve depth (-1. to 1.) for the next interpolation.
    fn set_curve(&mut self, _curve: f32) {}
    /// Create a new EnvPoint.
    fn new(_x: X, _y: Y) -> Self;
}

/// Determine whether or not the cursor is over the EnvelopeEditor.
/// If it is, return the element under the cursor and the closest
/// EnvPoint to the cursor.
fn is_over_and_closest(pos: Point,
                       mouse_pos: Point,
                       dim: internal::Dimensions,
                       pad_pos: Point,
                       pad_dim: internal::Dimensions,
                       perc_env: &Vec<(f32, f32, f32)>,
                       pt_radius: f64) -> (Option<Element>, Option<Element>) {
    match rectangle::is_over(pos, mouse_pos, dim) {
        false => (None, None),
        true => match rectangle::is_over(pad_pos, mouse_pos, pad_dim) {
            false => (Some(Element::Rect), Some(Element::Rect)),
            true => {
                let mut closest_distance = ::std::f64::MAX_VALUE;
                let mut closest_env_point = Element::Pad;
                for (i, p) in perc_env.iter().enumerate() {
                    let (x, y, _) = *p;
                    let p_pos = [map_range(x, 0.0, 1.0, pad_pos[0], pad_pos[0] + pad_dim[0]),
                                 map_range(y, 0.0, 1.0, pad_pos[1] + pad_dim[1], pad_pos[1])];
                    let distance = (mouse_pos[0] - p_pos[0]).powf(2.0)
                                 + (mouse_pos[1] - p_pos[1]).powf(2.0);
                    //let distance = ::std::num::abs(mouse_pos.x - p_pos.x);
                    if distance <= pt_radius.powf(2.0) {
                        return (Some(Element::EnvPoint(i, (p_pos[0], p_pos[1]))),
                                Some(Element::EnvPoint(i, (p_pos[0], p_pos[1]))))
                    }
                    else if distance < closest_distance {
                        closest_distance = distance;
                        closest_env_point = Element::EnvPoint(i, (p_pos[0], p_pos[1]));
                    }
                }
                (Some(Element::Pad), Some(closest_env_point))
            },
        },
    }
}

/// Determine and return the new state from the previous
/// state and the mouse position.
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
                EnvPoint(idx, _) => Clicked(EnvPoint(idx, (mouse.pos[0], mouse.pos[1])), m_button),
                CurvePoint(idx, _) => Clicked(CurvePoint(idx, (mouse.pos[0], mouse.pos[1])), m_button),
                _ => Clicked(p_elem, m_button),
            }
        },
        (None, Clicked(p_elem, m_button), Down, Up) => {
            match (p_elem, m_button) {
                (EnvPoint(idx, _), Left) => Clicked(EnvPoint(idx, (mouse.pos[0], mouse.pos[1])), Left),
                (CurvePoint(idx, _), Left) => Clicked(CurvePoint(idx, (mouse.pos[0], mouse.pos[1])), Left),
                _ => Clicked(p_elem, Left),
            }
        },
        (Some(_), Highlighted(p_elem), Up, Down) => {
            match p_elem {
                EnvPoint(idx, _) => Clicked(EnvPoint(idx, (mouse.pos[0], mouse.pos[1])), Right),
                CurvePoint(idx, _) => Clicked(CurvePoint(idx, (mouse.pos[0], mouse.pos[1])), Right),
                _ => Clicked(p_elem, Right),
            }
        },
        _ => Normal,
    }
}

/// Draw a circle at the given position.
fn draw_circle(
    win_w: f64,
    win_h: f64,
    graphics: &mut Gl,
    pos: Point,
    color: Color,
    radius: f64
) {
    let context = &Context::abs(win_w, win_h);
    let Color(col) = color;
    graphics::Ellipse::new(col)
        .draw([pos[0], pos[1], 2.0 * radius, 2.0 * radius], context, graphics);
}

////////////////////////// NEW DESIGN //////////////////////////////////////////

pub struct EnvelopeEditor<'a, X, Y, E:'a> {
    env: &'a mut Vec<E>,
    skew_y_range: internal::Skew,
    min_x: X, max_x: X,
    min_y: Y, max_y: Y,
    pt_radius: internal::PointRadius,
    line_width: internal::LineWidth,
    font_size: FontSize,
    pos: Point,
    dim: internal::Dimensions,
    maybe_color: Option<internal::Color>,
    maybe_frame: Option<internal::Frame>,
    maybe_frame_color: Option<internal::Color>,
    maybe_label: Option<Label<'a>>,
}

impl<'a, X, Y, E: 'a> EnvelopeEditor<'a, X, Y, E> {
    /// Creates a new envelope editor.
    pub fn new(
        env: &'a mut Vec<E>,
        min_x: X,
        max_x: X,
        min_y: Y,
        max_y: Y
    ) -> Self {
        EnvelopeEditor {
            env: env,
            skew_y_range: 1.0, // Default skew amount (no skew).
            min_x: min_x, max_x: max_x,
            min_y: min_y, max_y: max_y,
            pt_radius: 6.0, // Default envelope point radius.
            line_width: 2.0, // Default envelope line width.
            font_size: 18u32,
            pos: [0.0, 0.0],
            dim: [256.0, 128.0],
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label: None,
        }
    }
}


impl<'a, X, Y, E: 'a> EnvelopeEditor<'a, X, Y, E>
    where
        X: Float + FromPrimitive + ToString,
        Y: Float + FromPrimitive + ToString,
        E: EnvelopePoint<X, Y>,
{
    pub fn draw(
        &mut self,
        ui_id: UIID,
        mut maybe_callback: Option<Box<FnMut(&mut Vec<E>, usize) + 'a>>,
        uic: &mut UiContext,
        graphics: &mut Gl
    ) {
        let state = *get_state(uic, ui_id);
        let mouse = uic.get_mouse_state();
        let skew = self.skew_y_range;
        let (min_x, max_x, min_y, max_y) = (self.min_x, self.max_x, self.min_y, self.max_y);
        let pt_radius = self.pt_radius;
        let font_size = self.font_size;

        // Rect.
        let color = self.maybe_color.unwrap_or(uic.theme.shape_color.0);
        let frame_w = self.maybe_frame.unwrap_or(uic.theme.frame_width);
        let frame_w2 = frame_w * 2.0;
        let maybe_frame = match frame_w > 0.0 {
            true => Some((frame_w, self.maybe_frame_color
                .map(|x| Color(x))
                .unwrap_or(uic.theme.frame_color))),
            false => None,
        };
        let pad_pos = vec2_add(self.pos, [frame_w; 2]);
        let pad_dim = vec2_sub(self.dim, [frame_w2; 2]);

        // Create a vector with each EnvelopePoint value represented as a
        // skewed percentage between 0.0 .. 1.0 .
        let perc_env: Vec<(f32, f32, f32)> = self.env.iter().map(|pt| {
            (percentage(pt.get_x(), min_x, max_x),
             percentage(pt.get_y(), min_y, max_y).powf(1.0 / skew),
             pt.get_curve())
        }).collect();

        // Check for new state.
        let (is_over_elem, is_closest_elem) = is_over_and_closest(
            self.pos, mouse.pos, self.dim,
            pad_pos, pad_dim, &perc_env, pt_radius
        );
        let new_state = get_new_state(is_over_elem, state, mouse);

        // Draw rect.
        rectangle::draw(uic.win_w, uic.win_h, graphics,
                        new_state.as_rectangle_state(),
                        self.pos, self.dim, maybe_frame, Color(color));

        // If there's a label, draw it.
        if let Some(label) = self.maybe_label {
            let Text(l_text) = label.get();
            let l_size = self.maybe_label
                .map(|x| x.font_size(&uic.theme))
                .unwrap_or(uic.theme.font_size_medium);
            let l_color = self.maybe_label
                .map(|x| x.color(&uic.theme))
                .unwrap_or(uic.theme.label_color.0);
            let l_w = label::width(uic, l_size, l_text);
            let l_pos = [pad_pos[0] + (pad_dim[0] - l_w) / 2.0,
                         pad_pos[1] + (pad_dim[1] - l_size as f64) / 2.0];
            uic.draw_text(graphics, l_pos, l_size, Color(l_color), l_text);
        };

        // Draw the envelope lines.
        match self.env.len() {
            0us | 1us => (),
            _ => {
                let Color(col) = Color(color).plain_contrast();
                let line = graphics::Line::round(col, 0.5 * self.line_width);
                for i in 1us..perc_env.len() {
                    let (x_a, y_a, _) = perc_env[i - 1us];
                    let (x_b, y_b, _) = perc_env[i];
                    let p_a = [map_range(x_a, 0.0, 1.0, pad_pos[0], pad_pos[0] + pad_dim[0]),
                               map_range(y_a, 0.0, 1.0, pad_pos[1] + pad_dim[1], pad_pos[1])];
                    let p_b = [map_range(x_b, 0.0, 1.0, pad_pos[0], pad_pos[0] + pad_dim[0]),
                               map_range(y_b, 0.0, 1.0, pad_pos[1] + pad_dim[1], pad_pos[1])];
                    let context = Context::abs(uic.win_w, uic.win_h);
                    line.draw([p_a[0], p_a[1], p_b[0], p_b[1]], &context, graphics);
                }
            },
        }

        // Determine the left and right X bounds for a point.
        let get_x_bounds = |&: envelope_perc: &Vec<(f32, f32, f32)>, idx: usize| -> (f32, f32) {
            let right_bound = if envelope_perc.len() > 0us && envelope_perc.len() - 1us > idx {
                (*envelope_perc)[idx + 1us].0
            } else { 1.0 };
            let left_bound = if envelope_perc.len() > 0us && idx > 0us {
                (*envelope_perc)[idx - 1us].0
            } else { 0.0 };
            (left_bound, right_bound)
        };

        // Draw the (closest) envelope point and it's label and
        // return the idx if it is currently clicked.
        let is_clicked_env_point = match (state, new_state) {

            (_, State::Clicked(elem, _)) | (_, State::Highlighted(elem)) => {

                // Draw the envelope point.
                let mut draw_env_pt = |&mut: uic: &mut UiContext, envelope: &mut Vec<E>, idx: usize, p_pos: Point| {
                    let x_string = val_to_string(
                        (*envelope)[idx].get_x(),
                        max_x, max_x - min_x, pad_dim[0] as usize
                    );
                    let y_string = val_to_string(
                        (*envelope)[idx].get_y(),
                        max_y, max_y - min_y, pad_dim[1] as usize
                    );
                    let xy_string = format!("{}, {}", x_string, y_string);
                    let xy_string_w = label::width(uic, font_size, xy_string.as_slice());
                    let xy_string_pos = match rectangle::corner(pad_pos, p_pos, pad_dim) {
                        Corner::TopLeft => [p_pos[0], p_pos[1]],
                        Corner::TopRight => [p_pos[0] - xy_string_w, p_pos[1]],
                        Corner::BottomLeft => [p_pos[0], p_pos[1] - font_size as f64],
                        Corner::BottomRight => [p_pos[0] - xy_string_w, p_pos[1] - font_size as f64],
                    };
                    uic.draw_text(graphics, xy_string_pos,
                                font_size, Color(color).plain_contrast(), xy_string.as_slice());
                    draw_circle(uic.win_w, uic.win_h, graphics,
                                vec2_sub(p_pos, [pt_radius, pt_radius]),
                                Color(color).plain_contrast(), pt_radius);
                };

                match elem {
                    // If a point is clicked, draw that point.
                    Element::EnvPoint(idx, p_pos) => {
                        let p_pos = [p_pos.0, p_pos.1];
                        let pad_x_right = pad_pos[0] + pad_dim[0];
                        let (left_x_bound, right_x_bound) = get_x_bounds(&perc_env, idx);
                        let left_pixel_bound = map_range(left_x_bound, 0.0, 1.0, pad_pos[0], pad_x_right);
                        let right_pixel_bound = map_range(right_x_bound, 0.0, 1.0, pad_pos[0], pad_x_right);
                        let p_pos_x_clamped = clamp(p_pos[0], left_pixel_bound, right_pixel_bound);
                        let p_pos_y_clamped = clamp(p_pos[1], pad_pos[1], pad_pos[1] + pad_dim[1]);
                        draw_env_pt(uic, self.env, idx, [p_pos_x_clamped, p_pos_y_clamped]);
                        Some(idx)
                    },
                    // Otherwise, draw the closest point.
                    Element::Pad => {
                        for closest_elem in is_closest_elem.iter() {
                            match *closest_elem {
                                Element::EnvPoint(closest_idx, closest_env_pt) => {
                                    let closest_env_pt = [closest_env_pt.0, closest_env_pt.1];
                                    draw_env_pt(uic, self.env, closest_idx, closest_env_pt);
                                },
                                _ => (),
                            }
                        }
                        None
                    }, _ => None,
                }

            }, _ => None,

        };

        // Determine new values.
        let get_new_value = |&: perc_envelope: &Vec<(f32, f32, f32)>,
                             idx: usize,
                             mouse_x: f64,
                             mouse_y: f64| -> (X, Y) {
            let mouse_x_on_pad = mouse_x - pad_pos[0];
            let mouse_y_on_pad = mouse_y - pad_pos[1];
            let mouse_x_clamped = clamp(mouse_x_on_pad, 0f64, pad_dim[0]);
            let mouse_y_clamped = clamp(mouse_y_on_pad, 0.0, pad_dim[1]);
            let new_x_perc = percentage(mouse_x_clamped, 0f64, pad_dim[0]);
            let new_y_perc = percentage(mouse_y_clamped, pad_dim[1], 0f64).powf(skew);
            let (left_bound, right_bound) = get_x_bounds(perc_envelope, idx);
            (map_range(if new_x_perc > right_bound { right_bound }
                       else if new_x_perc < left_bound { left_bound }
                       else { new_x_perc }, 0.0, 1.0, min_x, max_x),
             map_range(new_y_perc, 0.0, 1.0, min_y, max_y))
        };

        // If a point is currently clicked, check for callback
        // and value setting conditions.
        match is_clicked_env_point {

            Some(idx) => {

                // Call the `callback` closure if mouse was released
                // on one of the DropDownMenu items.
                match (state, new_state) {
                    (State::Clicked(_, m_button), State::Highlighted(_)) | (State::Clicked(_, m_button), State::Normal) => {
                        match m_button {
                            MouseButton::Left => {
                                // Adjust the point and trigger the callback.
                                let (new_x, new_y) = get_new_value(&perc_env, idx, mouse.pos[0], mouse.pos[1]);
                                self.env[idx].set_x(new_x);
                                self.env[idx].set_y(new_y);
                                match maybe_callback {
                                    Some(ref mut callback) => (*callback)(self.env, idx),
                                    None => (),
                                }
                            },
                            MouseButton::Right => {
                                // Delete the point and trigger the callback.
                                self.env.remove(idx);
                                match maybe_callback {
                                    Some(ref mut callback) => (*callback)(self.env, idx),
                                    None => (),
                                }
                            },
                        }
                    },

                    (State::Clicked(_, prev_m_button), State::Clicked(_, m_button)) => {
                        match (prev_m_button, m_button) {
                            (MouseButton::Left, MouseButton::Left) => {
                                let (new_x, new_y) = get_new_value(&perc_env, idx, mouse.pos[0], mouse.pos[1]);
                                let current_x = (*self.env)[idx].get_x();
                                let current_y = (*self.env)[idx].get_y();
                                if new_x != current_x || new_y != current_y {
                                    // Adjust the point and trigger the callback.
                                    self.env[idx].set_x(new_x);
                                    self.env[idx].set_y(new_y);
                                    match maybe_callback {
                                        Some(ref mut callback) => (*callback)(self.env, idx),
                                        None => (),
                                    }
                                }
                            }, _ => (),
                        }
                    }, _ => (),

                }

            },

            None => {

                // Check if a there are no points. If there are
                // and the mouse was clicked, add a point.
                if self.env.len() == 0us {
                    match (state, new_state) {
                        (State::Clicked(elem, m_button), State::Highlighted(_)) => {
                            match (elem, m_button) {
                                (Element::Pad, MouseButton::Left) => {
                                    let (new_x, new_y) = get_new_value(&perc_env, 0us, mouse.pos[0], mouse.pos[1]);
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
                                        let mouse_x_on_pad = mouse.pos[0] - pad_pos[0];
                                        let mouse_y_on_pad = mouse.pos[1] - pad_pos[1];
                                        let mouse_x_clamped = clamp(mouse_x_on_pad, 0f64, pad_dim[0]);
                                        let mouse_y_clamped = clamp(mouse_y_on_pad, 0.0, pad_dim[1]);
                                        let new_x_perc = percentage(mouse_x_clamped, 0f64, pad_dim[0]);
                                        let new_y_perc = percentage(mouse_y_clamped, pad_dim[1], 0f64).powf(skew);
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

        // Set the new state.
        set_state(uic, ui_id, new_state, self.pos, self.dim);

    }
}


impl<'a, X, Y, E: 'a> GetFrom for (Position, EnvelopeEditor<'a, X, Y, E>) {
    type Property = Position;
    type Object = EnvelopeEditor<'a, X, Y, E>;

    fn get_from(envelope_editor: &EnvelopeEditor<'a, X, Y, E>) -> Position {
        Position(envelope_editor.pos)
    }
}

impl<'a, X, Y, E: 'a> SetAt for (Position, EnvelopeEditor<'a, X, Y, E>) {
    type Property = Position;
    type Object = EnvelopeEditor<'a, X, Y, E>;

    fn set_at(
        Position(pos): Position,
        envelope_editor: &mut EnvelopeEditor<'a, X, Y, E>
    ) {
        envelope_editor.pos = pos;
    }
}

impl<'a, X, Y, E: 'a> GetFrom for (Dimensions, EnvelopeEditor<'a, X, Y, E>) {
    type Property = Dimensions;
    type Object = EnvelopeEditor<'a, X, Y, E>;

    fn get_from(envelope_editor: &EnvelopeEditor<'a, X, Y, E>) -> Dimensions {
        Dimensions(envelope_editor.dim)
    }
}

impl<'a, X, Y, E: 'a> SetAt for (Dimensions, EnvelopeEditor<'a, X, Y, E>) {
    type Property = Dimensions;
    type Object = EnvelopeEditor<'a, X, Y, E>;

    fn set_at(
        Dimensions(dim): Dimensions,
        envelope_editor: &mut EnvelopeEditor<'a, X, Y, E>
    ) {
        envelope_editor.dim = dim;
    }
}

impl<'a, X, Y, E: 'a> SetAt for (Color, EnvelopeEditor<'a, X, Y, E>) {
    type Property = Color;
    type Object = EnvelopeEditor<'a, X, Y, E>;

    fn set_at(
        Color(color): Color,
        envelope_editor: &mut EnvelopeEditor<'a, X, Y, E>
    ) {
        envelope_editor.maybe_color = Some(color);
    }
}

impl<'a, X, Y, E: 'a> SetAt for (Frame, EnvelopeEditor<'a, X, Y, E>) {
    type Property = Frame;
    type Object = EnvelopeEditor<'a, X, Y, E>;

    fn set_at(
        Frame(frame): Frame,
        envelope_editor: &mut EnvelopeEditor<'a, X, Y, E>
    ) {
        envelope_editor.maybe_frame = Some(frame);
    }
}

impl<'a, X, Y, E: 'a> SetAt for (FrameColor, EnvelopeEditor<'a, X, Y, E>) {
    type Property = FrameColor;
    type Object = EnvelopeEditor<'a, X, Y, E>;

    fn set_at(
        FrameColor(color): FrameColor,
        envelope_editor: &mut EnvelopeEditor<'a, X, Y, E>
    ) {
        envelope_editor.maybe_frame_color = Some(color);
    }
}

impl<'a, X, Y, E: 'a> SetAt for (Label<'a>, EnvelopeEditor<'a, X, Y, E>) {
    type Property = Label<'a>;
    type Object = EnvelopeEditor<'a, X, Y, E>;

    fn set_at(
        label: Label<'a>,
        envelope_editor: &mut EnvelopeEditor<'a, X, Y, E>
    ) {
        envelope_editor.maybe_label = Some(label);
    }
}

impl<'a, X, Y, E: 'a> GetFrom for (SkewY, EnvelopeEditor<'a, X, Y, E>) {
    type Property = SkewY;
    type Object = EnvelopeEditor<'a, X, Y, E>;

    fn get_from(envelope_editor: &EnvelopeEditor<'a, X, Y, E>) -> SkewY {
        SkewY(envelope_editor.skew_y_range)
    }
}

impl<'a, X, Y, E: 'a> SetAt for (SkewY, EnvelopeEditor<'a, X, Y, E>) {
    type Property = SkewY;
    type Object = EnvelopeEditor<'a, X, Y, E>;

    fn set_at(
        SkewY(skew_y): SkewY,
        envelope_editor: &mut EnvelopeEditor<'a, X, Y, E>
    ) {
        envelope_editor.skew_y_range = skew_y;
    }
}

impl<'a, X, Y, E: 'a> GetFrom for (PointRadius, EnvelopeEditor<'a, X, Y, E>) {
    type Property = PointRadius;
    type Object = EnvelopeEditor<'a, X, Y, E>;

    fn get_from(envelope_editor: &EnvelopeEditor<'a, X, Y, E>) -> PointRadius {
        PointRadius(envelope_editor.pt_radius)
    }
}

impl<'a, X, Y, E: 'a> SetAt for (PointRadius, EnvelopeEditor<'a, X, Y, E>) {
    type Property = PointRadius;
    type Object = EnvelopeEditor<'a, X, Y, E>;

    fn set_at(
        PointRadius(pt_radius): PointRadius,
        envelope_editor: &mut EnvelopeEditor<'a, X, Y, E>
    ) {
        envelope_editor.pt_radius = pt_radius;
    }
}

impl<'a, X, Y, E: 'a> GetFrom for (LineWidth, EnvelopeEditor<'a, X, Y, E>) {
    type Property = LineWidth;
    type Object = EnvelopeEditor<'a, X, Y, E>;

    fn get_from(envelope_editor: &EnvelopeEditor<'a, X, Y, E>) -> LineWidth {
        LineWidth(envelope_editor.line_width)
    }
}

impl<'a, X, Y, E: 'a> SetAt for (LineWidth, EnvelopeEditor<'a, X, Y, E>) {
    type Property = LineWidth;
    type Object = EnvelopeEditor<'a, X, Y, E>;

    fn set_at(
        LineWidth(line_width): LineWidth,
        envelope_editor: &mut EnvelopeEditor<'a, X, Y, E>
    ) {
        envelope_editor.line_width = line_width;
    }
}
