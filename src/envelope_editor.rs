
use color::Color;
use graphics::{
    AddColor,
    AddEllipse,
    AddLine,
    AddRoundBorder,
    Context,
    Draw,
};
use label;
use label::FontSize;
use mouse_state::{
    MouseState,
    Up,
    Down,
};
use opengl_graphics::Gl;
use point::Point;
use rectangle;
use rectangle::{
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
};
use ui_context::{
    UIID,
    UIContext,
};
use utils::{
    clamp,
    map_range,
    percentage,
    val_to_string,
};
use widget::EnvelopeEditor;

/// Represents the specific elements that the
/// EnvelopeEditor is made up of. This is used to
/// specify which element is Highlighted or Clicked
/// when storing State.
#[deriving(Show, PartialEq, Clone)]
pub enum Element {
    Rect,
    Pad,
    /// Represents an EnvelopePoint at `uint` index
    /// as well as the last mouse pos for comparison
    /// in determining new value.
    EnvPoint(uint, Point<f64>),
    /// Represents an EnvelopePoint's `curve` value.
    CurvePoint(uint, Point<f64>),
}

/// An enum to define which button is clicked.
#[deriving(Show, PartialEq, Clone)]
pub enum MouseButton {
    Left,
    Right,
}

/// Represents the state of the xy_pad widget.
#[deriving(Show, PartialEq, Clone)]
pub enum State {
    Normal,
    Highlighted(Element),
    Clicked(Element, MouseButton),
}

impl State {
    /// Return the associated Rectangle state.
    fn as_rectangle_state(&self) -> rectangle::State {
        match self {
            &Normal => rectangle::Normal,
            &Highlighted(_) => rectangle::Highlighted,
            &Clicked(_, _) => rectangle::Clicked,
        }
    }
}

widget_fns!(EnvelopeEditor, State, EnvelopeEditor(Normal))

