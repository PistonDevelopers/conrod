
use color::{Color, Colorable};
use frame::Frameable;
use graphics::character::CharacterCache;
use label::{self, FontSize, Labelable};
use mouse::Mouse;
use num::{Float, NumCast};
use position::{self, Depth, Dimensions, HorizontalAlign, Point, Position, VerticalAlign};
use std::cmp::Ordering;
use std::iter::repeat;
use utils::clamp;
use ui::{UiId, Ui};
use vecmath::{vec2_add, vec2_sub};
use widget::Kind;

/// Represents the specific elements that the NumberDialer is made up of. This is used to specify
/// which element is Highlighted or Clicked when storing State.
#[derive(Clone, Copy, Debug, RustcEncodable, RustcDecodable, PartialEq)]
pub enum Element {
    Rect,
    LabelGlyphs,
    /// Represents a value glyph slot at `usize` index as well as the last mouse.xy.y for
    /// comparison in determining new value.
    ValueGlyph(usize, f64)
}

/// Represents the state of the Button widget.
#[derive(Clone, Copy, Debug, RustcEncodable, RustcDecodable, PartialEq)]
pub enum State {
    Normal,
    Highlighted(Element),
    Clicked(Element),
}

widget_fns!(NumberDialer, State, Kind::NumberDialer(State::Normal));

/// Create the string to be drawn from the given values and precision. Combine this with the label
/// string if one is given.
fn create_val_string<T: ToString>(val: T, len: usize, precision: u8) -> String {
    let mut val_string = val.to_string();
    // First check we have the correct number of decimal places.
    match (val_string.chars().position(|ch| ch == '.'), precision) {
        (None, 0) => (),
        (None, _) => {
            val_string.push('.');
            val_string.extend(repeat('0').take(precision as usize));
        },
        (Some(idx), 0) => {
            val_string.truncate(idx);
        },
        (Some(idx), _) => {
            let (len, desired_len) = (val_string.len(), idx + precision as usize + 1);
            match len.cmp(&desired_len) {
                Ordering::Greater => val_string.truncate(desired_len),
                Ordering::Equal => (),
                Ordering::Less => val_string.extend(repeat('0').take(desired_len - len)),
            }
        },
    }
    // Now check that the total length matches. We already know that the decimal end of the string
    // is correct, so if the lengths don't match we know we must prepend the difference as '0's.
    if val_string.len() < len {
        repeat('0').take(len - val_string.len()).chain(val_string.chars()).collect()
    } else {
        val_string
    }
}

/// Return the dimensions of a value glyph slot.
fn value_glyph_slot_width(size: FontSize) -> f64 {
    (size as f64 * 0.75).floor() as f64
}

/// Return the dimensions of value string glyphs.
fn val_string_width(font_size: FontSize, val_string: &String) -> f64 {
    let slot_w = value_glyph_slot_width(font_size);
    let val_string_w = slot_w * val_string.len() as f64;
    val_string_w
}

/// Determine if the cursor is over the number_dialer and if so, which element.
#[inline]
fn is_over(mouse_xy: Point,
           dim: Dimensions,
           pad_dim: Dimensions,
           label_xy: Point,
           label_dim: Dimensions,
           val_string_dim: Point,
           val_string_len: usize) -> Option<Element> {
    use utils::is_over_rect;
    if is_over_rect([0.0, 0.0], mouse_xy, dim) {
        if is_over_rect(label_xy, mouse_xy, label_dim) {
            Some(Element::LabelGlyphs)
        } else {
            let slot_w = value_glyph_slot_width(val_string_dim[1] as u32);
            let slot_rect_xy = [label_xy[0] + label_dim[0] / 2.0 + slot_w / 2.0, 0.0];
            let val_string_xy = [slot_rect_xy[0] - slot_w / 2.0 + val_string_dim[0] / 2.0, 0.0];
            if is_over_rect(val_string_xy, mouse_xy, [val_string_dim[0], pad_dim[1]]) {
                let mut slot_xy = slot_rect_xy;
                for i in 0..val_string_len {
                    if is_over_rect(slot_xy, mouse_xy, [slot_w, pad_dim[1]]) {
                        return Some(Element::ValueGlyph(i, mouse_xy[1]))
                    }
                    slot_xy[0] += slot_w;
                }
                Some(Element::Rect)
            } else {
                Some(Element::Rect)
            }
        }
    } else {
        None
    }
}

/// Check and return the current state of the NumberDialer.
#[inline]
fn get_new_state(is_over_elem: Option<Element>, prev: State, mouse: Mouse) -> State {
    use mouse::ButtonState::{Down, Up};
    use self::Element::ValueGlyph;
    use self::State::{Normal, Highlighted, Clicked};
    match (is_over_elem, prev, mouse.left) {
        (Some(_),    Normal,          Down) => Normal,
        (Some(elem), _,               Up)   => Highlighted(elem),
        (Some(elem), Highlighted(_),  Down) => Clicked(elem),
        (Some(_),    Clicked(p_elem), Down) => {
            match p_elem {
                ValueGlyph(idx, _) => Clicked(ValueGlyph(idx, mouse.xy[1])),
                _                  => Clicked(p_elem),
            }
        },
        (None,       Clicked(p_elem), Down) => {
            match p_elem {
                ValueGlyph(idx, _) => Clicked(ValueGlyph(idx, mouse.xy[1])),
                _                  => Clicked(p_elem),
            }
        },
        _                                   => Normal,
    }
}

