
use color::Color;
use frame::{
    Framing,
    Frame,
    NoFrame,
};
use graphics::{
    Context,
    AddColor,
    AddRectangle,
    AddImage,
    Draw,
    RelativeTransform2d,
};
use label;
use label::{
    FontSize,
    Labeling,
    NoLabel,
    Label,
};
use mouse_state::{
    MouseState,
    Up,
    Down,
};
use opengl_graphics::Gl;
use point::Point;
use rectangle;
use utils::{
    clamp,
    compare_f64s,
};
use ui_context::{
    UIID,
    UIContext,
};
use widget::NumberDialer;

/// Represents the specific elements that the
/// NumberDialer is made up of. This is used to
/// specify which element is Highlighted or Clicked
/// when storing State.
#[deriving(Show, PartialEq)]
pub enum Element {
    Rect,
    LabelGlyphs,
    /// Represents a value glyph slot at `uint` index
    /// as well as the last mouse.pos.y for comparison
    /// in determining new value.
    ValueGlyph(uint, f64)
}

/// Represents the state of the Button widget.
#[deriving(PartialEq)]
pub enum State {
    Normal,
    Highlighted(Element),
    Clicked(Element),
}

widget_fns!(NumberDialer, State, NumberDialer(Normal))

/// Draw the number_dialer. When successfully pressed,
/// or if the value is changed, the given `callback`
/// function will be called.
pub fn draw<T: Num + Copy + Primitive + FromPrimitive + ToPrimitive + ToString>
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
     value: T,
     min: T,
     max: T,
     precision: u8,
     callback: |T|) {

    let val = clamp(value, min, max);
    let state = *get_state(uic, ui_id);
    let mouse = uic.get_mouse_state();
    let frame_w = match frame { Frame(frame_w, _) => frame_w, NoFrame => 0.0 };
    let (label_string, label_w, label_h) = label_string_and_dimensions(uic, label);
    let val_string_len = max.to_string().len() + if precision == 0 { 0u }
                                                 else { 1u + precision as uint };
    let mut val_string = create_val_string(val, val_string_len, precision);
    let (val_string_w, val_string_h) = val_string_dimensions(font_size, label, &val_string);
    /*let (rect_w, rect_h) = (val_string_w + label_w + RECT_PADDING * 2.0 + frame_w * 2.0,
                            val_string_h + RECT_PADDING * 2.0 + frame_w * 2.0);*/
    let l_pos = Point::new(pos.x + (width - (label_w + val_string_w)) / 2.0,
                           pos.y + (height - font_size as f64) / 2.0,
                           0.0);
    let is_over_elem = is_over(pos, frame_w, mouse.pos,
                               width, height,
                               l_pos, label_w, label_h,
                               val_string_w, val_string_h,
                               val_string.len());
    let new_state = get_new_state(is_over_elem, state, mouse);

    // Draw the widget rectangle.
    rectangle::draw(uic.win_w, uic.win_h, gl, rectangle::Normal,
                    pos, width, height, frame, color);

    // If there's a label, draw it.
    //let l_pos = pos + Point::new(RECT_PADDING + frame_w, RECT_PADDING + frame_w, 0.0);
    let (val_string_color, val_string_size) = match label {
        NoLabel => (color.plain_contrast(), font_size),
        Label(_, l_size, l_color) => {
            label::draw(gl, uic, l_pos, l_size, l_color, label_string.as_slice());
            (l_color, l_size)
        },
    };

    // Determine new value from the initial state and the new state.
    let new_val = match (state, new_state) {
        (Clicked(elem), Clicked(new_elem)) => {
            match (elem, new_elem) {
                (ValueGlyph(idx, y), ValueGlyph(_, new_y)) => {
                    get_new_value(val, min, max, idx, compare_f64s(new_y, y), &val_string)
                },
                _ => val,
            }
        },
        _ => val,
    };

    // If the value has changed, create a new string for val_string.
    if val != new_val { val_string = create_val_string(new_val, val_string_len, precision) }

    // Draw the value string.
    let val_string_pos = l_pos + Point::new(label_w, 0.0, 0.0);
    draw_value_string(uic.win_w, uic.win_h, gl, uic, new_state,
                      pos.y + frame_w, color, height, frame_w,
                      value_glyph_slot_width(val_string_size),
                      val_string_pos,
                      val_string_size,
                      val_string_color,
                      val_string.as_slice());
    set_state(uic, ui_id, new_state);

    // Call the `callback` with the new value if the mouse is pressed/released
    // on the widget or if the value has changed.
    if value != new_val || match (state, new_state) {
        (Highlighted(_), Clicked(_)) | (Clicked(_), Highlighted(_)) => true,
        _ => false,
    } { callback(new_val) };

}

