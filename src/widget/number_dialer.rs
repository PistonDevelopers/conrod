use {
    Backend,
    Color,
    Colorable,
    FontSize,
    Frameable,
    FramedRectangle,
    IndexSlot,
    Labelable,
    NodeIndex,
    Point,
    Positionable,
    Rectangle,
    Scalar,
    Sizeable,
    Text,
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
    /// The reaction function for the NumberDialer.
    ///
    /// It will be triggered when the value is updated or if the mouse button is released while the
    /// cursor is above the widget.
    maybe_react: Option<F>,
    style: Style,
    /// If true, will allow user input. If false, will disallow user inputs.
    enabled: bool,
}

/// Unique kind for the widget.
pub const KIND: widget::Kind = "NumberDialer";

widget_style!{
    KIND;
    /// Unique graphical styling for the NumberDialer.
    style Style {
        /// Color of the NumberDialer's rectangle.
        - color: Color { theme.shape_color }
        /// The color of the rectangle frame.
        - frame: Scalar { theme.frame_width }
        /// The color of the rectangle frame.
        - frame_color: Color { theme.frame_color }
        /// The color of the NumberDialer's label.
        - label_color: Color { theme.label_color }
        /// The font size for the NumberDialer's label.
        - label_font_size: FontSize { theme.font_size_medium }
    }
}

/// The state of the NumberDialer.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// The index of the value that is currently pressed.
    pressed_value_idx: Option<usize>,
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

    builder_methods!{
        pub react { maybe_react = Some(F) }
        pub enabled { enabled = bool }
    }

}

impl<'a, T, F> Widget for NumberDialer<'a, T, F>
    where F: FnOnce(T),
          T: Any + ::std::fmt::Debug + Float + NumCast + ToString,
{
    type State = State;
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

    fn init_state(&self) -> Self::State {
        State {
            pressed_value_idx: None,
            rectangle_idx: IndexSlot::new(),
            label_idx: IndexSlot::new(),
            glyph_slot_indices: Vec::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the NumberDialer.
    fn update<B: Backend>(self, args: widget::UpdateArgs<Self, B>) {
        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let NumberDialer {
            value, min, max, precision, maybe_label, maybe_react, ..
        } = self;

        let rel_rect = rect.relative_to(rect.xy());
        let frame = style.frame(ui.theme());
        let inner_rel_rect = rel_rect.pad(frame);
        let font_size = style.label_font_size(ui.theme());
        let label_string = maybe_label.map_or_else(|| String::new(), |text| format!("{}: ", text));
        let label_dim = [ui.glyph_cache().width(font_size, &label_string), font_size as f64];
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

        let value_under_mouse = ui.widget_input(idx).mouse()
            .and_then(|m| value_under_rel_xy(m.rel_xy()));
        let mut pressed_value_idx = state.view().pressed_value_idx;
        let mut new_value = value;

        // Check for the following events:
        // - If a value has been `Press`ed and is being dragged.
        // - `Drag`ging of the mouse while a button is pressed.
        // - `Scroll`ing of the mouse over a value.
        'events: for widget_event in ui.widget_input(idx).events() {
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

        // If the value has changed and some reaction function was given, call it.
        if let Some(react) = maybe_react {
            if value != new_value {
                react(new_value);
            }
        }

        if state.view().pressed_value_idx != pressed_value_idx {
            state.update(|state| state.pressed_value_idx = pressed_value_idx);
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
        let val_string_pos = [label_rel_x + label_dim[0] / 2.0, 0.0];
        let mut rel_slot_x = slot_w / 2.0 + val_string_pos[0];
        for (i, _) in val_string.char_indices() {
            let glyph_string = &val_string[i..i+1];
            let slot = state.view().glyph_slot_indices[i];

            // We only want to draw the slot **Rectangle** if it is highlighted or selected.
            let maybe_slot_color = if Some(i) == pressed_value_idx {
                Some(color.clicked())
            } else if Some(i) == value_under_mouse {
                Some(color.highlighted())
            } else {
                None
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


impl<'a, T, F> Colorable for NumberDialer<'a, T, F> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, T, F> Frameable for NumberDialer<'a, T, F> {
    builder_methods!{
        frame { style.frame = Some(Scalar) }
        frame_color { style.frame_color = Some(Color) }
    }
}

impl<'a, T, F> Labelable<'a> for NumberDialer<'a, T, F> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
