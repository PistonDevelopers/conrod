use {
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
    Padding,
    Point,
    Positionable,
    Range,
    Rectangle,
    Scalar,
    Text,
    Theme,
};

use input::keyboard::Key::{Backspace, Left, Right, Return, A, E, LCtrl, RCtrl};
use vecmath::vec2_sub;
use widget::{self, Widget, KidArea, UpdateArgs};


pub type Idx = usize;
pub type CursorX = f64;

const TEXT_PADDING: Scalar = 5.0;

/// A widget for displaying and mutating a given one-line text `String`. It's reaction is
/// triggered upon pressing of the `Enter`/`Return` key.
pub struct TextBox<'a, F> {
    common: widget::CommonBuilder,
    text: &'a mut String,
    maybe_react: Option<F>,
    style: Style,
    enabled: bool,
}

/// Styling for the TextBox, necessary for constructing its renderable Element.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    /// Color of the rectangle behind the text. If you don't want to see the rectangle, set the
    /// color with a zeroed alpha.
    pub maybe_color: Option<Color>,
    /// The frame around the rectangle behind the text.
    pub maybe_frame: Option<Scalar>,
    /// The color of the frame.
    pub maybe_frame_color: Option<Color>,
    /// The font size for the text.
    pub maybe_font_size: Option<u32>,
    /// The color of the text.
    pub maybe_text_color: Option<Color>,
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

/// Unique kind for the widget type.
pub const KIND: widget::Kind = "TextBox";

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

    /// Set the font size of the text.
    pub fn font_size(mut self, font_size: FontSize) -> TextBox<'a, F> {
        self.style.maybe_font_size = Some(font_size);
        self
    }

    /// Set the reaction for the TextBox. It will be triggered upon pressing of the
    /// `Enter`/`Return` key.
    pub fn react(mut self, reaction: F) -> TextBox<'a, F> {
        self.maybe_react = Some(reaction);
        self
    }

    /// If true, will allow user inputs.  If false, will disallow user inputs.
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
    }

}

fn get_clicked_elem<C, F>(text: &str, point: Point, args: &UpdateArgs<TextBox<F>, C>) -> Elem
                                                                        where C: CharacterCache,
                                                                        F: FnMut(&mut String) {
    let style = args.style;
    let theme = args.ui.theme();
    let font_size = style.font_size(theme);
    let (_xy, widget_dimension) = args.rect.xy_dim();
    let inner_dimension = get_inner_dimensions(widget_dimension, style, theme);
    let glyph_cache = args.ui.glyph_cache();
    let text_w = glyph_cache.width(font_size, text);
    let text_x = text_w / 2.0 - inner_dimension[0] / 2.0 + TEXT_PADDING;
    let text_start_x = text_x - text_w / 2.0;
    over_elem(args.ui.glyph_cache(),
        point,
        widget_dimension,
        inner_dimension,
        text_start_x,
        text_w,
        font_size,
        text)
}

fn get_elem_for_drag<C, F>(text: &str, point: Point, args: &UpdateArgs<TextBox<F>, C>) -> Elem
                                                                        where C: CharacterCache,
                                                                        F: FnMut(&mut String) {
    let style = args.style;
    let theme = args.ui.theme();
    let font_size = style.font_size(theme);
    let (_xy, widget_dimension) = args.rect.xy_dim();
    let inner_dimension = get_inner_dimensions(widget_dimension, style, theme);
    let glyph_cache = args.ui.glyph_cache();
    let text_w = glyph_cache.width(font_size, text);
    let text_x = text_w / 2.0 - inner_dimension[0] / 2.0 + TEXT_PADDING;
    let text_start_x = text_x - text_w / 2.0;
    let (cursor_index, _) = closest_idx(args.ui.glyph_cache(),
                                        point,
                                        text_start_x,
                                        text_w,
                                        font_size,
                                        text);
    Elem::Char(cursor_index)
}

fn get_interaction_for_click(text: &str, clicked_elem: Elem) -> Interaction {
    match clicked_elem {
        Elem::Char(idx) => Interaction::Captured(View{
            cursor: Cursor::from_index(idx),
            offset: 0f64
        }),
        Elem::Rect => Interaction::Captured(View{
            cursor: Cursor::from_range(0, text.chars().count()),
            offset: 0f64
        }),
        Elem::Nill => Interaction::Uncaptured(Uncaptured::Normal)
    }
}