/// Create the string to be drawn from the given values
/// and precision. Combine this with the label string if
/// one is given.
fn create_val_string<T: ToString>(val: T, len: uint, precision: u8) -> String {
    let mut val_string = val.to_string();
    // First check we have the correct number of decimal places.
    match (val_string.as_slice().chars().position(|ch| ch == '.'), precision) {
        (None, 0u8) => (),
        (None, _) => {
            val_string.push_char('.');
            val_string.grow(precision as uint, '0');
        },
        (Some(idx), 0u8) => {
            val_string.truncate(idx);
        },
        (Some(idx), _) => {
            let (len, desired_len) = (val_string.len(), idx + precision as uint + 1u);
            match len.cmp(&desired_len) {
                Greater => val_string.truncate(desired_len),
                Equal => (),
                Less => val_string.grow(desired_len - len, '0'),
            }
        },
    }
    // Now check that the total length matches. We already know that
    // the decimal end of the string is correct, so if the lengths
    // don't match we know we must prepend the difference as '0's.
    match val_string.len().cmp(&len) {
        Less => String::from_char(len - val_string.len(), '0').append(val_string.as_slice()),
        _ => val_string,
    }
}

/// Return the dimensions of a value glyph slot.
fn value_glyph_slot_width(size: FontSize) -> f64 {
    (size as f64 * 0.75).floor() as f64
}

/// Return the dimensions of the label.
fn label_string_and_dimensions(uic: &mut UIContext, label: Labeling) -> (String, f64, f64) {
    match label {
        NoLabel => (String::new(), 0f64, 0f64),
        Label(ref text, size, _) => {
            let string = text.to_string().append(": ");
            let label_width = label::width(uic, size, string.as_slice());
            (string, label_width, size as f64)
        },
    }
}

/// Return the dimensions of value string glyphs.
fn val_string_dimensions(font_size: FontSize,
                         label: Labeling,
                         val_string: &String) -> (f64, f64) {
    let size = match label {
        NoLabel => font_size,
        Label(_, label_font_size, _) => label_font_size,
    };
    let slot_w = value_glyph_slot_width(size);
    let val_string_w = slot_w * val_string.len() as f64;
    (val_string_w, size as f64)
}

/// Determine if the cursor is over the number_dialer and if so, which element.
fn is_over(pos: Point<f64>,
           frame_w: f64,
           mouse_pos: Point<f64>,
           rect_w: f64,
           rect_h: f64,
           l_pos: Point<f64>,
           label_w: f64,
           label_h: f64,
           val_string_w: f64,
           val_string_h: f64,
           val_string_len: uint) -> Option<Element> {
    match rectangle::is_over(pos, mouse_pos, rect_w, rect_h) {
        false => None,
        true => {
            match rectangle::is_over(l_pos, mouse_pos, label_w, label_h) {
                true => Some(LabelGlyphs),
                false => {
                    let frame_w2 = frame_w * 2.0;
                    let slot_rect_pos = Point::new(l_pos.x + label_w, pos.y + frame_w, 0.0);
                    match rectangle::is_over(slot_rect_pos, mouse_pos,
                                             val_string_w, rect_h - frame_w2) {
                        false => Some(Rect),
                        true => {
                            let slot_w = value_glyph_slot_width(val_string_h as u32);
                            let mut slot_pos = slot_rect_pos;
                            for i in range(0u, val_string_len) {
                                if rectangle::is_over(slot_pos, mouse_pos, slot_w, rect_h) {
                                    return Some(ValueGlyph(i, mouse_pos.y))
                                }
                                slot_pos.x += slot_w;
                            }
                            Some(Rect)
                        },
                    }
                },
            }
        },
    }
}