/// `EnvPoint` MUST be implemented for any type that is
/// contained within the Envelope.
pub trait EnvelopePoint
<X: Num + Copy + ToPrimitive + FromPrimitive + PartialOrd + ToString,
 Y: Num + Copy + ToPrimitive + FromPrimitive + PartialOrd + ToString> {
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
fn is_over_and_closest(pos: Point<f64>,
                       mouse_pos: Point<f64>,
                       rect_w: f64,
                       rect_h: f64,
                       pad_pos: Point<f64>,
                       pad_w: f64,
                       pad_h: f64,
                       perc_env: &Vec<(f32, f32, f32)>,
                       pt_radius: f64) -> (Option<Element>, Option<Element>) {
    match rectangle::is_over(pos, mouse_pos, rect_w, rect_h) {
        false => (None, None),
        true => match rectangle::is_over(pad_pos, mouse_pos, pad_w, pad_h) {
            false => (Some(Rect), Some(Rect)),
            true => {
                let mut closest_distance = ::std::f64::MAX_VALUE;
                let mut closest_env_point = Pad;
                for (i, p) in perc_env.iter().enumerate() {
                    let (x, y, _) = *p;
                    let p_pos = Point::new(map_range(x, 0.0, 1.0,
                                                     pad_pos.x, pad_pos.x + pad_w),
                                           map_range(y, 0.0, 1.0,
                                                     pad_pos.y + pad_h, pad_pos.y), 0.0);
                    let distance = (mouse_pos.x - p_pos.x).powf(2.0)
                                 + (mouse_pos.y - p_pos.y).powf(2.0);
                    if distance <= pt_radius.powf(2.0) {
                        return (Some(EnvPoint(i, p_pos)), Some(EnvPoint(i, p_pos)))
                    }
                    else if distance < closest_distance {
                        closest_distance = distance;
                        closest_env_point = EnvPoint(i, p_pos);
                    }
                }
                (Some(Pad), Some(closest_env_point))
            },
        },
    }
}

/// Determine and return the new state from the previous
/// state and the mouse position.
fn get_new_state(is_over_elem: Option<Element>,
                 prev: State,
                 mouse: MouseState) -> State {
    match (is_over_elem, prev, mouse.left, mouse.right) {
        (Some(_), Normal, Down, Up) => Normal,
        (Some(elem), _, Up, Up) => Highlighted(elem),
        (Some(elem), Highlighted(_), Down, Up) => Clicked(elem, Left),
        (Some(_), Clicked(p_elem, m_button), Down, Up)
        | (Some(_), Clicked(p_elem, m_button), Up, Down) => {
            match p_elem {
                EnvPoint(idx, _) => Clicked(EnvPoint(idx, mouse.pos), m_button),
                CurvePoint(idx, _) => Clicked(CurvePoint(idx, mouse.pos), m_button),
                _ => Clicked(p_elem, m_button),
            }
        },
        (None, Clicked(p_elem, m_button), Down, Up) => {
            match (p_elem, m_button) {
                (EnvPoint(idx, _), Left) => Clicked(EnvPoint(idx, mouse.pos), Left),
                (CurvePoint(idx, _), Left) => Clicked(CurvePoint(idx, mouse.pos), Left),
                _ => Clicked(p_elem, Left),
            }
        },
        (Some(_), Highlighted(p_elem), Up, Down) => {
            match p_elem {
                EnvPoint(idx, _) => Clicked(EnvPoint(idx, mouse.pos), Right),
                CurvePoint(idx, _) => Clicked(CurvePoint(idx, mouse.pos), Right),
                _ => Clicked(p_elem, Right),
            }
        },
        _ => Normal,
    }
}

/// Draw a circle at the given position.
fn draw_circle(win_w: f64,
               win_h: f64,
               gl: &mut Gl,
               pos: Point<f64>,
               color: Color,
               radius: f64) {
    let context = &Context::abs(win_w, win_h);
    let (r, g, b, a) = color.as_tuple();
    context
        .ellipse(pos.x, pos.y, radius * 2.0, radius * 2.0)
        .rgba(r, g, b, a)
        .draw(gl)
}



/// A context on which the builder pattern can be implemented.
pub struct EnvelopeEditorContext<'a, X, Y, E:'a> {
    uic: &'a mut UIContext,
    ui_id: UIID,
    env: &'a mut Vec<E>,
    skew_y_range: f32,
    min_x: X, max_x: X,
    min_y: Y, max_y: Y,
    pt_radius: f64,
    line_width: f64,
    font_size: FontSize,
    pos: Point<f64>,
    width: f64,
    height: f64,
    maybe_callback: Option<|&mut Vec<E>, uint|:'a>,
    maybe_color: Option<Color>,
    maybe_frame: Option<(f64, Color)>,
    maybe_label: Option<(&'a str, FontSize, Color)>,
}

impl<'a, X, Y, E> EnvelopeEditorContext<'a, X, Y, E> {
    #[inline]
    pub fn point_radius(self, radius: f64) -> EnvelopeEditorContext<'a, X, Y, E> {
        EnvelopeEditorContext { pt_radius: radius, ..self }
    }
    #[inline]
    pub fn line_width(self, width: f64) -> EnvelopeEditorContext<'a, X, Y, E> {
        EnvelopeEditorContext { line_width: width, ..self }
    }
    #[inline]
    pub fn value_font_size(self, size: FontSize) -> EnvelopeEditorContext<'a, X, Y, E> {
        EnvelopeEditorContext { font_size: size, ..self }
    }
    #[inline]
    pub fn skew_y(self, skew: f32) -> EnvelopeEditorContext<'a, X, Y, E> {
        EnvelopeEditorContext { skew_y_range: skew, ..self }
    }
}

pub trait EnvelopeEditorBuilder
<'a, X: Num + Copy + ToPrimitive + FromPrimitive + PartialOrd + ToString,
     Y: Num + Copy + ToPrimitive + FromPrimitive + PartialOrd + ToString,
     E: EnvelopePoint<X, Y>> {
    /// An envelope editor builder method to be implemented by the UIContext.
    fn envelope_editor(&'a mut self, ui_id: UIID, env: &'a mut Vec<E>,
                       min_x: X, max_x: X, min_y: Y, max_y: Y) -> EnvelopeEditorContext<'a, X, Y, E>;
}

