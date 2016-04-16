use {
    Backend,
    CharacterCache,
    Color,
    Colorable,
    Dimensions,
    FontSize,
    Frameable,
    FramedRectangle,
    GlyphCache,
    IndexSlot,
    Line,
    Mouse,
    Padding,
    Point,
    Positionable,
    Range,
    Rectangle,
    Scalar,
    Text,
    Widget,
};
use input::keyboard::Key::{Backspace, Left, Right, Return, A, E, LCtrl, RCtrl};
use utils::vec2_sub;
use widget::{self, KidArea};


pub type Idx = usize;
pub type CursorX = f64;

const TEXT_PADDING: Scalar = 5.0;

/// A widget for displaying and mutating a given one-line text `String`. It's reaction is
/// triggered upon pressing of the `Enter`/`Return` key.
pub struct TextBox<'a, F> {
    common: widget::CommonBuilder,
    text: &'a mut String,
    /// The reaction for the TextBox.
    ///
    /// If `Some`, this will be triggered upon pressing of the `Enter`/`Return` key.
    pub maybe_react: Option<F>,
    style: Style,
    /// Whether or not user input is enabled for the TextBox.
    pub enabled: bool,
}

/// Unique kind for the widget type.
pub const KIND: widget::Kind = "TextBox";

widget_style!{
    KIND;
    /// Unique graphical styling for the TextBox.
    style Style {
        /// Color of the rectangle behind the text. If you don't want to see the rectangle, set the
        /// color with a zeroed alpha.
        - color: Color { theme.shape_color }
        /// The frame around the rectangle behind the text.
        - frame: Scalar { theme.frame_width }
        /// The color of the frame.
        - frame_color: Color { theme.frame_color }
        /// The font size for the text.
        - font_size: FontSize { 24 }
        /// The color of the text.
        - text_color: Color { theme.label_color }
    }
}

/// The State of the TextBox widget that will be cached within the Ui.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    interaction: Interaction,
    rectangle_idx: IndexSlot,
    text_idx: IndexSlot,
    cursor_idx: IndexSlot,
    highlight_idx: IndexSlot,
    control_pressed: bool
}

/// Represents the state of the text_box widget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Interaction {
    Captured(View),
    Uncaptured(Uncaptured),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    cursor: Cursor,
    offset: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cursor {
    anchor: Anchor,
    start: Idx,
    end: Idx,
}

/// The beginning of the cursor's highlighted range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Anchor {
    None,
    Start,
    End,
}

/// The TextBox's state if it is uncaptured.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Uncaptured {
    Highlighted,
    Normal,
}

/// Represents an element of the TextBox widget.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Elem {
    Char(Idx),
    Nill,
    Rect,
}


impl Interaction {
    /// Return the color associated with the current state.
    fn color(&self, color: Color) -> Color {
        match *self {
            Interaction::Captured(_) => color,
            Interaction::Uncaptured(state) => match state {
                Uncaptured::Highlighted => color.highlighted(),
                Uncaptured::Normal => color,
            }
        }
    }
}


impl Cursor {

    /// Construct a Cursor from a String index.
    fn from_index(idx: Idx) -> Cursor {
        Cursor { anchor: Anchor::Start, start: idx, end: idx }
    }

    /// Construct a Cursor from an index range.
    fn from_range(start: Idx, end: Idx) -> Cursor {
        if start < end {
            Cursor { anchor: Anchor::Start, start: start, end: end }
        } else {
            Cursor { anchor: Anchor::End, start: end, end: start }
        }
    }

    /// Ensure the cursor does not go past end_limit. Returns true if the cursor was
    /// previously too large.
    fn limit_end_to(&mut self, end_limit: usize) -> bool {
        if self.start > end_limit {
            self.start = end_limit;
            self.end = end_limit;
            return true;
        }
        else if self.end > end_limit {
            self.end = end_limit;
            return true
        }
        false
    }

    /// Shift the curosr by the given number of indices.
    fn shift(&mut self, by: i32) {
        if by == 0 {
             return;
        }
        let mut new_idx = self.start as i32 + by;
        if new_idx < 0 {
            new_idx = 0;
        }
        self.start = new_idx as Idx;
        self.end = new_idx as Idx;
    }

    fn is_cursor(&self) -> bool {
        self.start == self.end
    }
}

// widget_fns!(TextBox, State, Kind::TextBox(State::Uncaptured(Uncaptured::Normal)));

