
use color::Color;
use frame::{
    Framing,
    Frame,
    NoFrame,
};
use graphics::{
    AddColor,
    AddEllipse,
    AddLine,
    AddRoundBorder,
    Context,
    Draw,
};
use label;
use label::{
    NoLabel,
    Label,
    FontSize,
    Labeling,
};
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
#[deriving(Show, PartialEq)]
pub enum Element {
    Rect,
    Pad,
    /// Represents an EnvelopePoint at `uint` index
    /// as well as the last mouse pos for comparison
    /// in determining new value.
    EnvPoint(uint, Point<f64>),
}

/// An enum to define which button is clicked.
#[deriving(Show, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
}

/// Represents the state of the xy_pad widget.
#[deriving(Show, PartialEq)]
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
    /// Return the X value.
    fn set_x(&mut self, _x: X);
    /// Return the Y value.
    fn set_y(&mut self, _y: Y);
    /// Create a new EnvPoint.
    fn new(_x: X, _y: Y) -> Self;
}

/// Draw the envelope_editor. When successfully pressed,
/// the given `callback` closure will be called with the
/// xy coordinates as params.
pub fn draw
<X: Num + Copy + ToPrimitive + FromPrimitive + PartialOrd + ToString,
 Y: Num + Copy + ToPrimitive + FromPrimitive + PartialOrd + ToString,
 E: EnvelopePoint<X, Y>>
           (gl: &mut Gl,
            uic: &mut UIContext,
            ui_id: UIID,
            pos: Point<f64>,
            width: f64,
            height: f64,
            font_size: FontSize,
            frame: Framing,
            color: Color,
            label: Labeling,
            env: &mut Vec<E>,
            maybe_skew_y_range: Option<f32>,
            min_x: X, max_x: X, // pub enum Ranging { AutoRange, Range(min, max) }
            min_y: Y, max_y: Y,
            pt_radius: f64,
            line_width: f64,
            callback: |&mut Vec<E>, uint|) {

    let state = *get_state(uic, ui_id);
    let mouse = uic.get_mouse_state();
    let skew = maybe_skew_y_range.unwrap_or(1.0);

    // Rect.
    let frame_w = match frame { Frame(w, _) => w, NoFrame => 0.0 };
    let frame_w2 = frame_w * 2.0;
    let pad_pos = pos + Point::new(frame_w, frame_w, 0.0);
    let pad_w = width - frame_w2;
    let pad_h = height - frame_w2;

    // Create a vector with each EnvelopePoint value represented as a
    // skewed percentage between 0.0 .. 1.0 .
    let perc_env: Vec<(f32, f32)> = env.iter().map(|pt| {
        (percentage(pt.get_x(), min_x, max_x),
         percentage(pt.get_y(), min_y, max_y).powf(1.0 / skew))
    }).collect();

    // Check for new state.
    let (is_over_elem, is_closest_elem) = is_over_and_closest(
        pos, mouse.pos, width, height, pad_pos, pad_w, pad_h, &perc_env, pt_radius
    );
    let new_state = get_new_state(is_over_elem, state, mouse);

    // Draw rect.
    rectangle::draw(uic.win_w, uic.win_h, gl, new_state.as_rectangle_state(),
                    pos, width, height, frame, color);

    // If there's a label, draw it.
    match label {
        NoLabel => (),
        Label(l_text, l_size, l_color) => {
            let l_w = label::width(uic, l_size, l_text);
            let l_pos = Point::new(pad_pos.x + (pad_w - l_w) / 2.0,
                                   pad_pos.y + (pad_h - l_size as f64) / 2.0,
                                   0.0);
            label::draw(gl, uic, l_pos, l_size, l_color, l_text);
        },
    };

    // Draw the envelope lines.
    match env.len() {
        0u | 1u => (),
        _ => {
            let (r, g, b, a) = color.plain_contrast().as_tuple();
            for (i, env_p) in perc_env.iter().enumerate().skip(1u) {
                let (x_a, y_a) = perc_env[i - 1u];
                let (x_b, y_b) = perc_env[i];
                let p_a = Point::new(map_range(x_a, 0.0, 1.0, pad_pos.x, pad_pos.x + pad_w),
                                     map_range(y_a, 0.0, 1.0, pad_pos.y + pad_h, pad_pos.y), 0.0);
                let p_b = Point::new(map_range(x_b, 0.0, 1.0, pad_pos.x, pad_pos.x + pad_w),
                                     map_range(y_b, 0.0, 1.0, pad_pos.y + pad_h, pad_pos.y), 0.0);
                let context = Context::abs(uic.win_w, uic.win_h);
                context
                    .line(p_a.x, p_a.y, p_b.x, p_b.y)
                    .round_border_width(line_width)
                    .rgba(r, g, b, a)
                    .draw(gl);
            }
        },
    }

    // Determine the left and right X bounds.
    let get_x_bounds = |envelope_perc: &Vec<(f32, f32)>, idx: uint| -> (f32, f32) {
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
            let draw_env_pt = ref |envelope: &mut Vec<E>, idx: uint, p_pos: Point<f64>| {
                let x_string = val_to_string((*envelope)[idx].get_x(), max_x, max_x - min_x, pad_w as uint);
                let y_string = val_to_string((*envelope)[idx].get_y(), max_y, max_y - min_y, pad_h as uint);
                let xy_string = x_string.append(", ").append(y_string.as_slice());
                let xy_string_w = label::width(uic, font_size, xy_string.as_slice());
                let xy_string_pos = match rectangle::corner(pad_pos, p_pos, pad_w, pad_h) {
                    TopLeft => Point::new(p_pos.x, p_pos.y, 0.0),
                    TopRight => Point::new(p_pos.x - xy_string_w, p_pos.y, 0.0),
                    BottomLeft => Point::new(p_pos.x, p_pos.y - font_size as f64, 0.0),
                    BottomRight => Point::new(p_pos.x - xy_string_w, p_pos.y - font_size as f64, 0.0),
                };
                label::draw(gl, uic, xy_string_pos, font_size, color.plain_contrast(), xy_string.as_slice());
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
                    draw_env_pt(env, idx, Point::new(p_pos_x_clamped, p_pos_y_clamped, 0.0));
                    Some(idx)
                },
                // Otherwise, draw the closest point.
                Pad => {
                    for closest_elem in is_closest_elem.iter() {
                        match *closest_elem {
                            EnvPoint(closest_idx, closest_env_pt) => {
                                draw_env_pt(env, closest_idx, closest_env_pt);
                            },
                            _ => (),
                        }
                    }
                    None
                }, _ => None,
            }

        }, _ => None,

    };

    // Set the new state.
    set_state(uic, ui_id, new_state);

    // Determine new values.
    let get_new_value = |perc_envelope: &Vec<(f32, f32)>,
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
                            env.get_mut(idx).set_x(new_x);
                            env.get_mut(idx).set_y(new_y);
                            callback(env, idx);
                        },
                        Right => {
                            // Delete the point and trigger the callback.
                            env.remove(idx);
                            callback(env, idx);
                        },
                    }
                },

                (Clicked(_, prev_m_button), Clicked(_, m_button)) => {
                    match (prev_m_button, m_button) {
                        (Left, Left) => {
                            let (new_x, new_y) = get_new_value(&perc_env, idx, mouse.pos.x, mouse.pos.y);
                            let current_x = (*env)[idx].get_x();
                            let current_y = (*env)[idx].get_y();
                            if new_x != current_x || new_y != current_y {
                                // Adjust the point and trigger the callback.
                                env.get_mut(idx).set_x(new_x);
                                env.get_mut(idx).set_y(new_y);
                                callback(env, idx);
                            }
                        }, _ => (),
                    }
                }, _ => (),
            }

        },

        None => {

            // Check if a there are no points. If there are
            // and the mouse was clicked, add a point.
            if env.len() == 0u {
                match (state, new_state) {
                    (Clicked(elem, m_button), Highlighted(_)) => {
                        match (elem, m_button) {
                            (Pad, Left) => {
                                let (new_x, new_y) = get_new_value(&perc_env, 0u, mouse.pos.x, mouse.pos.y);
                                let new_point = EnvelopePoint::new(new_x, new_y);
                                env.push(new_point);
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
                                        env.insert(insert_idx, new_point);
                                    }, _ => (),
                                }
                            }, _ => (),
                        }
                    }, _ => (),
                }
            }

        },

    }

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
                       perc_env: &Vec<(f32, f32)>,
                       pt_radius: f64) -> (Option<Element>, Option<Element>) {
    match rectangle::is_over(pos, mouse_pos, rect_w, rect_h) {
        false => (None, None),
        true => match rectangle::is_over(pad_pos, mouse_pos, pad_w, pad_h) {
            false => (Some(Rect), Some(Rect)),
            true => {
                let mut closest_distance = ::std::f64::MAX_VALUE;
                let mut closest_env_point = Pad;
                for (i, p) in perc_env.iter().enumerate() {
                    let (x, y) = *p;
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
                _ => Clicked(p_elem, m_button),
            }
        },
        (None, Clicked(p_elem, m_button), Down, Up) => {
            match (p_elem, m_button) {
                (EnvPoint(idx, _), Left) => Clicked(EnvPoint(idx, mouse.pos), Left),
                _ => Clicked(p_elem, Left),
            }
        },
        (Some(_), Highlighted(p_elem), Up, Down) => {
            match p_elem {
                EnvPoint(idx, _) => Clicked(EnvPoint(idx, mouse.pos), Right),
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