impl <'a, X: Num + Copy + ToPrimitive + FromPrimitive + PartialOrd + ToString,
          Y: Num + Copy + ToPrimitive + FromPrimitive + PartialOrd + ToString,
          E: EnvelopePoint<X, Y>> EnvelopeEditorBuilder<'a, X, Y, E> for UIContext {
    /// An envelope editor builder method to be implemented by the UIContext.
    fn envelope_editor(&'a mut self, ui_id: UIID, env: &'a mut Vec<E>,
                       min_x: X, max_x: X, min_y: Y, max_y: Y) -> EnvelopeEditorContext<'a, X, Y, E> {
        EnvelopeEditorContext {
            uic: self,
            ui_id: ui_id,
            env: env,
            skew_y_range: 1.0, // Default skew amount (no skew).
            min_x: min_x, max_x: max_x,
            min_y: min_y, max_y: max_y,
            pt_radius: 6.0, // Default envelope point radius.
            line_width: 2.0, // Default envelope line width.
            font_size: 18u32,
            pos: Point::new(0.0, 0.0, 0.0),
            width: 256.0,
            height: 128.0,
            maybe_callback: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_label: None,
        }
    }
}

impl_callable!(EnvelopeEditorContext, |&mut Vec<E>, uint|:'a, X, Y, E)
impl_colorable!(EnvelopeEditorContext, X, Y, E)
impl_frameable!(EnvelopeEditorContext, X, Y, E)
impl_labelable!(EnvelopeEditorContext, X, Y, E)
impl_positionable!(EnvelopeEditorContext, X, Y, E)
impl_shapeable!(EnvelopeEditorContext, X, Y, E)

impl<'a, X: Num + Copy + ToPrimitive + FromPrimitive + PartialOrd + ToString,
         Y: Num + Copy + ToPrimitive + FromPrimitive + PartialOrd + ToString,
         E: EnvelopePoint<X, Y>> ::draw::Drawable for EnvelopeEditorContext<'a, X, Y, E> {
    #[inline]
    fn draw(&mut self, gl: &mut Gl) {
        let state = *get_state(self.uic, self.ui_id);
        let mouse = self.uic.get_mouse_state();
        let skew = self.skew_y_range;
        let (min_x, max_x, min_y, max_y) = (self.min_x, self.max_x, self.min_y, self.max_y);
        let pt_radius = self.pt_radius;
        let font_size = self.font_size;

        // Rect.
        let color = self.maybe_color.unwrap_or(::std::default::Default::default());
        let frame_w = match self.maybe_frame { Some((w, _)) => w, None => 0.0 };
        let frame_w2 = frame_w * 2.0;
        let pad_pos = self.pos + Point::new(frame_w, frame_w, 0.0);
        let pad_w = self.width - frame_w2;
        let pad_h = self.height - frame_w2;

        // Create a vector with each EnvelopePoint value represented as a
        // skewed percentage between 0.0 .. 1.0 .
        let perc_env: Vec<(f32, f32, f32)> = self.env.iter().map(|pt| {
            (percentage(pt.get_x(), min_x, max_x),
             percentage(pt.get_y(), min_y, max_y).powf(1.0 / skew),
             pt.get_curve())
        }).collect();

        // Check for new state.
        let (is_over_elem, is_closest_elem) = is_over_and_closest(
            self.pos, mouse.pos, self.width, self.height,
            pad_pos, pad_w, pad_h, &perc_env, pt_radius
        );
        let new_state = get_new_state(is_over_elem, state, mouse);

        // Draw rect.
        rectangle::draw(self.uic.win_w, self.uic.win_h, gl,
                        new_state.as_rectangle_state(),
                        self.pos, self.width, self.height, self.maybe_frame, color);

        // If there's a label, draw it.
        match self.maybe_label {
            None => (),
            Some((l_text, l_size, l_color)) => {
                let l_w = label::width(self.uic, l_size, l_text);
                let pad_x = pad_pos.x + (pad_w - l_w) / 2.0;
                let pad_y = pad_pos.y + (pad_h - l_size as f64) / 2.0;
                let l_pos = Point::new(pad_x, pad_y, 0.0);
                label::draw(gl, self.uic, l_pos, l_size, l_color, l_text);
            },
        };

        // Draw the envelope lines.
        match self.env.len() {
            0u | 1u => (),
            _ => {
                let (r, g, b, a) = color.plain_contrast().as_tuple();
                for i in range(1u, perc_env.len()) {
                    let (x_a, y_a, _) = perc_env[i - 1u];
                    let (x_b, y_b, _) = perc_env[i];
                    let p_a = Point::new(map_range(x_a, 0.0, 1.0,
                                                   pad_pos.x, pad_pos.x + pad_w),
                                         map_range(y_a, 0.0, 1.0,
                                                   pad_pos.y + pad_h, pad_pos.y), 0.0);
                    let p_b = Point::new(map_range(x_b, 0.0, 1.0,
                                                   pad_pos.x, pad_pos.x + pad_w),
                                         map_range(y_b, 0.0, 1.0,
                                                   pad_pos.y + pad_h, pad_pos.y), 0.0);
                    let context = Context::abs(self.uic.win_w, self.uic.win_h);
                    context
                        .line(p_a.x, p_a.y, p_b.x, p_b.y)
                        .round_border_width(self.line_width)
                        .rgba(r, g, b, a)
                        .draw(gl);
                }
            },
        }

        // Determine the left and right X bounds for a point.
        let get_x_bounds = |envelope_perc: &Vec<(f32, f32, f32)>, idx: uint| -> (f32, f32) {
            let right_bound = if envelope_perc.len() > 0u && envelope_perc.len() - 1u > idx {
                (*envelope_perc)[idx + 1u].val0()
            } else { 1.0 };
            let left_bound = if envelope_perc.len() > 0u && idx > 0u {
                (*envelope_perc)[idx - 1u].val0()
            } else { 0.0 };
            (left_bound, right_bound)
        };

        // Draw the (closest) envelope point and it's label and
        // return the idx if it is currently clicked.
        let is_clicked_env_point = match (state, new_state) {

            (_, Clicked(elem, _)) | (_, Highlighted(elem)) => {

                // Draw the envelope point.
                let draw_env_pt = |uic: &mut UIContext, envelope: &mut Vec<E>, idx: uint, p_pos: Point<f64>| {
                    let x_string = val_to_string(
                        (*envelope)[idx].get_x(),
                        max_x, max_x - min_x, pad_w as uint
                    );
                    let y_string = val_to_string(
                        (*envelope)[idx].get_y(),
                        max_y, max_y - min_y, pad_h as uint
                    );
                    let xy_string = x_string.append(", ").append(y_string.as_slice());
                    let xy_string_w = label::width(uic, font_size, xy_string.as_slice());
                    let xy_string_pos = match rectangle::corner(pad_pos, p_pos, pad_w, pad_h) {
                        TopLeft => Point::new(p_pos.x, p_pos.y, 0.0),
                        TopRight => Point::new(p_pos.x - xy_string_w, p_pos.y, 0.0),
                        BottomLeft => Point::new(p_pos.x, p_pos.y - font_size as f64, 0.0),
                        BottomRight => Point::new(p_pos.x - xy_string_w, p_pos.y - font_size as f64, 0.0),
                    };
                    label::draw(gl, uic, xy_string_pos,
                                font_size, color.plain_contrast(), xy_string.as_slice());
                    draw_circle(uic.win_w, uic.win_h, gl,
                                p_pos - Point::new(pt_radius, pt_radius, 0.0),
                                color.plain_contrast(), pt_radius);
                };

                match elem {
                    // If a point is clicked, draw that point.
                    EnvPoint(idx, p_pos) => {
                        let pad_x_right = pad_pos.x + pad_w;
                        let (left_x_bound, right_x_bound) = get_x_bounds(&perc_env, idx);
                        let left_pixel_bound = map_range(left_x_bound, 0.0, 1.0, pad_pos.x, pad_x_right);
                        let right_pixel_bound = map_range(right_x_bound, 0.0, 1.0, pad_pos.x, pad_x_right);
                        let p_pos_x_clamped = clamp(p_pos.x, left_pixel_bound, right_pixel_bound);
                        let p_pos_y_clamped = clamp(p_pos.y, pad_pos.y, pad_pos.y + pad_h);
                        draw_env_pt(self.uic, self.env, idx,
                                    Point::new(p_pos_x_clamped, p_pos_y_clamped, 0.0));
                        Some(idx)
                    },
                    // Otherwise, draw the closest point.
                    Pad => {
                        for closest_elem in is_closest_elem.iter() {
                            match *closest_elem {
                                EnvPoint(closest_idx, closest_env_pt) => {
                                    draw_env_pt(self.uic, self.env, closest_idx, closest_env_pt);
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
        let get_new_value = |perc_envelope: &Vec<(f32, f32, f32)>,
                             idx: uint,
                             mouse_x: f64,
                             mouse_y: f64| -> (X, Y) {
            let mouse_x_on_pad = mouse_x - pad_pos.x;
            let mouse_y_on_pad = mouse_y - pad_pos.y;
            let mouse_x_clamped = clamp(mouse_x_on_pad, 0f64, pad_w);
            let mouse_y_clamped = clamp(mouse_y_on_pad, 0.0, pad_h);
            let new_x_perc = percentage(mouse_x_clamped, 0f64, pad_w);
            let new_y_perc = percentage(mouse_y_clamped, pad_h, 0f64).powf(skew);
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
                    (Clicked(_, m_button), Highlighted(_)) | (Clicked(_, m_button), Normal) => {
                        match m_button {
                            Left => {
                                // Adjust the point and trigger the callback.
                                let (new_x, new_y) = get_new_value(&perc_env, idx, mouse.pos.x, mouse.pos.y);
                                self.env.get_mut(idx).set_x(new_x);
                                self.env.get_mut(idx).set_y(new_y);
                                match self.maybe_callback {
                                    Some(ref mut callback) => (*callback)(self.env, idx),
                                    None => (),
                                }
                            },
                            Right => {
                                // Delete the point and trigger the callback.
                                self.env.remove(idx);
                                match self.maybe_callback {
                                    Some(ref mut callback) => (*callback)(self.env, idx),
                                    None => (),
                                }
                            },
                        }
                    },

                    (Clicked(_, prev_m_button), Clicked(_, m_button)) => {
                        match (prev_m_button, m_button) {
                            (Left, Left) => {
                                let (new_x, new_y) = get_new_value(&perc_env, idx, mouse.pos.x, mouse.pos.y);
                                let current_x = (*self.env)[idx].get_x();
                                let current_y = (*self.env)[idx].get_y();
                                if new_x != current_x || new_y != current_y {
                                    // Adjust the point and trigger the callback.
                                    self.env.get_mut(idx).set_x(new_x);
                                    self.env.get_mut(idx).set_y(new_y);
                                    match self.maybe_callback {
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
                if self.env.len() == 0u {
                    match (state, new_state) {
                        (Clicked(elem, m_button), Highlighted(_)) => {
                            match (elem, m_button) {
                                (Pad, Left) => {
                                    let (new_x, new_y) = get_new_value(&perc_env, 0u, mouse.pos.x, mouse.pos.y);
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
                        (Clicked(elem, m_button), Highlighted(_)) => {
                            match (elem, m_button) {
                                (Pad, Left) => for closest_elem in is_closest_elem.iter() {
                                    match *closest_elem {
                                        EnvPoint(idx, p_pos) => {
                                            // Create a new point.
                                            let (new_x, new_y) = get_new_value(&perc_env, idx, mouse.pos.x, mouse.pos.y);
                                            let new_point = EnvelopePoint::new(new_x, new_y);
                                            let insert_idx = if mouse.pos.x > p_pos.x { idx + 1u }
                                                             else { idx };
                                            self.env.insert(insert_idx, new_point);
                                        }, _ => (),
                                    }
                                }, _ => (),
                            }
                        }, _ => (),
                    }
                }

            },

        }

        // Set the new state.
        set_state(self.uic, self.ui_id, new_state,
                  self.pos.x, self.pos.y, self.width, self.height);

    }
}

