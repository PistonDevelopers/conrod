use {
    Align,
    Backend,
    CharacterCache,
    Color,
    Colorable,
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
    Rect,
    Rectangle,
    Scalar,
    Text,
    Widget,
};
use std;
use text;
use utils;
use widget::primitive::text::Wrap;
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
        /// The horizontal alignment of the text.
        - x_align: Align { Align::Start }
        /// The vertical alignment of the text.
        - y_align: Align { Align::End }
        /// The vertical space between each line of text.
        - line_spacing: Scalar { 1.0 }
        /// The way in which text is wrapped at the end of a line.
        - line_wrap: Wrap { Wrap::Whitespace }
    }
}

/// The State of the TextBox widget that will be cached within the Ui.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    cursor: Cursor,
    /// Track whether some sort of dragging is currently occurring.
    drag: Option<Drag>,
    rectangle_idx: IndexSlot,
    text_idx: IndexSlot,
    cursor_idx: IndexSlot,
    highlight_idx: IndexSlot,
}

/// Track whether some sort of dragging is currently occurring.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Drag {
    Selecting,
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
            cursor: Cursor::Idx(text::cursor::Index { line: 0, idx: 0 }),
            drag: None,
            rectangle_idx: IndexSlot::new(),
            text_idx: IndexSlot::new(),
            cursor_idx: IndexSlot::new(),
            highlight_idx: IndexSlot::new(),
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
        let TextBox { text, maybe_react, .. } = self;

        // let frame = style.frame(ui.theme());
        // let inner_rect = rect.pad(frame);
        // let max_text_rect = inner_rect.pad_left(TEXT_PADDING).pad_right(TEXT_PADDING);
        let rect = rect;
        let font_size = style.font_size(ui.theme());
        let line_wrap = style.line_wrap(ui.theme());
        let x_align = style.x_align(ui.theme());
        let y_align = style.y_align(ui.theme());
        let line_spacing = style.line_spacing(ui.theme());

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

        // Find the closest cursor index to the given `xy` position.
        //
        // Returns `None` if the given `text` is empty.
        let closest_cursor_index_and_xy = |xy: Point,
                                           text: &str,
                                           glyph_cache: &GlyphCache<B::CharacterCache>|
            -> Option<(text::cursor::Index, Point)>
        {
            let line_infos_vec: Vec<_> =
                line_infos(text, glyph_cache, font_size, line_wrap, rect.w())
                    .collect();
            let line_infos = line_infos_vec.iter().cloned();
            let lines = line_infos.clone().map(|info| &text[info.range()]);
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
                let lines_with_rects = line_infos.map(|info| &text[info.range()]).zip(line_rects);
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

                        let index = text::cursor::Index { line: closest_line_idx, idx: closest.idx };
                        let point = [closest.x, line_y.middle()];
                        (index, point)
                    })
            })
        };

        let mut cursor = state.view().cursor;
        let mut drag = state.view().drag;

        // Check for the following events:
        // - `Text` events for receiving new text.
        // - Left mouse `Press` events for either:
        //     - setting the cursor or start of a selection.
        //     - begin dragging selected text.
        // - Left mouse `Drag` for extending the end of the selection, or for dragging selected text.
        'events: for widget_event in ui.widget_input(idx).events() {
            use event;
            use input;
            match widget_event {

                event::Widget::Press(press) => match press.button {

                    // If the left mouse button was pressed, place a `Cursor` with the starting
                    // index at the mouse position.
                    event::Button::Mouse(input::MouseButton::Left, rel_xy) => {
                        let abs_xy = utils::vec2_add(rel_xy, rect.xy());
                        let closest = closest_cursor_index_and_xy(abs_xy, text, ui.glyph_cache());
                        if let Some((closest_cursor, closest_cursor_xy)) = closest {
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
                            let line_infos: Vec<_> =
                                line_infos(text, ui.glyph_cache(), font_size,
                                           line_wrap, rect.w())
                                    .collect();
                            let line_infos = line_infos.iter().cloned();
                            match cursor {

                                Cursor::Idx(cursor_idx) => {
                                    let idx =
                                        text::char::index_after_cursor(line_infos.clone(), cursor_idx)
                                            .expect("text::cursor::Index was out of range");
                                    if idx > 0 {
                                        let idx_to_remove = idx - 1;
                                        let new_cursor_idx =
                                            text::cursor::index_before_char(line_infos, idx_to_remove)
                                                .expect("char index was out of range");
                                        cursor = Cursor::Idx(new_cursor_idx);
                                        text.remove(idx_to_remove);
                                    }
                                },

                                Cursor::Selection { start, end } => {
                                    let (start_idx, end_idx) =
                                        (text::char::index_after_cursor(line_infos.clone(), start)
                                            .expect("text::cursor::Index was out of range"),
                                         text::char::index_after_cursor(line_infos.clone(), end)
                                            .expect("text::cursor::Index was out of range"));
                                    let (start_idx, end_idx) = if start_idx <= end_idx {
                                        (start_idx, end_idx)
                                    } else {
                                        (end_idx, start_idx)
                                    };
                                    let new_cursor_char_idx =
                                        if start_idx > 0 { start_idx - 1 } else { 0 };
                                    let new_cursor_idx =
                                        text::cursor::index_before_char(line_infos, new_cursor_char_idx)
                                            .expect("char index was out of range");
                                    cursor = Cursor::Idx(new_cursor_idx);
                                    *text = text[0..start_idx].chars().chain(text[end_idx..].chars()).collect();
                                },

                            }
                        },

                        // Move the cursor through the text.
                        input::Key::Left => {
                        },
                        input::Key::Right => {
                        },
                        input::Key::Up => {
                        },
                        input::Key::Down => {
                        },

                        input::Key::A => {
                            // Select all text on Ctrl+a.
                            if press.modifiers.contains(input::keyboard::CTRL) {
                                let start = text::cursor::Index { line: 0, idx: 0 };
                                let end = {
                                    let line_infos =
                                        line_infos(text, ui.glyph_cache(), font_size,
                                                   line_wrap, rect.w());
                                    text::cursor::index_before_char(line_infos, text.len())
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

                    println!("{:?}", string);

                    let (new_text, new_cursor): (String, Cursor) = {
                        let (cursor_start, cursor_end) = match cursor {
                            Cursor::Idx(idx) => (idx, idx),
                            Cursor::Selection { start, end } =>
                                (std::cmp::min(start, end), std::cmp::max(start, end)),
                        };

                        let line_infos_vec: Vec<_> =
                            line_infos(text, ui.glyph_cache(), font_size, line_wrap, rect.w())
                                .collect();
                        let line_infos = line_infos_vec.iter().cloned();

                        let (start_idx, end_idx) =
                            (text::char::index_after_cursor(line_infos.clone(), cursor_start)
                                .expect("text::cursor::Index was out of range"),
                             text::char::index_after_cursor(line_infos.clone(), cursor_end)
                                .expect("text::cursor::Index was out of range"));

                        let new_cursor_idx = {
                            let new_cursor_char_idx = start_idx + string.chars().count();
                            text::cursor::index_before_char(line_infos, new_cursor_char_idx)
                                .expect("char index was out of range")
                        };

                        let new_cursor = Cursor::Idx(new_cursor_idx);
                        let new_text = text.chars().take(start_idx)
                            .chain(string.chars())
                            .chain(text.chars().skip(end_idx))
                            .collect();
                        (new_text, new_cursor)
                    };

                    // Check that the new text would not exceed the `inner_rect` bounds.
                    let num_lines =
                        line_infos(text, ui.glyph_cache(), font_size, line_wrap, rect.w())
                            .count();
                    let height = text::height(num_lines, font_size, line_spacing);
                    if height < rect.h() {
                        *text = new_text;
                        cursor = new_cursor;
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
                                match closest_cursor_index_and_xy(abs_xy, text, ui.glyph_cache()) {
                                    Some((end_cursor_idx, _)) =>
                                        cursor = Cursor::Selection {
                                            start: start_cursor_idx,
                                            end: end_cursor_idx,
                                        },
                                    _ => (),
                                }
                            },

                            Some(Drag::MoveSelection) => {
                            },

                            None => (),
                        }
                    }
                },

                _ => (),
            }
        }

        if state.view().cursor != cursor {
            state.update(|state| state.cursor = cursor);
        }

        if state.view().drag != drag {
            state.update(|state| state.drag = drag);
        }

        let text_color = style.text_color(ui.theme());
        let font_size = style.font_size(ui.theme());
        let text_idx = state.view().text_idx.get(&mut ui);
        match line_wrap {
            Wrap::Whitespace => Text::new(&self.text).wrap_by_word(),
            Wrap::Character => Text::new(&self.text).wrap_by_character(),
        }
            .x_align_to(idx, x_align)
            .y_align_to(idx, y_align)
            .graphics_for(idx)
            .color(text_color)
            .font_size(font_size)
            .set(text_idx, &mut ui);

        // Draw the line for the cursor.
        let cursor_idx = match cursor {
            Cursor::Idx(idx) => idx,
            Cursor::Selection { start, end } => end,
        };

        // If this widget is not capturing the keyboard, no need to draw cursor or selection.
        if ui.global_input().current.widget_capturing_keyboard != Some(idx) {
            return;
        }

        // TODO: Simplify this block!
        let (cursor_x, cursor_y_range) = {
            let line_infos_vec: Vec<_> =
                line_infos(text, ui.glyph_cache(), font_size, line_wrap, rect.w())
                    .collect();
            let line_infos = line_infos_vec.iter().cloned();
            let lines = line_infos.clone().map(|info| &text[info.range()]);
            let line_rects = text::line::rects(line_infos.clone(), font_size, rect,
                                               x_align, y_align, line_spacing);
            let lines_with_rects = lines.zip(line_rects.clone());
            let xys_per_line = text::cursor::xys_per_line(lines_with_rects, ui.glyph_cache(), font_size);
            text::cursor::xy_at(xys_per_line, cursor_idx)
                .expect("cursor index was not in range of text")
        };

        let cursor_line_idx = state.view().cursor_idx.get(&mut ui);
        let start = [0.0, cursor_y_range.start];
        let end = [0.0, cursor_y_range.end];
        Line::centred(start, end)
            .x_y(cursor_x, cursor_y_range.middle())
            .graphics_for(idx)
            .parent(idx)
            .color(text_color)
            .set(cursor_line_idx, &mut ui);

        // if let Interaction::Captured(view) = new_interaction {
        //     let cursor = view.cursor;
        //     // This matters if the text is scrolled with the mouse.
        //     let cursor_idx = match cursor.anchor {
        //         Anchor::End => cursor.start,
        //         Anchor::Start | Anchor::None => cursor.end,
        //     };
        //     let cursor_x = cursor_position(ui.glyph_cache(), cursor_idx, text_start_x, font_size,
        //                                    &self.text);

        //     if cursor.is_cursor() {
        //         let cursor_idx = state.view().cursor_idx.get(&mut ui);
        //         let start = [0.0, 0.0];
        //         let end = [0.0, inner_rect.h()];
        //         Line::centred(start, end)
        //             .x_relative_to(idx, cursor_x)
        //             .graphics_for(idx)
        //             .parent(idx)
        //             .color(text_color)
        //             .set(cursor_idx, &mut ui);
        //     } else {
        //         let (rel_x, w) = {
        //             let (start, end) = (cursor.start, cursor.end);
        //             let cursor_x = cursor_position(ui.glyph_cache(), start, text_start_x, font_size,
        //                                            &self.text);
        //             let highlighted_text = &self.text[start..end];
        //             let w = ui.glyph_cache().width(font_size, &highlighted_text);
        //             (cursor_x + w / 2.0, w)
        //         };
        //         let dim = [w, inner_rect.h()];
        //         let highlight_idx = state.view().highlight_idx.get(&mut ui);
        //         Rectangle::fill(dim)
        //             .x_relative_to(idx, rel_x)
        //             .color(text_color.highlighted().alpha(0.25))
        //             .graphics_for(idx)
        //             .parent(idx)
        //             .set(highlight_idx, &mut ui);
        //     }
        // }
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