/// Find the position of a character in a text box.
fn cursor_position<C: CharacterCache>(glyph_cache: &GlyphCache<C>,
                                      idx: usize,
                                      mut text_start_x: f64,
                                      font_size: FontSize,
                                      text: &str) -> CursorX {
    assert!(idx <= text.chars().count());

    if idx == 0 {
         return text_start_x;
    }

    for (i, ch) in text.chars().enumerate() {
        if i >= idx { break; }
        text_start_x += glyph_cache.char_width(font_size, ch);
    }
    text_start_x
}

/// Check if cursor is over the pad and if so, which
fn over_elem<C: CharacterCache>(glyph_cache: &GlyphCache<C>,
                                mouse_xy: Point,
                                dim: Dimensions,
                                pad_dim: Dimensions,
                                text_start_x: f64,
                                text_w: f64,
                                font_size: FontSize,
                                text: &str) -> Elem {
    use position::is_over_rect;
    if is_over_rect([0.0, 0.0], dim, mouse_xy) {
        if is_over_rect([0.0, 0.0], pad_dim, mouse_xy) {
            let (idx, _) = closest_idx(glyph_cache, mouse_xy, text_start_x, text_w, font_size, text);
            Elem::Char(idx)
        } else {
            Elem::Rect
        }
    } else {
        Elem::Nill
    }
}

/// Check which character is closest to the mouse cursor.
fn closest_idx<C: CharacterCache>(glyph_cache: &GlyphCache<C>,
                                  mouse_xy: Point,
                                  text_start_x: f64,
                                  text_w: f64,
                                  font_size: FontSize,
                                  text: &str) -> (Idx, f64) {
    if mouse_xy[0] <= text_start_x { return (0, text_start_x) }
    let mut left_x = text_start_x;
    let mut x = left_x;
    let mut prev_x = x;
    for (i, ch) in text.chars().enumerate() {
        let char_w = glyph_cache.char_width(font_size, ch);
        x += char_w;
        let right_x = prev_x + char_w / 2.0;
        if mouse_xy[0] > left_x && mouse_xy[0] <= right_x { return (i, prev_x) }
        prev_x = x;
        left_x = right_x;
    }
    (text.chars().count(), text_start_x + text_w)
}

/// Check and return the current state of the TextBox.
fn get_new_interaction(over_elem: Elem, prev_interaction: Interaction, mouse: Mouse) -> Interaction {
    use mouse::ButtonPosition::{Down, Up};
    use self::Interaction::{Captured, Uncaptured};
    use self::Uncaptured::{Normal, Highlighted};

    match prev_interaction {
        Interaction::Captured(mut prev) => match mouse.left.position {
            Down => match over_elem {
                Elem::Nill => if prev.cursor.anchor == Anchor::None {
                    Uncaptured(Normal)
                } else {
                    prev_interaction
                },
                Elem::Rect =>  if prev.cursor.anchor == Anchor::None {
                    prev.cursor = Cursor::from_index(0);
                    Captured(prev)
                } else {
                    prev_interaction
                },
                Elem::Char(idx) => {
                    match prev.cursor.anchor {
                        Anchor::None  => prev.cursor = Cursor::from_index(idx),
                        Anchor::Start => prev.cursor = Cursor::from_range(prev.cursor.start, idx),
                        Anchor::End   => prev.cursor = Cursor::from_range(prev.cursor.end, idx),
                    }
                    Captured(prev)
                },
            },
            Up => {
                prev.cursor.anchor = Anchor::None;
                Captured(prev)
            },
        },

        Interaction::Uncaptured(prev) => match mouse.left.position {
            Down => match over_elem {
                Elem::Nill => Uncaptured(Normal),
                Elem::Rect => match prev {
                    Normal => prev_interaction,
                    Highlighted => Captured(View {
                         cursor: Cursor::from_index(0),
                         offset: 0.0,
                    })
                },
                Elem::Char(idx) =>  match prev {
                    Normal => prev_interaction,
                    Highlighted => Captured(View {
                        cursor: Cursor::from_index(idx),
                        offset: 0.0,
                    })
                },
            },
            Up => match over_elem {
                Elem::Nill => Uncaptured(Normal),
                Elem::Char(_) | Elem::Rect => match prev {
                    Normal => Uncaptured(Highlighted),
                    Highlighted => prev_interaction,
                },
            },
        },
    }
}

impl<'a, F> TextBox<'a, F> {

    /// Construct a TextBox widget.
    pub fn new(text: &'a mut String) -> TextBox<'a, F> {
        TextBox {
            common: widget::CommonBuilder::new(),
            text: text,
            maybe_react: None,
            style: Style::new(),
            enabled: true,
        }
    }

    builder_methods!{
        pub font_size { style.font_size = Some(FontSize) }
        pub react { maybe_react = Some(F) }
        pub enabled { enabled = bool }
    }

}

impl<'a, F> Widget for TextBox<'a, F>
    where F: FnMut(&mut String),
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

