
use {
    CharacterCache,
    Color,
    Colorable,
    Dimensions,
    FontSize,
    Frameable,
    FramedRectangle,
    IndexSlot,
    Labelable,
    Mouse,
    NodeIndex,
    Point,
    Positionable,
    Rectangle,
    Sizeable,
    Text,
    Theme,
    Widget,
};
use num::{Float, NumCast};
use std::any::Any;
use std::cmp::Ordering;
use std::iter::repeat;
use utils::clamp;
use widget;


/// A widget for precision control over any digit within a value.
///
/// The reaction is triggered when the value is updated or if the mouse button is released while
/// the cursor is above the widget.
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
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    /// Color of the NumberDialer's rectangle.
    pub maybe_color: Option<Color>,
    /// The frame around the NumberDialer's rectangle.
    pub maybe_frame: Option<f64>,
    /// The color of the rectangle frame.
    pub maybe_frame_color: Option<Color>,
    /// The color of the NumberDialer's label.
    pub maybe_label_color: Option<Color>,
    /// The font size for the NumberDialer's label.
    pub maybe_label_font_size: Option<u32>,
}

/// Unique kind for the widget.
pub const KIND: widget::Kind = "NumberDialer";

/// Represents the specific elements that the NumberDialer is made up of. This is used to specify
/// which element is Highlighted or Clicked when storing State.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Elem {
    Rect,
    LabelGlyphs,
    /// Represents a value glyph slot at `usize` index as well as the last mouse.xy.y for
    /// comparison in determining new value.
    ValueGlyph(usize, f64)
}

/// The current interaction with the NumberDialer.
#[derive(Copy, Clone, Debug, PartialEq)]
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
    interaction: Interaction,
    rectangle_idx: IndexSlot,
    label_idx: IndexSlot,
    glyph_slot_indices: Vec<GlyphSlot>,
}

/// Each digit in the adjustable value has its own **Rectangle** and **Text** widgets.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GlyphSlot {
    /// The highlightable **Rectangle** behind the glyph.
    rectangle_idx: NodeIndex,
    /// The **Text** widget for the glyph itself.
    text_idx: NodeIndex,
}


