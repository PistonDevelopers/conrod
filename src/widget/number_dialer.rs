
use color::{Color, Colorable};
use elmesque::Element;
use frame::Frameable;
use graphics::character::CharacterCache;
use graphics::math::Scalar;
use label::{FontSize, Labelable};
use mouse::Mouse;
use num::{Float, NumCast};
use position::{Dimensions, Point};
use std::any::Any;
use std::cmp::Ordering;
use std::iter::repeat;
use theme::Theme;
use utils::clamp;
use ui::GlyphCache;
use widget::{self, Widget};


/// A widget for precision control over any digit within a value. The reaction is triggered when
/// the value is updated or if the mouse button is released while the cursor is above the widget.
pub struct NumberDialer<'a, T, F> {
    common: widget::CommonBuilder,
    value: T,
    min: T,
    max: T,
    maybe_label: Option<&'a str>,
    precision: u8,
    maybe_react: Option<F>,
    style: Style,
    enabled: bool,
}

/// Styling for the NumberDialer, necessary for constructing its renderable Element.
#[allow(missing_docs, missing_copy_implementations)]
#[derive(Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    pub maybe_color: Option<Color>,
    pub maybe_frame: Option<f64>,
    pub maybe_frame_color: Option<Color>,
    pub maybe_label_color: Option<Color>,
    pub maybe_label_font_size: Option<u32>,
}

/// Represents the specific elements that the NumberDialer is made up of. This is used to specify
/// which element is Highlighted or Clicked when storing State.
#[derive(Clone, Copy, Debug, RustcEncodable, RustcDecodable, PartialEq)]
pub enum Elem {
    Rect,
    LabelGlyphs,
    /// Represents a value glyph slot at `usize` index as well as the last mouse.xy.y for
    /// comparison in determining new value.
    ValueGlyph(usize, f64)
}

/// The current interaction with the NumberDialer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Interaction {
    Normal,
    Highlighted(Elem),
    Clicked(Elem),
}

/// The state of the NumberDialer.
#[derive(Clone, Debug, PartialEq)]
pub struct State<T> {
    value: T,
    min: T,
    max: T,
    precision: u8,
    maybe_label: Option<String>,
    interaction: Interaction,
}


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
fn is_over(mouse_xy: Point,
           dim: Dimensions,
           pad_dim: Dimensions,
           label_x: f64,
           label_dim: Dimensions,
           val_string_dim: Point,
           val_string_len: usize) -> Option<Elem> {
    use position::is_over_rect;
    if is_over_rect([0.0, 0.0], dim, mouse_xy) {
        if is_over_rect([label_x, 0.0], label_dim, mouse_xy) {
            Some(Elem::LabelGlyphs)
        } else {
            let slot_w = value_glyph_slot_width(val_string_dim[1] as u32);
            let slot_rect_xy = [label_x + label_dim[0] / 2.0 + slot_w / 2.0, 0.0];
            let val_string_xy = [slot_rect_xy[0] - slot_w / 2.0 + val_string_dim[0] / 2.0, 0.0];
            if is_over_rect(val_string_xy, [val_string_dim[0], pad_dim[1]], mouse_xy) {
                let mut slot_xy = slot_rect_xy;
                for i in 0..val_string_len {
                    if is_over_rect(slot_xy, [slot_w, pad_dim[1]], mouse_xy) {
                        return Some(Elem::ValueGlyph(i, mouse_xy[1]))
                    }
                    slot_xy[0] += slot_w;
                }
                Some(Elem::Rect)
            } else {
                Some(Elem::Rect)
            }
        }
    } else {
        None
    }
}

/// Check and return the current state of the NumberDialer.
fn get_new_interaction(is_over_elem: Option<Elem>, prev: Interaction, mouse: Mouse) -> Interaction {
    use mouse::ButtonPosition::{Down, Up};
    use self::Elem::ValueGlyph;
    use self::Interaction::{Normal, Highlighted, Clicked};
    match (is_over_elem, prev, mouse.left.position) {
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


impl<'a, T, F> NumberDialer<'a, T, F> where T: Float {

    /// Construct a new NumberDialer widget.
    pub fn new(value: T, min: T, max: T, precision: u8) -> NumberDialer<'a, T, F> {
        NumberDialer {
            common: widget::CommonBuilder::new(),
            value: clamp(value, min, max),
            min: min,
            max: max,
            precision: precision,
            maybe_label: None,
            maybe_react: None,
            style: Style::new(),
            enabled: true,
        }
    }

    /// Set the reaction for the NumberDialer. It will be triggered when the value is updated or if
    /// the mouse button is released while the cursor is above the widget.
    pub fn react(mut self, reaction: F) -> NumberDialer<'a, T, F> {
        self.maybe_react = Some(reaction);
        self
    }

    /// If true, will allow user inputs.  If false, will disallow user inputs.
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
    }

}