    fn init_state(&self) -> State {
        State {
            interaction: Interaction::Uncaptured(Uncaptured::Normal),
            rectangle_idx: IndexSlot::new(),
            text_idx: IndexSlot::new(),
            cursor_idx: IndexSlot::new(),
            highlight_idx: IndexSlot::new(),
            control_pressed: false,
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn kid_area<C: CharacterCache>(&self, args: widget::KidAreaArgs<Self, C>) -> widget::KidArea {
        KidArea {
            rect: args.rect,
            pad: Padding {
                x: Range::new(TEXT_PADDING, TEXT_PADDING),
                y: Range::new(TEXT_PADDING, TEXT_PADDING),
            },
        }
    }

    /// Update the state of the TextBox.
    fn update<B: Backend>(mut self, args: widget::UpdateArgs<Self, B>) {
        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;

        let (xy, dim) = rect.xy_dim();
        let maybe_mouse = ui.input(idx).maybe_mouse.map(|mouse| mouse.relative_to(xy));
        let frame = style.frame(ui.theme());
        let inner_rect = rect.pad(frame);
        let font_size = style.font_size(ui.theme());
        let pad_dim = vec2_sub(dim, [frame * 2.0; 2]);
        let text_w = ui.glyph_cache().width(font_size, &self.text);
        let text_x = text_w / 2.0 - pad_dim[0] / 2.0 + TEXT_PADDING;
        let text_start_x = text_x - text_w / 2.0;
        let mut new_control_pressed = state.view().control_pressed;
        let mut new_interaction = match (self.enabled, maybe_mouse) {
            (false, _) | (true, None) => Interaction::Uncaptured(Uncaptured::Normal),
            (true, Some(mouse)) => {
                let over_elem = over_elem(ui.glyph_cache(), mouse.xy, dim, pad_dim, text_start_x,
                                          text_w, font_size, &self.text);
                get_new_interaction(over_elem, state.view().interaction, mouse)
            },
        };

        // Check cursor validity (and update new_interaction if necessary).
        if let Interaction::Captured(view) = new_interaction {
            let mut cursor = view.cursor;
            let mut v_offset = view.offset;

            // Ensure the cursor is still valid.
            cursor.limit_end_to(self.text.chars().count());

            // This matters if the text is scrolled with the mouse.
            let cursor_idx = match cursor.anchor {
                Anchor::End => cursor.start,
                Anchor::Start | Anchor::None => cursor.end,
            };
            let cursor_x = cursor_position(ui.glyph_cache(), cursor_idx, text_start_x, font_size,
                                           &self.text);

            if cursor.is_cursor() || cursor.anchor != Anchor::None {
                let cursor_x_view = cursor_x - v_offset;
                let text_right = dim[0] - TEXT_PADDING - frame;

                if cursor_x_view < text_x {
                    v_offset += cursor_x_view - text_x;
                } else if cursor_x_view > text_right {
                    v_offset += cursor_x_view - text_right;
                }
            }

            // Set the updated state.
            new_interaction = Interaction::Captured(View { cursor: cursor, offset: v_offset });
        }

        // If TextBox is captured, check for recent input and update the text accordingly.
        if let Interaction::Captured(captured) = new_interaction {
            let mut cursor = captured.cursor;
            let input = ui.input(idx);

            // Check for entered text.
            for text in input.entered_text {
                if new_control_pressed { break; }
                if text.chars().count() == 0 { continue; }

                let max_w = pad_dim[0] - TEXT_PADDING * 2.0;
                if text_w + ui.glyph_cache().width(font_size, &text) > max_w { continue; }

                let end: String = self.text.chars().skip(cursor.end).collect();
                let start: String = self.text.chars().take(cursor.start).collect();
                self.text.clear();
                self.text.push_str(&start);
                self.text.push_str(&text);
                self.text.push_str(&end);
                cursor.shift(text.chars().count() as i32);
            }

            // Check for control keys.
            for key in input.pressed_keys.iter() {
                match *key {
                    Backspace => if cursor.is_cursor() {
                        if cursor.start > 0 {
                            let start: String = self.text.chars().take(cursor.start-1).collect();
                            let end: String = self.text.chars().skip(cursor.end).collect();
                            self.text.clear();
                            self.text.push_str(&start);
                            self.text.push_str(&end);
                            cursor.shift(-1);
                        }
                    } else {
                        let start: String = self.text.chars().take(cursor.start).collect();
                        let end: String = self.text.chars().skip(cursor.end).collect();
                        self.text.clear();
                        self.text.push_str(&start);
                        self.text.push_str(&end);
                        cursor.end = cursor.start;
                    },
                    Left => if cursor.is_cursor() {
                        cursor.shift(-1);
                    },
                    Right => if cursor.is_cursor() && self.text.chars().count() > cursor.end {
                        cursor.shift(1);
                    },
                    Return => if self.text.chars().count() > 0 {
                        let TextBox { ref mut maybe_react, ref mut text, .. } = self;
                        if let Some(ref mut react) = *maybe_react {
                            react(*text);
                        }
                    },
                    LCtrl | RCtrl if !new_control_pressed => {
                        new_control_pressed = true;
                    },
                    A if new_control_pressed => {
                        if cursor.is_cursor() {
                            cursor.start = 0;
                            cursor.end = self.text.chars().count();
                        }
                    },
                    E if new_control_pressed => {
                        if cursor.is_cursor() {
                            cursor.start = self.text.chars().count();
                            cursor.end = self.text.chars().count();
                        }
                    },
                    _ => (),
                }
            }

            for key in input.released_keys.iter() {
                match *key {
                    LCtrl | RCtrl if new_control_pressed => {
                        new_control_pressed = false;
                    },
                    _ => (),
                }
            }

            // In case the string text was mutated with the `react` function, we need to make sure
            // the cursor is limited to the current number of chars.
            cursor.limit_end_to(self.text.chars().count());

            new_interaction = Interaction::Captured(View { cursor: cursor, .. captured });
        }

        // Check the interactions to determine whether we need to capture or uncapture the keyboard.
        match (state.view().interaction, new_interaction) {
            (Interaction::Uncaptured(_), Interaction::Captured(_)) => { ui.capture_keyboard(idx); },
            (Interaction::Captured(_), Interaction::Uncaptured(_)) => { ui.uncapture_keyboard(idx); },
            _ => (),
        }

        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        if state.view().control_pressed != new_control_pressed {
            state.update(|state| state.control_pressed = new_control_pressed);
        }

        let rectangle_idx = state.view().rectangle_idx.get(&mut ui);
        let frame = style.frame(ui.theme());
        let color = new_interaction.color(style.color(ui.theme()));
        let frame_color = style.frame_color(ui.theme());
        FramedRectangle::new(rect.dim())
            .middle_of(idx)
            .graphics_for(idx)
            .color(color)
            .frame(frame)
            .frame_color(frame_color)
            .set(rectangle_idx, &mut ui);

        let text_color = style.text_color(ui.theme());
        let font_size = style.font_size(ui.theme());
        let text_idx = state.view().text_idx.get(&mut ui);
        Text::new(&self.text)
            .mid_left_of(idx)
            .graphics_for(idx)
            .color(text_color)
            .font_size(font_size)
            .no_line_wrap()
            .set(text_idx, &mut ui);


        if let Interaction::Captured(view) = new_interaction {
            let cursor = view.cursor;
            // This matters if the text is scrolled with the mouse.
            let cursor_idx = match cursor.anchor {
                Anchor::End => cursor.start,
                Anchor::Start | Anchor::None => cursor.end,
            };
            let cursor_x = cursor_position(ui.glyph_cache(), cursor_idx, text_start_x, font_size,
                                           &self.text);

            if cursor.is_cursor() {
                let cursor_idx = state.view().cursor_idx.get(&mut ui);
                let start = [0.0, 0.0];
                let end = [0.0, inner_rect.h()];
                Line::centred(start, end)
                    .x_relative_to(idx, cursor_x)
                    .graphics_for(idx)
                    .parent(idx)
                    .color(text_color)
                    .set(cursor_idx, &mut ui);
            } else {
                let (rel_x, w) = {
                    let (start, end) = (cursor.start, cursor.end);
                    let cursor_x = cursor_position(ui.glyph_cache(), start, text_start_x, font_size,
                                                   &self.text);
                    let highlighted_text = &self.text[start..end];
                    let w = ui.glyph_cache().width(font_size, &highlighted_text);
                    (cursor_x + w / 2.0, w)
                };
                let dim = [w, inner_rect.h()];
                let highlight_idx = state.view().highlight_idx.get(&mut ui);
                Rectangle::fill(dim)
                    .x_relative_to(idx, rel_x)
                    .color(text_color.highlighted().alpha(0.25))
                    .graphics_for(idx)
                    .parent(idx)
                    .set(highlight_idx, &mut ui);
            }
        }
    }

}


impl<'a, F> Colorable for TextBox<'a, F> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, F> Frameable for TextBox<'a, F> {
    builder_methods!{
        frame { style.frame = Some(Scalar) }
        frame_color { style.frame_color = Some(Color) }
    }
}
