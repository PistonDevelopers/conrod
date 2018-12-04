//! A widget for precision control over any base-10 digit within a given value.

use {
    Color,
    Colorable,
    FontSize,
    Borderable,
    Labelable,
    Point,
    Positionable,
    Scalar,
    Widget,
};
use num::{Float, NumCast};
use std::cmp::Ordering;
use std::iter::repeat;
use text;
use utils::clamp;
use widget;


/// A widget for precision control over any digit within a value.
///
/// The reaction is triggered when the value is updated or if the mouse button is released while
/// the cursor is above the widget.
#[derive(WidgetCommon_)]
pub struct NumberDialer<'a, T> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    value: T,
    min: T,
    max: T,
    maybe_label: Option<&'a str>,
    precision: u8,
    style: Style,
    /// If true, will allow user input. If false, will disallow user inputs.
    enabled: bool,
}

/// Unique graphical styling for the NumberDialer.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// Color of the NumberDialer's rectangle.
    #[conrod(default = "theme.shape_color")]
    pub color: Option<Color>,
    /// The color of the rectangle border.
    #[conrod(default = "theme.border_width")]
    pub border: Option<Scalar>,
    /// The color of the rectangle border.
    #[conrod(default = "theme.border_color")]
    pub border_color: Option<Color>,
    /// The color of the NumberDialer's label.
    #[conrod(default = "theme.label_color")]
    pub label_color: Option<Color>,
    /// The font size for the NumberDialer's label.
    #[conrod(default = "theme.font_size_medium")]
    pub label_font_size: Option<FontSize>,
    /// The `Id` associated with the font to use for the `NumberDialer` values.
    #[conrod(default = "theme.font_id")]
    pub font_id: Option<Option<text::font::Id>>,
}

widget_ids! {
    struct Ids {
        rectangle,
        label,
    }
}

/// The state of the NumberDialer.
pub struct State {
    /// The index of the value that is currently pressed.
    pressed_value_idx: Option<usize>,
    ids: Ids,
    glyph_slot_indices: Vec<GlyphSlot>,
}

/// Each digit in the adjustable value has its own **Rectangle** and **Text** widgets.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GlyphSlot {
    /// The highlightable **Rectangle** behind the glyph.
    rectangle_id: widget::Id,
    /// The **Text** widget for the glyph itself.
    text_id: widget::Id,
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


impl<'a, T> NumberDialer<'a, T>
    where T: Float,
{
    /// Construct a new NumberDialer widget.
    pub fn new(value: T, min: T, max: T, precision: u8) -> Self {
        NumberDialer {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            value: clamp(value, min, max),
            min: min,
            max: max,
            precision: precision,
            maybe_label: None,
            enabled: true,
        }
    }

    /// Specify the font used for displaying the label.
    pub fn font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.font_id = Some(Some(font_id));
        self
    }

    builder_methods!{
        pub enabled { enabled = bool }
    }
}