/// Check and return the current state of the NumberDialer.
fn get_new_state(is_over_elem: Option<Element>,
                 prev: State,
                 mouse: MouseState) -> State {
    match (is_over_elem, prev, mouse.left) {
        (Some(_), Normal, Down) => Normal,
        (Some(elem), _, Up) => Highlighted(elem),
        (Some(elem), Highlighted(_), Down) => Clicked(elem),
        (Some(_), Clicked(p_elem), Down) => {
            match p_elem {
                ValueGlyph(idx, _) => Clicked(ValueGlyph(idx, mouse.pos.y)),
                _ => Clicked(p_elem),
            }
        },
        (None, Clicked(p_elem), Down) => {
            match p_elem {
                ValueGlyph(idx, _) => Clicked(ValueGlyph(idx, mouse.pos.y)),
                _ => Clicked(p_elem),
            }
        },
        _ => Normal,
    }
}

/// Return the new value along with it's String representation.
fn get_new_value<T: Num + Copy + Primitive + FromPrimitive + ToPrimitive + ToString>
(val: T, min: T, max: T, idx: uint, y_ord: Ordering, val_string: &String) -> T {
    match y_ord {
        Equal => val,
        _ => {
            let decimal_pos = val_string.as_slice().chars().position(|ch| ch == '.');
            let val_f = val.to_f64().unwrap();
            let min_f = min.to_f64().unwrap();
            let max_f = max.to_f64().unwrap();
            let new_val_f = match decimal_pos {
                None => {
                    let power = val_string.len() - idx - 1u;
                    match y_ord {
                        Less => clamp(val_f + (10f32).powf(power as f32) as f64, min_f, max_f),
                        Greater => clamp(val_f - (10f32).powf(power as f32) as f64, min_f, max_f),
                        _ => val_f,
                    }
                },
                Some(dec_idx) => {
                    let mut power = dec_idx as int - idx as int - 1;
                    if power < -1 { power += 1; }
                    match y_ord {
                        Less => clamp(val_f + (10f32).powf(power as f32) as f64, min_f, max_f),
                        Greater => clamp(val_f - (10f32).powf(power as f32) as f64, min_f, max_f),
                        _ => val_f,
                    }
                },
            };
            FromPrimitive::from_f64(new_val_f).unwrap()
        },
    }
            
}

/// Draw the value string glyphs.
fn draw_value_string(win_w: f64,
                     win_h: f64,
                     gl: &mut Gl,
                     uic: &mut UIContext,
                     state: State,
                     slot_y: f64,
                     rect_color: Color,
                     rect_h: f64,
                     frame_w: f64,
                     slot_w: f64,
                     pos: Point<f64>,
                     size: FontSize,
                     font_color: Color,
                     string: &str) {
    let mut x = 0;
    let y = 0;
    let (font_r, font_g, font_b, font_a) = font_color.as_tuple();
    let context = Context::abs(win_w, win_h).trans(pos.x, pos.y + size as f64);
    let half_slot_w = slot_w / 2.0;
    for (i, ch) in string.chars().enumerate() {
        let character = uic.get_character(size, ch);
        match state {
            Highlighted(elem) => match elem {
                ValueGlyph(idx, _) => {
                    let context_slot_y = slot_y - (pos.y + size as f64);
                    let rect_color = if idx == i { rect_color.highlighted() }
                                     else { rect_color };
                    draw_slot_rect(gl, &context,
                                   x as f64,
                                   context_slot_y,
                                   size as f64,
                                   rect_h - frame_w * 2.0,
                                   rect_color);
                },
                _ => (),
            },
            Clicked(elem) => match elem {
                ValueGlyph(idx, _) => {
                    let context_slot_y = slot_y - (pos.y + size as f64);
                    let rect_color = if idx == i { rect_color.clicked() }
                                     else { rect_color };
                    draw_slot_rect(gl, &context,
                                   x as f64,
                                   context_slot_y,
                                   size as f64,
                                   rect_h - frame_w * 2.0,
                                   rect_color);
                },
                _ => (),
            },
            _ => (),
        };
        let x_shift = half_slot_w - (character.glyph.advance().x >> 16) as f64 / 2.0;
        context.trans((x + character.bitmap_glyph.left() + x_shift as i32) as f64,
                      (y - character.bitmap_glyph.top()) as f64)
                        .image(&character.texture)
                        .rgba(font_r, font_g, font_b, font_a)
                        .draw(gl);
        x += slot_w as i32;
    }
}

/// Draw the slot behind the value.
#[inline]
fn draw_slot_rect(gl: &mut Gl, context: &Context,
                  x: f64, y: f64, w: f64, h: f64,
                  color: Color) {
    let (r, g, b, a) = color.as_tuple();
    context.rect(x, y, w, h).rgba(r, g, b, a).draw(gl)
}