/// A widget for precision control over any digit within a value. The callback is triggered when
/// the value is updated or if the mouse button is released while the cursor is above the widget.
pub struct NumberDialer<'a, T, F> {
    value: T,
    min: T,
    max: T,
    pos: Position,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    dim: Dimensions,
    depth: Depth,
    precision: u8,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_label: Option<&'a str>,
    maybe_label_color: Option<Color>,
    maybe_label_font_size: Option<u32>,
    maybe_callback: Option<F>,
}

impl<'a, T: Float, F> NumberDialer<'a, T, F> {

    /// Construct a new NumberDialer widget.
    pub fn new(value: T, min: T, max: T, precision: u8) -> NumberDialer<'a, T, F> {
        NumberDialer {
            value: clamp(value, min, max),
            min: min,
            max: max,
            pos: Position::default(),
            maybe_h_align: None,
            maybe_v_align: None,
            dim: [128.0, 48.0],
            depth: 0.0,
            precision: precision,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
            maybe_callback: None,
        }
    }

    /// Set the callback for the NumberDialer. It will be triggered when the value is updated or if
    /// the mouse button is released while the cursor is above the widget.
    pub fn callback(mut self, cb: F) -> NumberDialer<'a, T, F> {
        self.maybe_callback = Some(cb);
        self
    }

    /// After building the NumberDialer, use this method to set its current state into the given
    /// `Ui`. It will use this state for rendering the next time `ui.draw(graphics)` is called.
    pub fn set<C>(mut self, ui_id: UiId, ui: &mut Ui<C>)
        where
            C: CharacterCache,
            F: FnMut(T),
            T: ToString,
    {
        use elmesque::form::{collage, rect, text};
        use elmesque::text::Text;

        let state = *get_state(ui, ui_id);
        let dim = self.dim;
        let h_align = self.maybe_h_align.unwrap_or(ui.theme.h_align);
        let v_align = self.maybe_v_align.unwrap_or(ui.theme.v_align);
        let xy = ui.get_xy(self.pos, dim, h_align, v_align);
        let mouse = ui.get_mouse_state(ui_id).relative_to(xy);
        let frame_w = self.maybe_frame.unwrap_or(ui.theme.frame_width);
        let frame_w2 = frame_w * 2.0;
        let pad_dim = vec2_sub(dim, [frame_w2; 2]);
        let font_size = self.maybe_label_font_size.unwrap_or(ui.theme.font_size_medium);
        let label_string = self.maybe_label.map_or_else(|| String::new(), |text| format!("{}: ", text));
        let label_dim = [label::width(ui, font_size, &label_string), font_size as f64];
        let val_string_len = self.max.to_string().len() + if self.precision == 0 { 0 }
                                                          else { 1 + self.precision as usize };
        let mut val_string = create_val_string(self.value, val_string_len, self.precision);
        let val_string_dim = [val_string_width(font_size, &val_string), font_size as f64];
        let label_x = -val_string_dim[0] / 2.0;
        let label_pos = [label_x, 0.0];
        let is_over_elem = is_over(mouse.xy, dim, pad_dim, label_pos, label_dim, val_string_dim, val_string_len);
        let new_state = get_new_state(is_over_elem, state, mouse);

        // Construct the frame and inner rectangle Forms.
        let frame_color = self.maybe_frame_color.unwrap_or(ui.theme.frame_color);
        let color = self.maybe_color.unwrap_or(ui.theme.shape_color);
        let frame_form = rect(dim[0], dim[1]).filled(frame_color);
        let inner_form = rect(pad_dim[0], pad_dim[1]).filled(color);

        // Construct the label form.
        let val_string_color = self.maybe_label_color.unwrap_or(ui.theme.label_color);
        let label_form = text(Text::from_string(label_string.clone())
                                  .color(val_string_color)
                                  .height(font_size as f64)).shift_x(label_x.floor());

        // Determine new value from the initial state and the new state.
        let mut new_val = self.value;
        if let (State::Clicked(elem), State::Clicked(new_elem)) = (state, new_state) {
            if let (Element::ValueGlyph(idx, y), Element::ValueGlyph(_, new_y)) = (elem, new_elem) {
                let ord = new_y.partial_cmp(&y).unwrap_or(Ordering::Equal);
                if ord != Ordering::Equal {
                    let decimal_pos = val_string.chars().position(|ch| ch == '.');
                    let val_f: f64 = NumCast::from(self.value).unwrap();
                    let min_f: f64 = NumCast::from(self.min).unwrap();
                    let max_f: f64 = NumCast::from(self.max).unwrap();
                    let new_val_f = match decimal_pos {
                        None => {
                            let power = val_string.len() - idx - 1;
                            match ord {
                                Ordering::Greater => {
                                    clamp(val_f + (10.0).powf(power as f32) as f64, min_f, max_f)
                                },
                                Ordering::Less => {
                                    clamp(val_f - (10.0).powf(power as f32) as f64, min_f, max_f)
                                },
                                _ => val_f,
                            }
                        },
                        Some(dec_idx) => {
                            let mut power = dec_idx as isize - idx as isize - 1;
                            if power < -1 { power += 1; }
                            match ord {
                                Ordering::Greater => {
                                    clamp(val_f + (10.0).powf(power as f32) as f64, min_f, max_f)
                                },
                                Ordering::Less => {
                                    clamp(val_f - (10.0).powf(power as f32) as f64, min_f, max_f)
                                },
                                _ => val_f,
                            }
                        },
                    };
                    new_val = NumCast::from(new_val_f).unwrap()
                };
            }
        };

        // If the value has changed, create a new string for val_string.
        if self.value != new_val {
            val_string = create_val_string(new_val, val_string_len, self.precision);
        }

        // Construct the value_string's Form.
        let val_string_pos = vec2_add(label_pos, [label_dim[0] / 2.0, 0.0]);
        let slot_w = value_glyph_slot_width(font_size);
        let mut x = slot_w / 2.0;
        let val_string_forms = {
            val_string.chars().enumerate().flat_map(|(i, ch)| {
                let maybe_rect_form = match new_state {
                    State::Highlighted(elem) => if let Element::ValueGlyph(idx, _) = elem {
                        let rect_color = if idx == i { color.highlighted() }
                                         else { color };
                        Some(rect(slot_w, pad_dim[1]).filled(rect_color)
                             .shift(val_string_pos[0].floor(), val_string_pos[1].floor())
                             .shift_x(x.floor()))
                    } else {
                        None
                    },
                    State::Clicked(elem) => if let Element::ValueGlyph(idx, _) = elem {
                        let rect_color = if idx == i { color.clicked() }
                                         else { color };
                        Some(rect(slot_w, pad_dim[1]).filled(rect_color)
                             .shift(val_string_pos[0].floor(), val_string_pos[1].floor())
                             .shift_x(x.floor()))
                    } else {
                        None
                    },
                    _ => None,
                };
                let character_form = text(Text::from_string(ch.to_string())
                                              .color(val_string_color)
                                              .height(font_size as f64))
                                        .shift(val_string_pos[0].floor(), val_string_pos[1].floor())
                                        .shift_x(x.floor());
                x += slot_w;
                maybe_rect_form.into_iter().chain(Some(character_form).into_iter())
            })
        };

        // Call the `callback` with the new value if the mouse is pressed/released on the widget
        // or if the value has changed.
        if self.value != new_val || match (state, new_state) {
            (State::Highlighted(_), State::Clicked(_)) |
            (State::Clicked(_), State::Highlighted(_)) => true,
            _ => false,
        } {
            if let Some(ref mut callback) = self.maybe_callback { callback(new_val) }
        }

        // Chain the forms and shift them into position.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(inner_form).into_iter())
            .chain(Some(label_form).into_iter())
            .chain(val_string_forms)
            .map(|form| form.shift(xy[0].floor(), xy[1].floor()));

        // Collect the Forms into a renderable Element.
        let element = collage(dim[0] as i32, dim[1] as i32, form_chain.collect());

        // Store the button's new state in the Ui.
        ui.set_widget(ui_id, ::widget::Widget {
            kind: Kind::NumberDialer(new_state),
            xy: xy,
            depth: self.depth,
            element: Some(element),
        });

    }

}