impl<'a, T, F> Widget for NumberDialer<'a, T, F> where
    F: FnMut(T),
    T: Any + ::std::fmt::Debug + Float + NumCast + ToString,
{
    type State = State<T>;
    type Style = Style;
    fn common(&self) -> &widget::CommonBuilder { &self.common }
    fn common_mut(&mut self) -> &mut widget::CommonBuilder { &mut self.common }
    fn unique_kind(&self) -> &'static str { "NumberDialer" }
    fn init_state(&self) -> State<T> {
        State {
            value: self.value,
            min: self.min,
            max: self.max,
            precision: self.precision,
            maybe_label: None,
            interaction: Interaction::Normal,
        }
    }
    fn style(&self) -> Style { self.style.clone() }

    fn default_width<C: CharacterCache>(&self, theme: &Theme, _: &GlyphCache<C>) -> Scalar {
        const DEFAULT_WIDTH: Scalar = 128.0;
        theme.maybe_number_dialer.as_ref().map(|default| {
            default.common.maybe_width.unwrap_or(DEFAULT_WIDTH)
        }).unwrap_or(DEFAULT_WIDTH)
    }

    fn default_height(&self, theme: &Theme) -> Scalar {
        const DEFAULT_HEIGHT: Scalar = 48.0;
        theme.maybe_number_dialer.as_ref().map(|default| {
            default.common.maybe_height.unwrap_or(DEFAULT_HEIGHT)
        }).unwrap_or(DEFAULT_HEIGHT)
    }

    /// Update the state of the NumberDialer.
    fn update<C: CharacterCache>(mut self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { state, rect, style, mut ui, .. } = args;

        let (xy, dim) = rect.xy_dim();
        let maybe_mouse = ui.input().maybe_mouse.map(|mouse| mouse.relative_to(xy));
        let frame = style.frame(ui.theme());
        let pad_dim = ::vecmath::vec2_sub(dim, [frame * 2.0; 2]);
        let font_size = style.label_font_size(ui.theme());
        let label_string = self.maybe_label.map_or_else(|| String::new(), |text| format!("{}: ", text));
        let label_dim = [ui.glyph_cache().width(font_size, &label_string), font_size as f64];
        let val_string_len = self.max.to_string().len() + if self.precision == 0 { 0 }
                                                          else { 1 + self.precision as usize };
        let val_string = create_val_string(self.value, val_string_len, self.precision);
        let val_string_dim = [val_string_width(font_size, &val_string), font_size as f64];
        let label_x = -val_string_dim[0] / 2.0;
        let new_interaction = match (self.enabled, maybe_mouse) {
            (false, _) | (true, None) => Interaction::Normal,
            (true, Some(mouse)) => {
                let is_over_elem = is_over(mouse.xy, dim, pad_dim, label_x, label_dim,
                                           val_string_dim, val_string_len);
                get_new_interaction(is_over_elem, state.view().interaction, mouse)
            },
        };

        // Capture the mouse if clicked, uncapture if released.
        match (state.view().interaction, new_interaction) {
            (Interaction::Highlighted(_), Interaction::Clicked(_)) => { ui.capture_mouse(); },
            (Interaction::Clicked(_), Interaction::Highlighted(_)) |
            (Interaction::Clicked(_), Interaction::Normal)         => { ui.uncapture_mouse(); },
            _ => (),
        }

        // Determine new value from the initial state and the new state.
        let mut new_val = self.value;
        if let (Interaction::Clicked(elem), Interaction::Clicked(new_elem)) =
            (state.view().interaction, new_interaction) {
            if let (Elem::ValueGlyph(idx, y), Elem::ValueGlyph(_, new_y)) = (elem, new_elem) {
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

        // Call the `react` with the new value if the mouse is pressed/released on the widget
        // or if the value has changed.
        if self.value != new_val || match (state.view().interaction, new_interaction) {
            (Interaction::Highlighted(_), Interaction::Clicked(_)) |
            (Interaction::Clicked(_), Interaction::Highlighted(_)) => true,
            _ => false,
        } {
            if let Some(ref mut react) = self.maybe_react { react(new_val) }
        }

        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        if state.view().value != new_val {
            state.update(|state| state.value = new_val);
        }

        if state.view().min != self.min {
            state.update(|state| state.min = self.min);
        }

        if state.view().max != self.max {
            state.update(|state| state.max = self.max);
        }

        if state.view().precision != self.precision {
            state.update(|state| state.precision = self.precision);
        }

        if state.view().maybe_label.as_ref().map(|label| &label[..]) != self.maybe_label {
            state.update(|state| {
                state.maybe_label = self.maybe_label.as_ref().map(|label| label.to_string());
            });
        }
    }

    /// Construct an Element from the given NumberDialer State.
    fn draw<C>(args: widget::DrawArgs<Self, C>) -> Element
        where C: CharacterCache,
    {
        use elmesque::form::{self, collage, text};
        use elmesque::text::Text;

        let widget::DrawArgs { rect, state, style, theme, glyph_cache, .. } = args;

        // Construct the frame and inner rectangle Forms.
        let (xy, dim) = rect.xy_dim();
        let frame = style.frame(theme);
        let pad_dim = ::vecmath::vec2_sub(dim, [frame * 2.0; 2]);
        let frame_color = style.frame_color(theme);
        let color = style.color(theme);
        let frame_form = form::rect(dim[0], dim[1]).filled(frame_color);
        let inner_form = form::rect(pad_dim[0], pad_dim[1]).filled(color);
        let val_string_len = state.max.to_string().len() + if state.precision == 0 { 0 }
                                                          else { 1 + state.precision as usize };
        let label_string = state.maybe_label.as_ref()
            .map_or_else(|| String::new(), |text| format!("{}: ", text));
        let font_size = style.label_font_size(theme);

        // If the value has changed, create a new string for val_string.
        let val_string = create_val_string(state.value, val_string_len, state.precision);
        let val_string_dim = [val_string_width(font_size, &val_string), font_size as f64];
        let label_x = -val_string_dim[0] / 2.0;
        let label_dim = [glyph_cache.width(font_size, &label_string), font_size as f64];

        // Construct the label form.
        let val_string_color = style.label_color(theme);
        let label_form = text(Text::from_string(label_string.clone())
                                  .color(val_string_color)
                                  .height(font_size as f64)).shift_x(label_x.floor());

        // Construct the value_string's Form.
        let val_string_pos = [label_x + label_dim[0] / 2.0, 0.0];
        let slot_w = value_glyph_slot_width(font_size);
        let mut x = slot_w / 2.0;
        let val_string_forms = {
            val_string.chars().enumerate().flat_map(|(i, ch)| {
                let maybe_rect_form = match state.interaction {
                    Interaction::Highlighted(elem) => if let Elem::ValueGlyph(idx, _) = elem {
                        let rect_color = if idx == i { color.highlighted() }
                                         else { color };
                        Some(form::rect(slot_w, pad_dim[1]).filled(rect_color)
                             .shift(val_string_pos[0].floor(), val_string_pos[1].floor())
                             .shift_x(x.floor()))
                    } else {
                        None
                    },
                    Interaction::Clicked(elem) => if let Elem::ValueGlyph(idx, _) = elem {
                        let rect_color = if idx == i { color.clicked() }
                                         else { color };
                        Some(form::rect(slot_w, pad_dim[1]).filled(rect_color)
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
                maybe_rect_form.into_iter().chain(Some(character_form))
            })
        };

        // Chain the forms and shift them into position.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(inner_form))
            .chain(Some(label_form))
            .chain(val_string_forms)
            .map(|form| form.shift(xy[0].floor(), xy[1].floor()));

        // Collect the Forms into a renderable Element.
        collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
    }

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
        }
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.maybe_number_dialer.as_ref().map(|default| {
            default.style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.maybe_number_dialer.as_ref().map(|default| {
            default.style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.maybe_number_dialer.as_ref().map(|default| {
            default.style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label Color for an Element.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.maybe_number_dialer.as_ref().map(|default| {
            default.style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn label_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.maybe_number_dialer.as_ref().map(|default| {
            default.style.maybe_label_font_size.unwrap_or(theme.font_size_medium)
        })).unwrap_or(theme.font_size_medium)
    }

}


impl<'a, T, F> Colorable for NumberDialer<'a, T, F> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

impl<'a, T, F> Frameable for NumberDialer<'a, T, F> {
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
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
        self.style.maybe_label_color = Some(color);
        self
    }

    fn label_font_size(mut self, size: FontSize) -> Self {
        self.style.maybe_label_font_size = Some(size);
        self
    }
}