/// Create the string to be drawn from the given values and precision.
///
/// Combine this with the label string if one is given.
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
           val_string_len: usize) -> Option<Elem>
{
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
        (Some(_),    Normal,          Down(_, _)) => Normal,
        (Some(elem), _,               Up)   => Highlighted(elem),
        (Some(elem), Highlighted(_),  Down(_, _)) => Clicked(elem),
        (Some(_),    Clicked(p_elem), Down(_, _)) => {
            match p_elem {
                ValueGlyph(idx, _) => Clicked(ValueGlyph(idx, mouse.xy[1])),
                _                  => Clicked(p_elem),
            }
        },
        (None,       Clicked(p_elem), Down(_, _)) => {
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
    pub fn new(value: T, min: T, max: T, precision: u8) -> Self {
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
    pub fn react(mut self, reaction: F) -> Self {
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
    F: FnOnce(T),
    T: Any + ::std::fmt::Debug + Float + NumCast + ToString,
{
    type State = State<T>;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> &'static str {
        KIND
    }

    fn init_state(&self) -> State<T> {
        State {
            value: self.value,
            min: self.min,
            max: self.max,
            precision: self.precision,
            interaction: Interaction::Normal,
            rectangle_idx: IndexSlot::new(),
            label_idx: IndexSlot::new(),
            glyph_slot_indices: Vec::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the NumberDialer.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        use self::Interaction::{Clicked, Highlighted, Normal};

        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let NumberDialer {
            value, min, max, precision, enabled, maybe_label, maybe_react, ..
        } = self;

        let maybe_mouse = ui.input().maybe_mouse.map(|mouse| mouse.relative_to(rect.xy()));
        let frame = style.frame(ui.theme());
        let inner_rect = rect.pad(frame);
        let font_size = style.label_font_size(ui.theme());
        let label_string = maybe_label.map_or_else(|| String::new(), |text| format!("{}: ", text));
        let label_dim = [ui.glyph_cache().width(font_size, &label_string), font_size as f64];
        let precision_len = if precision == 0 { 0 } else { precision as usize + 1 };
        let val_string_len = max.to_string().len() + precision_len;
        let val_string = create_val_string(value, val_string_len, precision);
        let val_string_dim = [val_string_width(font_size, &val_string), font_size as f64];
        let label_rel_x = -val_string_dim[0] / 2.0;
        let interaction = state.view().interaction;
        let new_interaction = match (enabled, maybe_mouse) {
            (false, _) | (true, None) => Normal,
            (true, Some(mouse)) => {
                let is_over_elem = is_over(mouse.xy, rect.dim(), inner_rect.dim(), label_rel_x,
                                           label_dim, val_string_dim, val_string_len);
                get_new_interaction(is_over_elem, interaction, mouse)
            },
        };

        // Capture the mouse if clicked, uncapture if released.
        match (interaction, new_interaction) {
            (Highlighted(_), Clicked(_)) => { ui.capture_mouse(); },
            (Clicked(_), Highlighted(_)) |
            (Clicked(_), Normal)         => { ui.uncapture_mouse(); },
            _ => (),
        }

        // Determine new value from the initial state and the new state.
        let mut new_val = value;
        if let (Clicked(elem), Clicked(new_elem)) = (interaction, new_interaction) {
            if let (Elem::ValueGlyph(idx, y), Elem::ValueGlyph(_, new_y)) = (elem, new_elem) {
                let ord = new_y.partial_cmp(&y).unwrap_or(Ordering::Equal);
                if ord != Ordering::Equal {
                    let decimal_pos = val_string.chars().position(|ch| ch == '.');
                    let val_f: f64 = NumCast::from(value).unwrap();
                    let min_f: f64 = NumCast::from(min).unwrap();
                    let max_f: f64 = NumCast::from(max).unwrap();
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

        // Call the `react` with the new value if the mouse is pressed/released on the widget or if
        // the value has changed.
        if let Some(react) = maybe_react {
            let should_react = value != new_val
                || match (interaction, new_interaction) {
                    (Highlighted(_), Clicked(_)) | (Clicked(_), Highlighted(_)) => true,
                    _ => false,
                };
            if should_react {
                react(new_val);
            }
        }

        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        if state.view().value != new_val {
            state.update(|state| state.value = new_val);
        }

        if state.view().min != min {
            state.update(|state| state.min = min);
        }

        if state.view().max != max {
            state.update(|state| state.max = max);
        }

        if state.view().precision != precision {
            state.update(|state| state.precision = precision);
        }

        // The **Rectangle** backdrop widget.
        let color = style.color(ui.theme());
        let frame = style.frame(ui.theme());
        let frame_color = style.frame_color(ui.theme());
        let rectangle_idx = state.view().rectangle_idx.get(&mut ui);
        FramedRectangle::new(rect.dim())
            .middle_of(idx)
            .graphics_for(idx)
            .color(color)
            .frame(frame)
            .frame_color(frame_color)
            .set(rectangle_idx, &mut ui);

        // The **Text** for the **NumberDialer**'s label.
        let label_color = style.label_color(ui.theme());
        let font_size = style.label_font_size(ui.theme());
        if maybe_label.is_some() {
            let label_idx = state.view().label_idx.get(&mut ui);
            Text::new(&label_string)
                .x_y_relative_to(idx, label_rel_x, 0.0)
                .graphics_for(idx)
                .color(label_color)
                .font_size(font_size)
                .parent(idx)
                .set(label_idx, &mut ui);
        }

        // Ensure we have at least as many glyph_slot_indices as there are chars in our val_string.
        if state.view().glyph_slot_indices.len() < val_string.chars().count() {
            state.update(|state| {
                let range = state.glyph_slot_indices.len()..val_string.chars().count();
                let extension = range.map(|_| GlyphSlot {
                    rectangle_idx: ui.new_unique_node_index(),
                    text_idx: ui.new_unique_node_index(),
                });
                state.glyph_slot_indices.extend(extension);
            })
        }

        // Instantiate the widgets necessary for each value glyph.
        let slot_w = value_glyph_slot_width(font_size);
        let slot_h = inner_rect.h();
        let val_string_pos = [label_rel_x + label_dim[0] / 2.0, 0.0];
        let mut rel_slot_x = slot_w / 2.0 + val_string_pos[0];
        for (i, _) in val_string.char_indices() {
            let glyph_string = &val_string[i..i+1];
            let slot = state.view().glyph_slot_indices[i];

            // We only want to draw the slot if **Rectangle** if its highlighted or selected.
            let maybe_slot_color = match new_interaction {
                Interaction::Highlighted(elem) => match elem {
                    Elem::ValueGlyph(idx, _) =>
                        if idx == i { Some(color.highlighted()) }
                        else { None },
                        //else { Some(color) },
                    _ => None
                },
                Interaction::Clicked(elem) => match elem {
                    Elem::ValueGlyph(idx, _) =>
                        if idx == i { Some(color.clicked()) }
                        else { None },
                        //else { Some(color) },
                    _ => None,
                },
                _ => None,
            };
            if let Some(slot_color) = maybe_slot_color {
                Rectangle::fill([slot_w, slot_h])
                    .depth(1.0)
                    .x_y_relative_to(idx, rel_slot_x, 0.0)
                    .graphics_for(idx)
                    .color(slot_color)
                    .parent(rectangle_idx)
                    .set(slot.rectangle_idx, &mut ui);
            }

            // Now a **Text** widget for the character itself.
            Text::new(glyph_string)
                .x_y_relative_to(idx, rel_slot_x, 0.0)
                .graphics_for(idx)
                .color(label_color)
                .font_size(font_size)
                .align_text_middle()
                .parent(idx)
                .set(slot.text_idx, &mut ui);

            rel_slot_x += slot_w;
        }
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