impl<'a, T, F> Colorable for NumberDialer<'a, T, F> {
    fn color(mut self, color: Color) -> Self {
        self.maybe_color = Some(color);
        self
    }
}

impl<'a, T, F> Frameable for NumberDialer<'a, T, F> {
    fn frame(mut self, width: f64) -> Self {
        self.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, T, F> Labelable<'a> for NumberDialer<'a, T, F>
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

impl<'a, T, F> position::Positionable for NumberDialer<'a, T, F> {
    fn position(mut self, pos: Position) -> Self {
        self.pos = pos;
        self
    }
    #[inline]
    fn horizontal_align(self, h_align: HorizontalAlign) -> Self {
        NumberDialer { maybe_h_align: Some(h_align), ..self }
    }
    #[inline]
    fn vertical_align(self, v_align: VerticalAlign) -> Self {
        NumberDialer { maybe_v_align: Some(v_align), ..self }
    }
}

impl<'a, T, F> position::Sizeable for NumberDialer<'a, T, F> {
    #[inline]
    fn width(self, w: f64) -> Self {
        let h = self.dim[1];
        NumberDialer { dim: [w, h], ..self }
    }
    #[inline]
    fn height(self, h: f64) -> Self {
        let w = self.dim[0];
        NumberDialer { dim: [w, h], ..self }
    }
}