impl<'a, T> Widget for NumberDialer<'a, T>
    where T: Float + NumCast + ToString,
{
    type State = State;
    type Style = Style;
    type Event = Option<T>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            pressed_value_idx: None,
            ids: Ids::new(id_gen),
            glyph_slot_indices: Vec::new(),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the NumberDialer.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, rect, style, mut ui, .. } = args;
        let NumberDialer { value, min, max, precision, maybe_label, .. } = self;

        let rel_rect = rect.relative_to(rect.xy());
        let border = style.border(ui.theme());
        let inner_rel_rect = rel_rect.pad(border);

        // Retrieve the `font_id`, as long as a valid `Font` for it still exists.
        //
        // If we've no font to use for text logic, bail out without updating.
        let font_id = match style.font_id(&ui.theme).or(ui.fonts.ids().next()) {
            Some(font_id) => font_id,
            None => return None,
        };

        let font_size = style.label_font_size(ui.theme());
        let label_string = maybe_label.map_or_else(|| String::new(), |text| format!("{}: ", text));
        let label_w = {
            let font = ui.fonts.get(font_id).unwrap();
            text::line::width(&label_string, font, font_size)
        };
        let label_dim = [label_w, font_size as f64];
        let precision_len = if precision == 0 { 0 } else { precision as usize + 1 };
        let val_string_len = max.to_string().len() + precision_len;
        let val_string = create_val_string(value, val_string_len, precision);
        let val_string_dim = [val_string_width(font_size, &val_string), font_size as f64];
        let label_rel_x = -val_string_dim[0] / 2.0;
        let slot_w = value_glyph_slot_width(val_string_dim[1] as u32);
        let slot_h = inner_rel_rect.h();

        // Determine if the cursor is over one of the NumberDialer's values.
        //
        // Returns the index into the `val_string` for retrieving the value.
        let value_under_rel_xy = |rel_xy: Point| -> Option<usize> {
            if rel_rect.is_over(rel_xy) {
                use position::Rect;
                let slot_rect_xy = [label_rel_x + label_dim[0] / 2.0 + slot_w / 2.0, 0.0];
                let val_string_xy = [slot_rect_xy[0] - slot_w / 2.0 + val_string_dim[0] / 2.0, 0.0];
                let val_string_dim = [val_string_dim[0], slot_h];
                let val_string_rect = Rect::from_xy_dim(val_string_xy, val_string_dim);
                if val_string_rect.is_over(rel_xy) {
                    let mut slot_xy = slot_rect_xy;
                    for i in 0..val_string_len {
                        let slot_rect = Rect::from_xy_dim(slot_xy, [slot_w, slot_h]);
                        if slot_rect.is_over(rel_xy) {
                            return Some(i);
                        }
                        slot_xy[0] += slot_w;
                    }
                }
            }
            None
        };

        let value_under_mouse = ui.widget_input(id).mouse()
            .and_then(|m| value_under_rel_xy(m.rel_xy()));
        let mut pressed_value_idx = state.pressed_value_idx;
        let mut new_value = value;

        // Check for the following events:
        // - If a value has been `Press`ed and is being dragged.
        // - `Drag`ging of the mouse while a button is pressed.
        // - `Scroll`ing of the mouse over a value.
        'events: for widget_event in ui.widget_input(id).events() {
            use event;
            use input::{self, MouseButton};

            match widget_event {

                // Check to see if a value was pressed in case it is later dragged.
                event::Widget::Press(press) => {
                    if let event::Button::Mouse(MouseButton::Left, _) = press.button {
                        pressed_value_idx = value_under_mouse;
                    }
                },

                // Check to see if a value was released in case it is later dragged.
                event::Widget::Release(release) => {
                    if let event::Button::Mouse(MouseButton::Left, _) = release.button {
                        pressed_value_idx = None;
                    }
                },

                // A left `Drag` moves the `pressed_point` if there is one.
                event::Widget::Drag(drag) if drag.button == input::MouseButton::Left => {
                    if let Some(idx) = pressed_value_idx {
                        let decimal_pos = val_string.chars().position(|ch| ch == '.');
                        let val_f: f64 = NumCast::from(value).unwrap();
                        let min_f: f64 = NumCast::from(min).unwrap();
                        let max_f: f64 = NumCast::from(max).unwrap();
                        let ord = drag.delta_xy[1].partial_cmp(&0.0).unwrap_or(Ordering::Equal);
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
                        new_value = NumCast::from(new_val_f).unwrap();
                    }
                },

                _ => (),
            }
        }

        // If the value has changed produce an event.
        let event = if value != new_value { Some(new_value) } else { None };

        if state.pressed_value_idx != pressed_value_idx {
            state.update(|state| state.pressed_value_idx = pressed_value_idx);
        }

        // The **Rectangle** backdrop widget.
        let color = style.color(ui.theme());
        let border = style.border(ui.theme());
        let border_color = style.border_color(ui.theme());
        widget::BorderedRectangle::new(rect.dim())
            .middle_of(id)
            .graphics_for(id)
            .color(color)
            .border(border)
            .border_color(border_color)
            .set(state.ids.rectangle, &mut ui);

        // The **Text** for the **NumberDialer**'s label.
        let label_color = style.label_color(ui.theme());
        let font_size = style.label_font_size(ui.theme());
        if maybe_label.is_some() {
            widget::Text::new(&label_string)
                .font_id(font_id)
                .x_y_relative_to(id, label_rel_x, 0.0)
                .graphics_for(id)
                .color(label_color)
                .font_size(font_size)
                .parent(id)
                .set(state.ids.label, &mut ui);
        }

        // Ensure we have at least as many glyph_slot_indices as there are chars in our val_string.
        if state.glyph_slot_indices.len() < val_string.chars().count() {
            state.update(|state| {
                let range = state.glyph_slot_indices.len()..val_string.chars().count();
                let mut id_gen = ui.widget_id_generator();
                let extension = range.map(|_| GlyphSlot {
                    rectangle_id: id_gen.next(),
                    text_id: id_gen.next(),
                });
                state.glyph_slot_indices.extend(extension);
            })
        }

        // Instantiate the widgets necessary for each value glyph.
        let val_string_pos = [label_rel_x + label_dim[0] / 2.0, 0.0];
        let mut rel_slot_x = slot_w / 2.0 + val_string_pos[0];
        for (i, _) in val_string.char_indices() {
            let glyph_string = &val_string[i..i+1];
            let slot = state.glyph_slot_indices[i];

            // We only want to draw the slot **Rectangle** if it is highlighted or selected.
            let maybe_slot_color = if Some(i) == pressed_value_idx {
                Some(color.clicked())
            } else if Some(i) == value_under_mouse {
                Some(color.highlighted())
            } else {
                None
            };

            if let Some(slot_color) = maybe_slot_color {
                widget::Rectangle::fill([slot_w, slot_h])
                    .x_y_relative_to(id, rel_slot_x, 0.0)
                    .graphics_for(id)
                    .color(slot_color)
                    .parent(state.ids.rectangle)
                    .set(slot.rectangle_id, &mut ui);
            }

            // Now a **Text** widget for the character itself.
            widget::Text::new(glyph_string)
                .font_id(font_id)
                .x_y_relative_to(id, rel_slot_x, 0.0)
                .graphics_for(id)
                .color(label_color)
                .font_size(font_size)
                .center_justify()
                .parent(id)
                .set(slot.text_id, &mut ui);

            rel_slot_x += slot_w;
        }

        event
    }

}


impl<'a, T> Colorable for NumberDialer<'a, T> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, T> Borderable for NumberDialer<'a, T> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a, T> Labelable<'a> for NumberDialer<'a, T> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
