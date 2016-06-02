use {
    Align,
    Backend,
    CharacterCache,
    Color,
    Colorable,
    FontSize,
    GlyphCache,
    IndexSlot,
    Line,
    NodeIndex,
    Point,
    Positionable,
    Range,
    Rect,
    Rectangle,
    Scalar,
    Sizeable,
    Text,
    Widget,
};
use event;
use input;
use std;
use text;
use utils;
use widget;
use widget::primitive::text::Wrap;


/// A widget for displaying and mutating multi-line text, given as a `String`.
///
/// By default the text is wrapped via the first whitespace before the line exceeds the
/// `TextEdit`'s width, however a user may change this using the `.wrap_by_character` method.
pub struct TextEdit<'a, F> {
    common: widget::CommonBuilder,
    text: &'a mut String,
    /// The reaction for the TextEdit.
    pub maybe_react: Option<F>,
    style: Style,
}

/// Unique kind for the widget type.
pub const KIND: widget::Kind = "TextEdit";

widget_style!{
    KIND;
    /// Unique graphical styling for the TextEdit.
    style Style {
        /// The color of the text (this includes cursor and selection color).
        - color: Color { theme.shape_color }
        /// The font size for the text.
        - font_size: FontSize { theme.font_size_medium }
        /// The horizontal alignment of the text.
        - x_align: Align { Align::Start }
        /// The vertical alignment of the text.
        - y_align: Align { Align::End }
        /// The vertical space between each line of text.
        - line_spacing: Scalar { 1.0 }
        /// The way in which text is wrapped at the end of a line.
        - line_wrap: Wrap { Wrap::Whitespace }
        /// Do not allow to enter text that would exceed the bounds of the `TextEdit`'s `Rect`.
        - restrict_to_height: bool { true }
    }
}

/// The State of the TextEdit widget that will be cached within the Ui.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    cursor: Cursor,
    /// Track whether some sort of dragging is currently occurring.
    drag: Option<Drag>,
    /// Information about each line of text.
    line_infos: Vec<text::line::Info>,
    selected_rectangle_indices: Vec<NodeIndex>,
    rectangle_idx: IndexSlot,
    text_idx: IndexSlot,
    cursor_idx: IndexSlot,
    highlight_idx: IndexSlot,
}

/// Track whether some sort of dragging is currently occurring.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Drag {
    Selecting,
    #[allow(dead_code)] // TODO: Implement this.
    MoveSelection,
}

/// The position of the `Cursor` over the text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cursor {
    /// The cursor is at the given character index.
    Idx(text::cursor::Index),
    /// The cursor is a selection between these two indices.
    ///
    /// The `start` is always the "anchor" point.
    ///
    /// The `end` may be either greater or less than the `start`.
    Selection {
        start: text::cursor::Index,
        end: text::cursor::Index,
    },
}


impl<'a, F> TextEdit<'a, F> {

    /// Construct a TextEdit widget.
    pub fn new(text: &'a mut String) -> Self {
        TextEdit {
            common: widget::CommonBuilder::new(),
            text: text,
            maybe_react: None,
            style: Style::new(),
        }
    }

    /// The `TextEdit` will wrap text via the whitespace that precedes the first width-exceeding
    /// character.
    ///
    /// This is the default setting.
    pub fn wrap_by_whitespace(self) -> Self {
        self.line_wrap(Wrap::Whitespace)
    }