fn get_interaction_for_drag(start_elem: Elem, end_elem: Elem) -> Interaction {
    if let (Elem::Char(start_idx), Elem::Char(end_idx)) = (start_elem, end_elem) {
        Interaction::Captured(View{
            cursor: Cursor::from_range(start_idx, end_idx),
            offset: 0f64
        })
    } else {
        Interaction::Captured(View{
            cursor: Cursor::from_index(0),
            offset: 0f64
        })
    }
}

fn get_highlight_all_interaction(text: &str) -> Interaction {
    Interaction::Captured(View{
        cursor: Cursor::from_range(0, text.chars().count()),
        offset: 0.0
    })
}

fn get_new_interaction<C, F>(text: &str, args: &UpdateArgs<TextBox<F>, C>) -> Interaction
                                                            where C: CharacterCache,
                                                            F: FnMut(&mut String) {
    use ::mouse::MouseButton;

    let maybe_mouse = args.ui.input().maybe_mouse;
    maybe_mouse.map(|m| m.relative_to(args.rect.xy())).iter()
        .flat_map(|mouse| mouse.events())
        // map MouseEvent to an Option<Interaction>
        .filter_map(|event| {
            use ::mouse::events::MouseEvent::{Click, Down, Drag};
            match event {
                Down(down_info) if down_info.mouse_button == MouseButton::Left => {
                    let clicked_elem = get_clicked_elem(text, down_info.position, args);
                    Some(get_interaction_for_click(text, clicked_elem))
                },
                Click(click_info) if click_info.mouse_button == MouseButton::Left => {
                    let clicked_elem = get_clicked_elem(text, click_info.position, args);
                    Some(get_interaction_for_click(text, clicked_elem))
                },
                Drag(drag_info) if drag_info.mouse_button == MouseButton::Left => {
                    let drag_start = get_elem_for_drag(text, drag_info.start.position, args);
                    let drag_end = get_elem_for_drag(text, drag_info.current.position, args);
                    Some(get_interaction_for_drag(drag_start, drag_end))
                },
                _ => None
            }
        }).next()
        .unwrap_or_else(|| {
            // If there was no interaction from a new mouse event, then we check previous interaction
            let prev_interaction = args.state.view().interaction;
            let is_capturing_keyboard = args.ui.is_capturing_keyboard();
            match prev_interaction {
                Interaction::Captured(_) if is_capturing_keyboard => prev_interaction,
                _ if is_capturing_keyboard && maybe_mouse.is_none() =>
                        get_highlight_all_interaction(text),
                _ if maybe_mouse.is_some() => Interaction::Uncaptured(Uncaptured::Highlighted),
                _ => Interaction::Uncaptured(Uncaptured::Normal)
            }
        })
}

fn get_inner_dimensions(outer_rect: Dimensions, style: &Style, theme: &Theme) -> Dimensions {
    let frame_width = style.frame(theme);
    vec2_sub(outer_rect, [frame_width * 2.0; 2])
}


impl<'a, F> Widget for TextBox<'a, F> where F: FnMut(&mut String) {
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
    fn update<C: CharacterCache>(mut self, args: widget::UpdateArgs<Self, C>) {
        let mut new_interaction = get_new_interaction(&self.text, &args);
        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;

        let dim = rect.dim();
        let frame = style.frame(ui.theme());
        let inner_rect = rect.pad(frame);
        let font_size = style.font_size(ui.theme());
        let pad_dim = vec2_sub(dim, [frame * 2.0; 2]);
        let text_w = ui.glyph_cache().width(font_size, &self.text);
        let text_x = text_w / 2.0 - pad_dim[0] / 2.0 + TEXT_PADDING;
        let text_start_x = text_x - text_w / 2.0;
        let mut new_control_pressed = state.view().control_pressed;

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
            let input = ui.input();

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

impl Style {

    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_font_size: None,
            maybe_text_color: None,
        }
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or_else(|| theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or_else(|| theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or_else(|| theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label font size for an Element.
    pub fn font_size(&self, theme: &Theme) -> FontSize {
        const DEFAULT_FONT_SIZE: u32 = 24;
        self.maybe_font_size.or_else(|| theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_font_size.unwrap_or(DEFAULT_FONT_SIZE)
        })).unwrap_or(DEFAULT_FONT_SIZE)
    }

    /// Get the Color for of the Text.
    pub fn text_color(&self, theme: &Theme) -> Color {
        self.maybe_text_color.or_else(|| theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_text_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

}

impl<'a, F> Colorable for TextBox<'a, F> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

impl<'a, F> Frameable for TextBox<'a, F> {
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
        self
    }
}