    /// By default, the `TextEdit` will wrap text via the whitespace that precedes the first
    /// width-exceeding character.
    ///
    /// Calling this method causes the `TextEdit` to wrap text at the first exceeding character.
    pub fn wrap_by_character(self) -> Self {
        self.line_wrap(Wrap::Character)
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn align_text_left(self) -> Self {
        self.x_align_text(Align::Start)
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn align_text_x_middle(self) -> Self {
        self.x_align_text(Align::Middle)
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn align_text_right(self) -> Self {
        self.x_align_text(Align::End)
    }

    /// Align the text to the left of its bounding **Rect**'s *y* axis range.
    pub fn align_text_bottom(self) -> Self {
        self.y_align_text(Align::Start)
    }

    /// Align the text to the middle of its bounding **Rect**'s *y* axis range.
    pub fn align_text_y_middle(self) -> Self {
        self.y_align_text(Align::Middle)
    }

    /// Align the text to the right of its bounding **Rect**'s *y* axis range.
    pub fn align_text_top(self) -> Self {
        self.y_align_text(Align::End)
    }

    /// Align the text to the middle of its bounding **Rect**.
    pub fn align_text_middle(self) -> Self {
        self.align_text_x_middle().align_text_y_middle()
    }

    builder_methods!{
        pub font_size { style.font_size = Some(FontSize) }
        pub react { maybe_react = Some(F) }
        pub x_align_text { style.x_align = Some(Align) }
        pub y_align_text { style.y_align = Some(Align) }
        pub line_wrap { style.line_wrap = Some(Wrap) }
        pub line_spacing { style.line_spacing = Some(Scalar) }
        pub restrict_to_height { style.restrict_to_height = Some(bool) }
    }

}

impl<'a, F> Widget for TextEdit<'a, F>
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
            cursor: Cursor::Idx(text::cursor::Index { line: 0, char: 0 }),
            drag: None,
            line_infos: Vec::new(),
            selected_rectangle_indices: Vec::new(),
            rectangle_idx: IndexSlot::new(),
            text_idx: IndexSlot::new(),
            cursor_idx: IndexSlot::new(),
            highlight_idx: IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the TextEdit.
    fn update<B: Backend>(self, args: widget::UpdateArgs<Self, B>) {
        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let TextEdit { text, .. } = self;

        let font_size = style.font_size(ui.theme());
        let line_wrap = style.line_wrap(ui.theme());
        let x_align = style.x_align(ui.theme());
        let y_align = style.y_align(ui.theme());
        let line_spacing = style.line_spacing(ui.theme());
        let restrict_to_height = style.restrict_to_height(ui.theme());
        let text_idx = state.text_idx.get(&mut ui);

        /// Returns an iterator yielding the `text::line::Info` for each line in the given text
        /// with the given styling.
        type LineInfos<'a, C> = text::line::Infos<'a, C, text::line::NextBreakFnPtr<C>>;
        fn line_infos<'a, C>(text: &'a str,
                             glyph_cache: &'a GlyphCache<C>,
                             font_size: FontSize,
                             line_wrap: Wrap,
                             max_width: Scalar) -> LineInfos<'a, C>
            where C: CharacterCache,
        {
            let infos = text::line::infos(text, glyph_cache, font_size);
            match line_wrap {
                Wrap::Whitespace => infos.wrap_by_whitespace(max_width),
                Wrap::Character => infos.wrap_by_character(max_width),
            }
        }

        // Check to see if the given text has changed since the last time the widget was updated.
        {
            let maybe_new_line_infos = {
                let line_info_slice = &state.line_infos[..];
                let new_line_infos =
                    line_infos(text, ui.glyph_cache(), font_size, line_wrap, rect.w());
                match utils::write_if_different(line_info_slice, new_line_infos) {
                    std::borrow::Cow::Owned(new) => Some(new),
                    _ => None,
                }
            };

            if let Some(new_line_infos) = maybe_new_line_infos {
                state.update(|state| state.line_infos = new_line_infos);
            }
        }

        // Find the closest cursor index to the given `xy` position.
        //
        // Returns `None` if the given `text` is empty.
        let closest_cursor_index_and_xy = |xy: Point,
                                           text: &str,
                                           line_infos: &[text::line::Info],
                                           glyph_cache: &GlyphCache<B::CharacterCache>|
            -> Option<(text::cursor::Index, Point)>
        {
            let line_infos = line_infos.iter().cloned();
            let lines = line_infos.clone().map(|info| &text[info.byte_range()]);
            let line_rects = text::line::rects(line_infos.clone(), font_size, rect,
                                               x_align, y_align, line_spacing);
            let lines_with_rects = lines.zip(line_rects.clone());

            // Find the index of the line that is closest on the *y* axis.
            let mut xys_per_line_enumerated =
                text::cursor::xys_per_line(lines_with_rects, glyph_cache, font_size).enumerate();
            xys_per_line_enumerated.next().and_then(|(first_line_idx, (_, first_line_y))| {
                let mut closest_line_idx = first_line_idx;
                let mut closest_diff = (xy[1] - first_line_y.middle()).abs();
                for (line_idx, (_, line_y)) in xys_per_line_enumerated {
                    if line_y.is_over(xy[1]) {
                        closest_line_idx = line_idx;
                        break;
                    } else {
                        let diff = (xy[1] - line_y.middle()).abs();
                        if diff < closest_diff {
                            closest_line_idx = line_idx;
                            closest_diff = diff;
                        } else {
                            break;
                        }
                    }
                }

                // Find the index of the cursor position along the closest line.
                let lines_with_rects = line_infos.map(|info| &text[info.byte_range()]).zip(line_rects);
                text::cursor::xys_per_line(lines_with_rects, glyph_cache, font_size)
                    .nth(closest_line_idx)
                    .map(|(xs, line_y)| {
                        let mut xs_enumerated = xs.enumerate();
                        // `xs` always yields at least one `x` (the start of the line).
                        let (first_idx, first_x) = xs_enumerated.next().unwrap();
                        let first_diff = (xy[0] - first_x).abs();
                        struct Closest { idx: usize, x: Scalar, diff: Scalar }
                        let mut closest = Closest { idx: first_idx, x: first_x, diff: first_diff };
                        for (i, x) in xs_enumerated {
                            let diff = (xy[0] - x).abs();
                            if diff < closest.diff {
                                closest = Closest { idx: i, x: x, diff: diff };
                            } else {
                                break;
                            }
                        }

                        let index = text::cursor::Index { line: closest_line_idx, char: closest.idx };
                        let point = [closest.x, line_y.middle()];
                        (index, point)
                    })
            })
        };

        let mut cursor = state.cursor;
        let mut drag = state.drag;

        // Check for the following events:
        // - `Text` events for receiving new text.
        // - Left mouse `Press` events for either:
        //     - setting the cursor or start of a selection.
        //     - begin dragging selected text.
        // - Left mouse `Drag` for extending the end of the selection, or for dragging selected text.
        'events: for widget_event in ui.widget_input(idx).events() {
            match widget_event {

                event::Widget::Press(press) => match press.button {

                    // If the left mouse button was pressed, place a `Cursor` with the starting
                    // index at the mouse position.
                    event::Button::Mouse(input::MouseButton::Left, rel_xy) => {
                        let abs_xy = utils::vec2_add(rel_xy, rect.xy());
                        let infos = &state.line_infos;
                        let cache = ui.glyph_cache();
                        let closest = closest_cursor_index_and_xy(abs_xy, text, infos, cache);
                        if let Some((closest_cursor, _)) = closest {
                            cursor = Cursor::Idx(closest_cursor);
                        }

                        // TODO: Differentiate between Selecting and MoveSelection.
                        drag = Some(Drag::Selecting);
                    }

                    // Check for control keys.
                    event::Button::Keyboard(key) => match key {

                        // If `Cursor::Idx`, remove the `char` behind the cursor.
                        // If `Cursor::Selection`, remove the selected text.
                        input::Key::Backspace => {
                            match cursor {

                                Cursor::Idx(cursor_idx) => {
                                    let idx_after_cursor = {
                                        let line_infos = state.line_infos.iter().cloned();
                                        text::char::index_after_cursor(line_infos, cursor_idx)
                                    };
                                    if let Some(idx) = idx_after_cursor {
                                        let idx_to_remove = idx - 1;
                                        let new_cursor_idx = {
                                            let line_infos = state.line_infos.iter().cloned();
                                            text::cursor::index_before_char(line_infos, idx_to_remove)
                                        };
                                        if let Some(new_cursor_idx) = new_cursor_idx {
                                            cursor = Cursor::Idx(new_cursor_idx);
                                            *text = text.chars().take(idx_to_remove)
                                                .chain(text.chars().skip(idx))
                                                .collect();
                                            state.update(|state| {
                                                state.line_infos =
                                                    line_infos(text, ui.glyph_cache(), font_size,
                                                               line_wrap, rect.w()).collect();
                                            });
                                        }
                                    }
                                },

                                Cursor::Selection { start, end } => {
                                    let (start_idx, end_idx) = {
                                        let line_infos = state.line_infos.iter().cloned();
                                        (text::char::index_after_cursor(line_infos.clone(), start)
                                            .expect("text::cursor::Index was out of range"),
                                         text::char::index_after_cursor(line_infos, end)
                                            .expect("text::cursor::Index was out of range"))
                                    };
                                    let (start_idx, end_idx) =
                                        if start_idx <= end_idx { (start_idx, end_idx) }
                                        else                    { (end_idx, start_idx) };
                                    let new_cursor_char_idx =
                                        if start_idx > 0 { start_idx } else { 0 };
                                    let new_cursor_idx = {
                                        let line_infos = state.line_infos.iter().cloned();
                                        text::cursor::index_before_char(line_infos, new_cursor_char_idx)
                                            .expect("char index was out of range")
                                    };
                                    cursor = Cursor::Idx(new_cursor_idx);
                                    *text = text.chars().take(start_idx)
                                        .chain(text.chars().skip(end_idx))
                                        .collect();
                                    state.update(|state| {
                                        state.line_infos =
                                            line_infos(text, ui.glyph_cache(), font_size,
                                                       line_wrap, rect.w()).collect();
                                    });
                                },

                            }
                        },

                        input::Key::Left => {
                            if !press.modifiers.contains(input::keyboard::CTRL) {
                                match cursor {

                                    // Move the cursor to the previous position.
                                    Cursor::Idx(cursor_idx) => {
                                        let new_cursor_idx = {
                                            let line_infos = state.line_infos.iter().cloned();
                                            cursor_idx.previous(line_infos).unwrap_or(cursor_idx)
                                        };

                                        cursor = Cursor::Idx(new_cursor_idx);
                                    },

                                    // Move the cursor to the start of the current selection.
                                    Cursor::Selection { start, end } => {
                                        let new_cursor_idx = std::cmp::min(start, end);
                                        cursor = Cursor::Idx(new_cursor_idx);
                                    },
                                }
                            }
                        },

                        input::Key::Right => {
                            if !press.modifiers.contains(input::keyboard::CTRL) {
                                match cursor {

                                    // Move the cursor to the next position.
                                    Cursor::Idx(cursor_idx) => {
                                        let new_cursor_idx = {
                                            let line_infos = state.line_infos.iter().cloned();
                                            cursor_idx.next(line_infos).unwrap_or(cursor_idx)
                                        };

                                        cursor = Cursor::Idx(new_cursor_idx);
                                    },

                                    // Move the cursor to the end of the current selection.
                                    Cursor::Selection { start, end } => {
                                        let new_cursor_idx = std::cmp::max(start, end);
                                        cursor = Cursor::Idx(new_cursor_idx);
                                    },
                                }
                            }
                        },

                        input::Key::Up => {
                        },
                        input::Key::Down => {
                        },

                        input::Key::A => {
                            // Select all text on Ctrl+a.
                            if press.modifiers.contains(input::keyboard::CTRL) {
                                let start = text::cursor::Index { line: 0, char: 0 };
                                let end = {
                                    let line_infos = state.line_infos.iter().cloned();
                                    text::cursor::index_before_char(line_infos, text.chars().count())
                                        .expect("char index was out of range")
                                };
                                cursor = Cursor::Selection { start: start, end: end };
                            }
                        },

                        input::Key::E => {
                            // If cursor is `Idx`, move cursor to end.
                            if press.modifiers.contains(input::keyboard::CTRL) {
                            }
                        },

                        _ => (),
                    },

                    _ => (),

                },

                event::Widget::Release(release) => {
                    // Release drag.
                    if let event::Button::Mouse(input::MouseButton::Left, _) = release.button {
                        drag = None;
                    }
                },

                event::Widget::Text(event::Text { string, modifiers }) => {
                    if modifiers.contains(input::keyboard::CTRL)
                    || string.chars().count() == 0
                    || string.chars().next().is_none() {
                        continue 'events;
                    }

                    // Ignore text produced by arrow keys.
                    // 
                    // TODO: These just happened to be the modifiers for the arrows on OS X, I've
                    // no idea if they also apply to other platforms. We should definitely see if
                    // there's a better way to handle this, or whether this should be fixed
                    // upstream.
                    match &string[..] {
                        "\u{f700}" | "\u{f701}" | "\u{f702}" | "\u{f703}" => continue 'events,
                        _ => ()
                    }

                    let string_char_count = string.chars().count();

                    // Construct the new text with the new string inserted at the cursor.
                    let (new_text, new_cursor_char_idx): (String, usize) = {
                        let (cursor_start, cursor_end) = match cursor {
                            Cursor::Idx(idx) => (idx, idx),
                            Cursor::Selection { start, end } =>
                                (std::cmp::min(start, end), std::cmp::max(start, end)),
                        };

                        let line_infos = state.line_infos.iter().cloned();

                        let (start_idx, end_idx) =
                            (text::char::index_after_cursor(line_infos.clone(), cursor_start)
                                .unwrap_or(0),
                             text::char::index_after_cursor(line_infos.clone(), cursor_end)
                                .unwrap_or(0));

                        let new_cursor_char_idx = start_idx + string_char_count;

                        let new_text = text.chars().take(start_idx)
                            .chain(string.chars())
                            .chain(text.chars().skip(end_idx))
                            .collect();
                        (new_text, new_cursor_char_idx)
                    };

                    // Calculate the new `line_infos` for the `new_text`.
                    let new_line_infos: Vec<_> = 
                        line_infos(&new_text, ui.glyph_cache(), font_size, line_wrap, rect.w())
                            .collect();

                    // Check that the new text would not exceed the `inner_rect` bounds.
                    let num_lines = new_line_infos.len();
                    let height = text::height(num_lines, font_size, line_spacing);
                    if height < rect.h() || !restrict_to_height {

                        // Determine the new `Cursor` and its position.
                        let new_cursor_idx = {
                            let line_infos = new_line_infos.iter().cloned();
                            text::cursor::index_before_char(line_infos, new_cursor_char_idx)
                                .unwrap_or(text::cursor::Index {
                                    line: 0,
                                    char: string_char_count,
                                })
                        };
                        let new_cursor = Cursor::Idx(new_cursor_idx);

                        // Update the text, cursor and line_infos.
                        *text = new_text;
                        cursor = new_cursor;
                        state.update(|state| state.line_infos = new_line_infos);
                    }
                },

                // Check whether or not 
                event::Widget::Drag(drag_event) => {
                    if let input::MouseButton::Left = drag_event.button {
                        match drag {

                            Some(Drag::Selecting) => {
                                let start_cursor_idx = match cursor {
                                    Cursor::Idx(idx) => idx,
                                    Cursor::Selection { start, .. } => start,
                                };
                                let abs_xy = utils::vec2_add(drag_event.to, rect.xy());
                                let infos = &state.line_infos;
                                let cache = ui.glyph_cache();
                                match closest_cursor_index_and_xy(abs_xy, text, infos, cache) {
                                    Some((end_cursor_idx, _)) =>
                                        cursor = Cursor::Selection {
                                            start: start_cursor_idx,
                                            end: end_cursor_idx,
                                        },
                                    _ => (),
                                }
                            },

                            // TODO: This should move the selected text.
                            Some(Drag::MoveSelection) => {
                                unimplemented!();
                            },

                            None => (),
                        }
                    }
                },

                _ => (),
            }
        }

        if state.cursor != cursor {
            state.update(|state| state.cursor = cursor);
        }

        if state.drag != drag {
            state.update(|state| state.drag = drag);
        }

        let color = style.color(ui.theme());
        let font_size = style.font_size(ui.theme());
        match line_wrap {
            Wrap::Whitespace => Text::new(&self.text).wrap_by_word(),
            Wrap::Character => Text::new(&self.text).wrap_by_character(),
        }
            .wh(rect.dim())
            .x_align_to(idx, x_align)
            .y_align_to(idx, y_align)
            .align_text_to(x_align)
            .graphics_for(idx)
            .color(color)
            .font_size(font_size)
            .set(text_idx, &mut ui);

        // Draw the line for the cursor.
        let cursor_idx = match cursor {
            Cursor::Idx(idx) => idx,
            Cursor::Selection { end, .. } => end,
        };

        // If this widget is not capturing the keyboard, no need to draw cursor or selection.
        if ui.global_input().current.widget_capturing_keyboard != Some(idx) {
            return;
        }

        // TODO: Simplify this block.
        let (cursor_x, cursor_y_range) = {
            let line_infos = state.line_infos.iter().cloned();
            let lines = line_infos.clone().map(|info| &text[info.byte_range()]);
            let line_rects = text::line::rects(line_infos.clone(), font_size, rect,
                                               x_align, y_align, line_spacing);
            let lines_with_rects = lines.zip(line_rects.clone());
            let xys_per_line = text::cursor::xys_per_line(lines_with_rects, ui.glyph_cache(), font_size);
            text::cursor::xy_at(xys_per_line, cursor_idx)
                .unwrap_or_else(|| {
                    let x = rect.left();
                    let y = Range::new(0.0, font_size as Scalar).align_to(y_align, rect.y);
                    (x, y)
                })
        };

        let cursor_line_idx = state.cursor_idx.get(&mut ui);
        let start = [0.0, cursor_y_range.start];
        let end = [0.0, cursor_y_range.end];
        Line::centred(start, end)
            .x_y(cursor_x, cursor_y_range.middle())
            .graphics_for(idx)
            .parent(idx)
            .color(color)
            .set(cursor_line_idx, &mut ui);

        if let Cursor::Selection { start, end } = cursor {
            let (start, end) = (std::cmp::min(start, end), std::cmp::max(start, end));

            let selected_rects: Vec<Rect> = {
                let line_infos = state.line_infos.iter().cloned();
                let lines = line_infos.clone().map(|info| &text[info.byte_range()]);
                let line_rects = text::line::rects(line_infos.clone(), font_size, rect,
                                                   x_align, y_align, line_spacing);
                let lines_with_rects = lines.zip(line_rects.clone());
                let cache = ui.glyph_cache();
                text::line::selected_rects(lines_with_rects, cache, font_size, start, end)
                    .collect()
            };

            // Draw a semi-transparent `Rectangle` for the selected range across each line.
            let selected_rect_color = color.highlighted().alpha(0.25);
            for (i, selected_rect) in selected_rects.iter().enumerate() {
                if i == state.selected_rectangle_indices.len() {
                    state.update(|state| {
                        state.selected_rectangle_indices.push(ui.new_unique_node_index());
                    });
                }
                let selected_rectangle_idx = state.selected_rectangle_indices[i];

                Rectangle::fill(selected_rect.dim())
                    .xy(selected_rect.xy())
                    .color(selected_rect_color)
                    .graphics_for(idx)
                    .parent(idx)
                    .set(selected_rectangle_idx, &mut ui);
            }
        }
    }

}


impl<'a, F> Colorable for TextEdit<'a, F> {
    builder_method!(color { style.color = Some(Color) });
}
