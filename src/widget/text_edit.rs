//! A widget for displaying and mutating multi-line text, given as a `String`.

use std::fmt;

use {Color, Colorable, FontSize, Positionable, Sizeable, Widget, Ui};
use event;
use input;
use position::{Align, Dimension, Point, Range, Rect, Scalar};
use std;
use text;
use utils;
use widget;
use cursor;
use widget::primitive::text::Wrap;

use smallstring::SmallString;
use void::Void;

/// A widget for displaying and mutating multi-line text, given as a `String`.
///
/// By default the text is wrapped via the first whitespace before the line exceeds the
/// `TextEdit`'s width, however a user may change this using the `.wrap_by_character` method.
///
/// Note that when text is updated with this widget, the internal text / text displayed is
/// not changed. Instead, the changes are returned as TextEvents, and it's up to the user to
/// update the actual text and display it the next widget cycle.
#[derive(WidgetCommon_)]
pub struct TextEdit<'a, EventTransform> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    text: &'a str,
    event_transform: EventTransform,
    style: Style,
}

/// Unique graphical styling for the TextEdit.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// The color of the text (this includes cursor and selection color).
    #[conrod(default = "theme.shape_color")]
    pub color: Option<Color>,
    /// The font size for the text.
    #[conrod(default = "theme.font_size_medium")]
    pub font_size: Option<FontSize>,
    /// The horizontal alignment of the text.
    #[conrod(default = "text::Justify::Left")]
    pub justify: Option<text::Justify>,
    /// The vertical alignment of the text.
    #[conrod(default = "Align::End")]
    pub y_align: Option<Align>,
    /// The vertical space between each line of text.
    #[conrod(default = "1.0")]
    pub line_spacing: Option<Scalar>,
    /// The way in which text is wrapped at the end of a line.
    #[conrod(default = "Wrap::Whitespace")]
    pub line_wrap: Option<Wrap>,
    /// Do not allow to enter text that would exceed the bounds of the `TextEdit`'s `Rect`.
    #[conrod(default = "true")]
    pub restrict_to_height: Option<bool>,
    /// The font used for the `Text`.
    #[conrod(default = "theme.font_id")]
    pub font_id: Option<Option<text::font::Id>>,
}

widget_ids! {
    struct Ids {
        selected_rectangles[],
        text,
        cursor,
    }
}

/// The State of the TextEdit widget that will be cached within the Ui.
pub struct State {
    cursor: Cursor,
    /// Track whether some sort of dragging is currently occurring.
    drag: Option<Drag>,
    /// Information about each line of text.
    line_infos: Vec<text::line::Info>,
    ids: Ids,
}

/// Track whether some sort of dragging is currently occurring.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Drag {
    /// The drag is currently selecting a range of text.
    Selecting,
    /// The drag is moving a selection of text.
    #[allow(dead_code)] // TODO: Implement this.
    MoveSelection,
}

/// The position of the `Cursor` over the text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cursor {
    /// The cursor is at the given character index.
    Idx(text::cursor::Index),
    /// The cursor is a selection between these two indices.
    Selection {
        /// The `start` is always the "anchor" point.
        start: text::cursor::Index,
        /// The `end` may be either greater or less than the `start`.
        ///
        /// The `end` is always the logical cursor position, if one is required. For example, when
        /// selecting text using `Shift+Right`, `end` is moved.
        end: text::cursor::Index,
    },
}

impl<'a> TextEdit<'a, fn(TextEvent<Void>) -> TextEvent<Void>> {
    /// Construct a TextEdit widget.
    pub fn new(text: &'a str) -> Self {
        fn default_transform(evt: TextEvent) -> TextEvent {
            evt
        }

        TextEdit::with_transform(text, default_transform)
    }
}

impl<'a, T> TextEdit<'a, T> {
    /// Construct a TextEdit widget with the given event transformation
    /// closure.
    pub fn with_transform<E>(text: &'a str, transform: T) -> Self
    where
        T: FnMut(TextEvent<Void>) -> TextEvent<E>,
    {
        TextEdit {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            text: text,
            event_transform: transform,
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
    pub fn left_justify(self) -> Self {
        self.justify(text::Justify::Left)
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn center_justify(self) -> Self {
        self.justify(text::Justify::Center)
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn right_justify(self) -> Self {
        self.justify(text::Justify::Right)
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
        self.center_justify().align_text_y_middle()
    }

    /// Specify the font used for displaying the text.
    pub fn font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.font_id = Some(Some(font_id));
        self
    }

    builder_methods!{
        pub font_size { style.font_size = Some(FontSize) }
        pub justify { style.justify = Some(text::Justify) }
        pub y_align_text { style.y_align = Some(Align) }
        pub line_wrap { style.line_wrap = Some(Wrap) }
        pub line_spacing { style.line_spacing = Some(Scalar) }
        pub restrict_to_height { style.restrict_to_height = Some(bool) }
    }
}

/// Text event.
#[derive(Clone)]
pub enum TextEvent<T = Void> {
    /// Passes through an event which is a no-op on the text, but can be picked
    /// up by whatever receives the TextEdit events. This is only ever returned
    /// if it's returned from the event_transform function.
    PassthroughData {
        /// The data to pass through.
        data: T,
    },
    /// Removes text from a string, and replaces it with other text.
    ///
    /// This is a catch-all variation for all direct modifications. If 'length'
    /// is 0, this is pure insertion. If 'text' is empty, this is pure deletion.
    Splice {
        /// Start byte index.
        start_index: usize,
        /// Byte length removed.
        length: usize,
        /// Text inserted into the removed text's place.
        text: SmallString<[u8; 8]>,
    },
    /// Moves text from one place in the string to another, without caring what
    /// the text is. This could be represented by two Splices, but it's more
    /// memory-efficient this way while storing events and has the potential to
    /// be more efficient if wrapping string storage is something like a rope.
    MoveText {
        /// Start byte index.
        start_index: usize,
        /// Byte length.
        length: usize,
        /// Index to insert at *in the original string*. If you've already removed the text,
        /// then this index is going to be `(insertion_index - length)` if `insertion_index > start_index`.
        ///
        /// If `start_index <= insertion_index <= start_index + length`, this is a no-op.
        insertion_index: usize,
    },
}
impl<T> fmt::Debug for TextEvent<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TextEvent::PassthroughData { .. } => f.debug_struct("TextEvent::PassthroughData")
                .field("data", &"<non-debug>")
                .finish(),
            TextEvent::Splice {
                start_index,
                length,
                ref text,
            } => f.debug_struct("TextEvent::Splice")
                .field("start_index", &start_index)
                .field("length", &length)
                .field("text", &text)
                .finish(),
            TextEvent::MoveText {
                start_index,
                length,
                insertion_index,
            } => f.debug_struct("TextEvent::MoveText")
                .field("start_index", &start_index)
                .field("length", &length)
                .field("insertion_index", &insertion_index)
                .finish(),
        }
    }
}

impl<T> TextEvent<T> {
    /* // We'll possibly want this, but for now we can do a "EventAllowed" closure instead.

    /// Turns an event which was intended to be applied after this set of events into one which can be applied
    /// while ignoring these events.
    ///
    /// Example: if there are three events, "insert char A at 0", "insert char B at 1", "insert char C at 2",
    /// they work when applied together. What if you have a text box which you only want to allow the character
    /// "C" in however? Applying the third event doesn't work, since it will insert it in the wrong position.
    ///
    /// This is where `into_correct_without_applying` comes in: you can call
    ///
    /// ```ignore
    /// (third_event).into_correct_without_applying(&[first_event, second_event]).apply(string)
    /// ```
    /// and it will act as though the event were "insert char C at 0".
    pub fn into_correct_without_applying<'a, T>(self, events: T) -> TextEvent
    where
        T: IntoIterator<Item = &'a TextEvent>
    {

    }

    */

    /// Returns true if this operation does anything when applied to strings.
    ///
    /// This will return false if this operation is a no-op AND it isn't a passthrough event.
    pub fn no_op(&self) -> bool {
        match *self {
            TextEvent::Splice {
                start_index: _,
                length,
                ref text,
            } => length == 0 && text.len() == 0,
            TextEvent::MoveText {
                start_index,
                length,
                insertion_index,
            } => length == 0 || (start_index <= insertion_index && insertion_index < (start_index + length)),
            TextEvent::PassthroughData { .. } => true,
        }
    }

    /// Returns true if this operation is a "passthrough" event, one which is a no-op
    /// with regards to the string, but is useful to outside consumers.
    pub fn is_passthrough(&self) -> bool {
        match *self {
            TextEvent::PassthroughData { .. } => true,
            _ => false,
        }
    }

    /// Creates a new string from the given str with this event applied.
    ///
    /// # Panics
    ///
    /// Panics if the indices stored in this TextEvent are out of bounds for
    /// the given string, or if the indices to match up with `char` boundaries.
    pub fn apply_new(&self, existing_text: &str) -> String {
        // TODO: make a more efficient version of this function which uses iterator
        // chains instead of modifying memory (and test if it's actually faster after
        // that). Or at least a version which copies "start text" then "middle text"
        // then "end text" instead of copying "start text" "end text" then inserting
        // "middle text".
        //
        // something like this:
        //
        // let new_text = text.chars().take(start_idx)
        //     .chain(string.chars())
        //     .chain(text.chars().skip(end_idx))
        //     .collect();
        let mut owned = existing_text.to_owned();
        self.apply(&mut owned);
        owned
    }

    /// Modifies the given string in place with this event.
    ///
    /// # Panics
    ///
    /// Panics if the indices stored in this TextEvent are out of bounds for
    /// the given string, or if the indices to match up with `char` boundaries.
    pub fn apply(&self, existing_text: &mut String) {
        match *self {
            TextEvent::Splice {
                start_index,
                length,
                ref text,
            } => {
                if length == 0 {
                    existing_text.insert_str(start_index, text);
                } else if text.len() == 0 {
                    existing_text.drain(start_index..(start_index + length));
                } else {
                    // TODO: use String::splice when it's stable.
                    existing_text.drain(start_index..(start_index + length));
                    existing_text.insert_str(start_index, text);
                }
            }
            TextEvent::MoveText {
                start_index,
                length,
                insertion_index,
            } => {
                if start_index <= insertion_index && insertion_index < (start_index + length) {
                    return;
                }
                // This is somewhat naive, but that's OK: if someone's editing large strings,
                // they're likely using a rope structure anyways, and they can manually
                // re-implement this method.
                let text_to_move: String = existing_text
                    .drain(start_index..(start_index + length))
                    .collect();

                let new_insert_index = if insertion_index < start_index {
                    insertion_index
                } else {
                    // TODO: verify this is correct
                    insertion_index - length
                };

                existing_text.insert_str(new_insert_index, &text_to_move);
            }
            TextEvent::PassthroughData { .. } => {}
        }
    }

    /// Gets the start index of the text affected after this event is applied.
    ///
    /// This will return 0 if the event is a `TextEvent::PassthroughData`.
    pub fn start_index(&self) -> usize {
        match *self {
            TextEvent::Splice { start_index, .. } => start_index,
            TextEvent::MoveText {
                start_index,
                length,
                insertion_index,
            } => {
                if insertion_index < start_index {
                    insertion_index
                } else if insertion_index < (start_index + length) {
                    // no-op version
                    start_index
                } else {
                    // we do 'insertion_index - length' because we've removed the same length of
                    // text before, then we add length to get the end of the text.
                    insertion_index - length
                }
            }
            // TODO: is this the best API surface we can make?
            // Could we do a "NonPassthroughEvent"? or should we get rid of that variant alltogether
            // and make events a trait which must be AsRef<TextEvent>?
            TextEvent::PassthroughData { .. } => 0,
        }
    }

    /// Gets the end index of text affected after this event is applied.
    ///
    /// This will return 0 if the event is a `TextEvent::PassthroughData`.
    pub fn end_index(&self) -> usize {
        match *self {
            TextEvent::Splice {
                start_index,
                length,
                ref text,
            } => start_index + text.len(),
            TextEvent::MoveText {
                start_index,
                length,
                insertion_index,
            } => {
                if insertion_index < start_index {
                    insertion_index + length
                } else if insertion_index < (start_index + length) {
                    // no-op version
                    start_index + length
                } else {
                    // we do 'insertion_index - length' because we've removed the same length of
                    // text before, then we add length to get the end of the text.
                    insertion_index
                }
            }
            // see concerns for similar statement in start_index
            TextEvent::PassthroughData { .. } => 0,
        }
    }
}

impl TextEvent<Void> {
    /// Transforms this event into a type which could contain a different type of extra data.
    ///
    /// The new event is exactly the same as this event, since TextEvent<Void> can never be
    /// the 'extra data' variant.
    pub fn into_specific_event<T>(self) -> TextEvent<T> {
        match self {
            TextEvent::Splice {
                start_index,
                length,
                text,
            } => TextEvent::Splice {
                start_index,
                length,
                text,
            },
            TextEvent::MoveText {
                start_index,
                length,
                insertion_index,
            } => TextEvent::MoveText {
                start_index,
                length,
                insertion_index,
            },
            TextEvent::PassthroughData { data } => match data {}, // data is 'Void'
        }
    }
}

/*
struct TextWithEvents<'a>(&'a str, &'a [TextEvent]);

impl TextWithEvents {
    fn new<'a>(s: &'a str, events: &'a [TextEvent]) {
        TextWithEvents(s, events)
    }
}
*/

impl<'a, Transformer, ExtraEvent> Widget for TextEdit<'a, Transformer>
where
    Transformer: FnMut(TextEvent<Void>) -> TextEvent<ExtraEvent>,
{
    type State = State;
    type Style = Style;
    // TODO: We should create a more specific `Event` type that:
    // - Allows for mutating an existing `String` directly
    // - Enumerates possible mutations (i.e. InsertChar, RemoveCharRange, etc).
    // - Enumerates cursor movement and range selection.
    type Event = Vec<TextEvent<ExtraEvent>>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            cursor: Cursor::Idx(text::cursor::Index { line: 0, char: 0 }),
            drag: None,
            line_infos: Vec::new(),
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn default_y_dimension(&self, ui: &Ui) -> Dimension {
        // If the user has specified `restrict_to_height = true`, then we should infer the height
        // using the previous widget as is the default case.
        if self.style.restrict_to_height(&ui.theme) {
            return widget::default_y_dimension(self, ui);
        }

        // Otherwise the height is unrestricted, and we should infer the height as the total height
        // of the fully styled, wrapped text.
        let font = match self.style.font_id(&ui.theme)
            .or(ui.fonts.ids().next())
            .and_then(|id| ui.fonts.get(id))
        {
            Some(font) => font,
            None => return Dimension::Absolute(0.0),
        };

        let text = &self.text;
        let font_size = self.style.font_size(&ui.theme);
        let num_lines = match self.get_w(ui) {
            None => text.lines().count(),
            Some(max_w) => match self.style.line_wrap(&ui.theme) {
                Wrap::Character =>
                    text::line::infos(text, font, font_size)
                        .wrap_by_character(max_w)
                        .count(),
                Wrap::Whitespace =>
                    text::line::infos(text, font, font_size)
                        .wrap_by_whitespace(max_w)
                        .count(),
            },
        };
        let line_spacing = self.style.line_spacing(&ui.theme);
        let height = text::height(std::cmp::max(num_lines, 1), font_size, line_spacing);
        Dimension::Absolute(height)
    }

    /// Update the state of the TextEdit.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, rect, style, ui, .. } = args;
        let TextEdit {
            text: original_text,
            mut event_transform,
            ..
        } = self;
        let mut cached_text = std::borrow::Cow::Borrowed(original_text);

        let mut events = Vec::<TextEvent<_>>::new();

        // Retrieve the `font_id`, as long as a valid `Font` for it still exists.
        //
        // If we've no font to use for text logic, bail out without updating.
        let font_id = match style.font_id(&ui.theme)
            .or(ui.fonts.ids().next())
            .and_then(|id| ui.fonts.get(id).map(|_| id))
        {
            Some(font_id) => font_id,
            None => return events,
        };

        let font_size = style.font_size(ui.theme());
        let line_wrap = style.line_wrap(ui.theme());
        let justify = style.justify(ui.theme());
        let y_align = style.y_align(ui.theme());
        let line_spacing = style.line_spacing(ui.theme());
        let restrict_to_height = style.restrict_to_height(ui.theme());

        /// Returns an iterator yielding the `text::line::Info` for each line in the given text
        /// with the given styling.
        type LineInfos<'a> = text::line::Infos<'a, text::line::NextBreakFnPtr>;
        fn line_infos<'a>(text: &'a str,
                          font: &'a text::Font,
                          font_size: FontSize,
                          line_wrap: Wrap,
                          max_width: Scalar) -> LineInfos<'a>
        {
            let infos = text::line::infos(text, font, font_size);
            match line_wrap {
                Wrap::Whitespace => infos.wrap_by_whitespace(max_width),
                Wrap::Character => infos.wrap_by_character(max_width),
            }
        }

        // Check to see if the given text has changed since the last time the widget was updated.
        {
            let maybe_new_line_infos = {
                let line_info_slice = &state.line_infos[..];
                let font = ui.fonts.get(font_id).unwrap();
                let new_line_infos = line_infos(&cached_text, font, font_size, line_wrap, rect.w());
                match utils::write_if_different(line_info_slice, new_line_infos) {
                    std::borrow::Cow::Owned(new) => Some(new),
                    _ => None,
                }
            };

            if let Some(new_line_infos) = maybe_new_line_infos {
                state.update(|state| state.line_infos = new_line_infos);
            }
        }

        // Validate the position of the cursor. Ensure the indices lie within the text.
        match state.cursor {
            Cursor::Idx(index) => {
                let new_index = index.clamp_to_lines(state.line_infos.iter().cloned());
                if index != new_index {
                    let new_cursor = Cursor::Idx(new_index);
                    state.update(|state| state.cursor = new_cursor);
                }
            },
            Cursor::Selection { start, end } => {
                let new_start = start.clamp_to_lines(state.line_infos.iter().cloned());
                let new_end = end.clamp_to_lines(state.line_infos.iter().cloned());
                if start != new_start || end != new_end {
                    let new_cursor = Cursor::Selection { start: new_start, end: new_end };
                    state.update(|state| state.cursor = new_cursor);
                }
            },
        }

        // Find the position of the cursor at the given index over the given text.
        let cursor_xy_at = |cursor_idx: text::cursor::Index,
                            text: &str,
                            line_infos: &[text::line::Info],
                            font: &text::Font|
            -> Option<(Scalar, Range)>
        {
            let xys_per_line = text::cursor::xys_per_line_from_text(text, line_infos, font,
                                                                    font_size, justify, y_align,
                                                                    line_spacing, rect);
            text::cursor::xy_at(xys_per_line, cursor_idx)
        };

        // Find the closest cursor index to the given `xy` position.
        //
        // Returns `None` if the given `text` is empty.
        let closest_cursor_index_and_xy = |xy: Point,
                                           text: &str,
                                           line_infos: &[text::line::Info],
                                           font: &text::Font|
            -> Option<(text::cursor::Index, Point)>
        {
            let xys_per_line = text::cursor::xys_per_line_from_text(text, line_infos, font,
                                                                    font_size, justify, y_align,
                                                                    line_spacing, rect);
            text::cursor::closest_cursor_index_and_xy(xy, xys_per_line)
        };

        // Find the closest cursor index to the given `x` position over the given line.
        let closest_cursor_index_on_line = |x_pos: Scalar,
                                            line_idx: usize,
                                            text: &str,
                                            line_infos: &[text::line::Info],
                                            font: &text::Font| -> Option<text::cursor::Index>
        {
            let mut xys_per_line = text::cursor::xys_per_line_from_text(text, line_infos, font,
                                                                        font_size, justify, y_align,
                                                                        line_spacing, rect);
            xys_per_line.nth(line_idx).and_then(|(line_xs,_)| {
                let (char_idx,_) = text::cursor::closest_cursor_index_on_line(x_pos,line_xs);
                Some(text::cursor::Index { line: line_idx, char: char_idx })
            })
        };

        let mut cursor = state.cursor;
        let mut drag = state.drag;

        // Insert the given `string` at the given `cursor` position within the given `text`.
        //
        // Produces the resulting TextEvent, cursor position and `line::Info`s for the new text.
        //
        // Returns `None` if the new text would exceed the height restriction.
        //
        // TODO: decide if we should return the created String representing the new text, or if
        // it's reasonable to leave it without that so we can in future avoid creating that string.
        let insert_text = |string: &str,
                               cursor: Cursor,
                               text: &str,
                               infos: &[text::line::Info],
                               font: &text::Font,
                               event_transform: &mut Transformer|
         -> Option<
            (
                TextEvent<ExtraEvent>,
                Option<(Cursor, std::vec::Vec<text::line::Info>)>,
            ),
        > {
            // Construct the new text with the new string inserted at the cursor.
            let (new_text, event, new_cursor_char_idx): (String, TextEvent<_>, usize) = {
                let (cursor_start, cursor_end) = match cursor {
                    Cursor::Idx(idx) => (idx, idx),
                    Cursor::Selection { start, end } =>
                        (std::cmp::min(start, end), std::cmp::max(start, end)),
                };

                let line_infos = infos.iter().cloned();

                let (start_idx, end_idx) =
                    (text::glyph::index_after_cursor(line_infos.clone(), cursor_start)
                        .unwrap_or(0),
                     text::glyph::index_after_cursor(line_infos.clone(), cursor_end)
                        .unwrap_or(0));

                let event = TextEvent::Splice {
                    start_index: start_idx,
                    length: end_idx - start_idx,
                    text: string.into(),
                };

                let event = event_transform(event);

                if event.no_op() {
                    if event.is_passthrough() {
                        return Some((event, None));
                    } else {
                        return None;
                    }
                }

                // TODO: make a function to get "line infos on string w/ this one event applied"
                // so we can do "event.apply(text.to_mut());" instead.
                let mut new_text = event.apply_new(&text);

                let new_cursor_char_idx = event.end_index();

                (new_text, event, new_cursor_char_idx)
            };

            // Calculate the new `line_infos` for the `new_text`.
            let new_line_infos: Vec<_> = {
                line_infos(&new_text, font, font_size, line_wrap, rect.w()).collect()
            };

            // Check that the new text would not exceed the `inner_rect` bounds.
            let num_lines = new_line_infos.len();
            let height = text::height(num_lines, font_size, line_spacing);
            if height < rect.h() || !restrict_to_height {

                // Determine the new `Cursor` and its position.
                let new_cursor_idx = {
                    let line_infos = new_line_infos.iter().cloned();
                    text::cursor::index_before_char(line_infos, new_cursor_char_idx).unwrap_or(text::cursor::Index {
                        line: 0,
                        // TODO: this is what this was before events. Understand what this meant, and re-do it!
                        // "let string_char_count = string.chars().count();"
                        // char: string_char_count,
                        char: new_cursor_char_idx,
                    })
                };

                Some((event, Some((Cursor::Idx(new_cursor_idx), new_line_infos))))
            } else {
                None
            }
        };

        // Check for the following events:
        // - `Text` events for receiving new text.
        // - Left mouse `Press` events for either:
        //     - setting the cursor or start of a selection.
        //     - begin dragging selected text.
        // - Left mouse `Drag` for extending the end of the selection, or for dragging selected text.
        // - Key presses for cursor movement.
        'events: for widget_event in ui.widget_input(id).events() {
            match widget_event {
                event::Widget::Press(press) => match press.button {
                    // If the left mouse button was pressed, place a `Cursor` with the starting
                    // index at the mouse position.
                    event::Button::Mouse(input::MouseButton::Left, rel_xy) => {
                        let abs_xy = utils::vec2_add(rel_xy, rect.xy());
                        let infos = &state.line_infos;
                        let font = ui.fonts.get(font_id).unwrap();
                        let closest = closest_cursor_index_and_xy(abs_xy, &cached_text, infos, font);
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
                        input::Key::Backspace | input::Key::Delete => {
                            let delete_word = press.modifiers.contains(input::keyboard::ModifierKey::CTRL);

                            // Calculate start/end indices of text to remove
                            let (start, end) = match cursor {
                                Cursor::Idx(cursor_idx) => {
                                    let line_infos = state.line_infos.iter().cloned();

                                    let end = match (key, delete_word) {
                                        (input::Key::Backspace, false) => {
                                            cursor_idx.previous(line_infos)
                                        }
                                        (input::Key::Backspace, true) => {
                                            cursor_idx.previous_word_start(&cached_text, line_infos)
                                        }
                                        (input::Key::Delete, false) => {
                                            cursor_idx.next(line_infos)
                                        }
                                        (input::Key::Delete, true) => {
                                            cursor_idx.next_word_end(&cached_text, line_infos)
                                        }
                                        _ => unreachable!(),
                                    }.unwrap_or(cursor_idx);

                                    (cursor_idx, end)
                                }
                                Cursor::Selection { start, end } => (start, end),
                            };

                            let (start_idx, end_idx) = {
                                let line_infos = state.line_infos.iter().cloned();
                                (text::glyph::index_after_cursor(line_infos.clone(), start),
                                 text::glyph::index_after_cursor(line_infos, end))
                            };

                            if let (Some(start_idx), Some(end_idx)) = (start_idx, end_idx) {
                                let (start_idx, end_idx) = (std::cmp::min(start_idx, end_idx),
                                                            std::cmp::max(start_idx, end_idx));

                                let event = TextEvent::Splice {
                                    start_index: start_idx,
                                    length: end_idx - start_idx,
                                    text: "".into(),
                                };
                                let event = event_transform(event);
                                // TODO: event_transform can even turn deletes into insertions... We need to
                                // put the same safeguards here that we have in `insert_text` for text which
                                // possibly escapes the restricted height...
                                // TODO: should event_transform be the thing which possibly guards against
                                // height in the first place? it could be made to be Fn(&str, TextEvent)
                                // instead of Fn(TextEvent) so it can check height. Or we could do a local
                                // closure which wraps it...

                                if !event.no_op() {
                                    event.apply(cached_text.to_mut());
                                    state.update(|state| {
                                        let font = ui.fonts.get(font_id).unwrap();
                                        let w = rect.w();
                                        state.line_infos =
                                            line_infos(&cached_text, font, font_size, line_wrap, w).collect();
                                    });
                                    let new_cursor_char_idx = event.end_index();
                                    let new_cursor_idx = {
                                        let line_infos = state.line_infos.iter().cloned();
                                        text::cursor::index_before_char(line_infos, new_cursor_char_idx)
                                            .unwrap_or_else(|| {
                                                panic!(
                                                    "end char index for event {:#?} was out of range of {:#?}",
                                                    event, state.line_infos
                                                )
                                            })
                                        // .unwrap_or(text::cursor::Index {
                                        //     line: 0,
                                        //     char: new_cursor_char_idx,
                                        // })
                                    };
                                    cursor = Cursor::Idx(new_cursor_idx);
                                    events.push(event);
                                } else {
                                    println!("no-op event ignored: {:#?}", event);
                                    if event.is_passthrough() {
                                        events.push(event);
                                    }
                                }
                            }
                        },

                        input::Key::Left | input::Key::Right | input::Key::Up | input::Key::Down => {
                            let font = ui.fonts.get(font_id).unwrap();
                            let move_word = press.modifiers.contains(input::keyboard::ModifierKey::CTRL);
                            let select = press.modifiers.contains(input::keyboard::ModifierKey::SHIFT);

                            let (old_selection_start, cursor_idx) = match cursor {
                                Cursor::Idx(idx) => (idx, idx),
                                Cursor::Selection { start, end } => (start, end),
                            };

                            let new_cursor_idx = {
                                let line_infos = state.line_infos.iter().cloned();
                                match (key, move_word) {
                                    (input::Key::Left, true) => cursor_idx
                                        .previous_word_start(&cached_text, line_infos),
                                    (input::Key::Right, true) => cursor_idx
                                        .next_word_end(&cached_text, line_infos),
                                    (input::Key::Left, false) => cursor_idx
                                        .previous(line_infos),
                                    (input::Key::Right, false) => cursor_idx
                                        .next(line_infos),

                                    // Up/Down movement
                                    _ => cursor_xy_at(cursor_idx, &cached_text, &state.line_infos, font).and_then(
                                        |(x_pos, _)| {
                                            let text::cursor::Index { line, .. } = cursor_idx;
                                            let next_line = match key {
                                                input::Key::Up => line.saturating_sub(1),
                                                input::Key::Down => line + 1,
                                                _ => unreachable!(),
                                            };
                                            closest_cursor_index_on_line(x_pos, next_line, &cached_text, &state.line_infos, font)
                                        })
                                }.unwrap_or(cursor_idx)
                            };

                            if select {
                                // Expand the selection (or create a new one)
                                cursor = Cursor::Selection {
                                    start: old_selection_start,
                                    end: new_cursor_idx,
                                };
                            } else {
                                match cursor {
                                    Cursor::Idx(_) => {
                                        cursor = Cursor::Idx(new_cursor_idx);
                                    },
                                    Cursor::Selection { start, end } => {
                                        // Move the cursor to the start/end of the current selection.
                                        let new_cursor_idx = {
                                            let cursor_idx = match key {
                                                input::Key::Left | input::Key::Up =>
                                                    std::cmp::min(start, end),
                                                input::Key::Right | input::Key::Down =>
                                                    std::cmp::max(start, end),
                                                _ => unreachable!(),
                                            };

                                            if !move_word {
                                                cursor_idx
                                            } else {
                                                // Move by word from the beginning or end of selection
                                                let line_infos = state.line_infos.iter().cloned();
                                                match key {
                                                    input::Key::Left | input::Key::Up => {
                                                        cursor_idx.previous_word_start(&cached_text, line_infos)
                                                    },
                                                    input::Key::Right | input::Key::Down => {
                                                        cursor_idx.next_word_end(&cached_text, line_infos)
                                                    }
                                                    _ => unreachable!(),
                                                }.unwrap_or(cursor_idx)
                                            }
                                        };
                                        cursor = Cursor::Idx(new_cursor_idx);
                                    },
                                }
                            }
                        },

                        input::Key::A => {
                            // Select all text on Ctrl+a.
                            if press.modifiers.contains(input::keyboard::ModifierKey::CTRL) {
                                let start = text::cursor::Index { line: 0, char: 0 };
                                let end = {
                                    let line_infos = state.line_infos.iter().cloned();
                                    text::cursor::index_before_char(line_infos, cached_text.chars().count())
                                        .expect("char index was out of range")
                                };
                                cursor = Cursor::Selection { start: start, end: end };
                            }
                        },

                        input::Key::E => {
                            // move cursor to end.
                            if press.modifiers.contains(input::keyboard::ModifierKey::CTRL) {
                                let mut line_infos = state.line_infos.iter().cloned();
                                let line = line_infos.len() - 1;
                                match line_infos.nth(line) {
                                    Some(line_info) => {
                                        let char = line_info.end_char() - line_info.start_char;
                                        let new_cursor_idx = text::cursor::Index { line: line, char: char };
                                        cursor = Cursor::Idx(new_cursor_idx);
                                    },
                                    _ => (),
                                }
                            }
                        },

                        input::Key::End => { // move cursor to end.
                            let mut line_infos = state.line_infos.iter().cloned();
                            let line = match cursor {
                                Cursor::Idx(idx) => idx.line,
                                Cursor::Selection {end, ..} => end.line, // use last line of selection
                            };
                            if let Some(line_info) = line_infos.nth(line) {
                                let char = line_info.end_char() - line_info.start_char;
                                let new_cursor_idx = text::cursor::Index { line: line, char: char };
                                cursor = Cursor::Idx(new_cursor_idx);
                            }
                        },

                        input::Key::Home => { // move cursor to beginning.
                            let mut line_infos = state.line_infos.iter().cloned();
                            let line = match cursor {
                                Cursor::Idx(idx) => idx.line,
                                Cursor::Selection {start, ..} => start.line, // use first line of selection
                            };
                            if line_infos.nth(line).is_some() {
                                let char = 0;
                                let new_cursor_idx = text::cursor::Index { line: line, char: char };
                                cursor = Cursor::Idx(new_cursor_idx);
                            }
                        },

                        input::Key::Return => {
                            let font = ui.fonts.get(font_id).unwrap();
                            match insert_text(
                                "\n",
                                cursor,
                                &cached_text,
                                &state.line_infos,
                                font,
                                &mut event_transform,
                            ) {
                                Some((event, None)) => events.push(event),
                                Some((event, Some((new_cursor, new_line_infos)))) => {
                                    event.apply(cached_text.to_mut());
                                    cursor = new_cursor;
                                    state.update(|state| state.line_infos = new_line_infos);
                                    events.push(event);
                                }
                                _ => (),
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
                    if modifiers.contains(input::keyboard::ModifierKey::CTRL)
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

                    let font = ui.fonts.get(font_id).unwrap();
                    match insert_text(
                        &string,
                        cursor,
                        &cached_text,
                        &state.line_infos,
                        font,
                        &mut event_transform,
                    ) {
                        Some((event, None)) => events.push(event),
                        Some((event, Some((new_cursor, new_line_infos)))) => {
                            event.apply(cached_text.to_mut());
                            cursor = new_cursor;
                            state.update(|state| state.line_infos = new_line_infos);
                            events.push(event);
                        }
                        _ => (),
                    }
                },

                // Check whether or not we need to extend a text selection or drag some text.
                event::Widget::Drag(drag_event) if drag_event.button == input::MouseButton::Left => {
                    match drag {

                        Some(Drag::Selecting) => {
                            let start_cursor_idx = match cursor {
                                Cursor::Idx(idx) => idx,
                                Cursor::Selection { start, .. } => start,
                            };
                            let abs_xy = utils::vec2_add(drag_event.to, rect.xy());
                            let infos = &state.line_infos;
                            let font = ui.fonts.get(font_id).unwrap();
                            match closest_cursor_index_and_xy(abs_xy, &cached_text, infos, font) {
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
                },

                _ => (),
            }
        }

        if let Some(_) = ui.widget_input(id).mouse() {
            ui.set_mouse_cursor(cursor::MouseCursor::Text);
        }

        // TODO; this is part of displaying the original text, rather than the cached_text.
        // See comments below (after `match line_wrap {`)
        let original_cursor = state.cursor;

        let display_cursor = original_cursor;
        let display_text = &original_text;

        let cursor_has_changed = state.cursor != cursor;
        if cursor_has_changed {
            state.update(|state| state.cursor = cursor);
        }

        if state.drag != drag {
            state.update(|state| state.drag = drag);
        }

        let color = style.color(ui.theme());
        let font_size = style.font_size(ui.theme());
        let num_lines = state.line_infos.iter().count();
        let text_height = text::height(num_lines, font_size, line_spacing);
        let text_y_range = Range::new(0.0, text_height).align_to(y_align, rect.y);
        let text_rect = Rect { x: rect.x, y: text_y_range };

        match line_wrap {
            // We could display cached_text here, but that would undermine password
            // widgets which want to hide what is being typed.
            //
            // Instead, we just display the original text that was given, and we can
            // display the updates on the next render.
            //
            // TODO: should we make it a configuration variable whether to display cached_text?
            //
            Wrap::Whitespace => widget::Text::new(&display_text).wrap_by_word(),
            Wrap::Character => widget::Text::new(&display_text).wrap_by_character(),
        }
            .font_id(font_id)
            .wh(text_rect.dim())
            .xy(text_rect.xy())
            .justify(justify)
            .parent(id)
            .graphics_for(id)
            .color(color)
            .line_spacing(line_spacing)
            .font_size(font_size)
            .set(state.ids.text, ui);

        // Draw the line for the cursor.
        let cursor_idx = match display_cursor {
            Cursor::Idx(idx) => idx,
            Cursor::Selection { end, .. } => end,
        };

        // If this widget is not capturing the keyboard, no need to draw cursor or selection.
        if ui.global_input().current.widget_capturing_keyboard != Some(id) {
            return events;
        }

        let (cursor_x, cursor_y_range) = {
            let font = ui.fonts.get(font_id).unwrap();
            cursor_xy_at(cursor_idx, &display_text, &state.line_infos, font)
                .unwrap_or_else(|| {
                    let x = rect.left();
                    let y = Range::new(0.0, font_size as Scalar).align_to(y_align, rect.y);
                    (x, y)
                })
        };

        let start = [0.0, cursor_y_range.start];
        let end = [0.0, cursor_y_range.end];
        let prev_cursor_rect = ui.rect_of(state.ids.cursor);
        widget::Line::centred(start, end)
            .x_y(cursor_x, cursor_y_range.middle())
            .graphics_for(id)
            .parent(id)
            .color(color)
            .set(state.ids.cursor, ui);

        // If the cursor position has changed due to input AND one of our parent widgets are
        // scrollable AND the change in cursor position would cause the cursor to fall outside the
        // scrollable parent's `Rect`, attempt to scroll the scrollable parent so that the cursor
        // would be visible.
        if cursor_has_changed {
            let cursor_rect = ui.rect_of(state.ids.cursor).unwrap();
            if prev_cursor_rect != Some(cursor_rect) {
                use graph::Walker;
                let mut scrollable_parents = ui.widget_graph().scrollable_y_parent_recursion(id);
                if let Some(parent_id) = scrollable_parents.next_node(ui.widget_graph()) {
                    if let Some(parent_rect) = ui.rect_of(parent_id) {
                        // If cursor is below, scroll down.
                        if cursor_rect.bottom() < parent_rect.bottom() {
                            let distance = parent_rect.bottom() - cursor_rect.bottom();
                            ui.scroll_widget(parent_id, [0.0, distance]);
                        // If cursor is above, scroll up.
                        } else if cursor_rect.top() > parent_rect.top() {
                            let distance = cursor_rect.top() - parent_rect.top();
                            ui.scroll_widget(parent_id, [0.0, -distance]);
                        }
                    }
                }
            }
        }

        if let Cursor::Selection { start, end } = display_cursor {
            let (start, end) = (std::cmp::min(start, end), std::cmp::max(start, end));

            let selected_rects: Vec<Rect> = {
                let line_infos = state.line_infos.iter().cloned();
                let lines = line_infos.clone().map(|info| &display_text[info.byte_range()]);
                let line_rects = text::line::rects(line_infos.clone(), font_size, rect,
                                                   justify, y_align, line_spacing);
                let lines_with_rects = lines.zip(line_rects.clone());
                let font = ui.fonts.get(font_id).unwrap();
                text::line::selected_rects(lines_with_rects, font, font_size, start, end).collect()
            };

            // Ensure we have at least as many widgets as selected_rectangles.
            if state.ids.selected_rectangles.len() < selected_rects.len() {
                let num_rects = selected_rects.len();
                let id_gen = &mut ui.widget_id_generator();
                state.update(|state| state.ids.selected_rectangles.resize(num_rects, id_gen));
            }

            // Draw a semi-transparent `Rectangle` for the selected range across each line.
            let selected_rect_color = color.highlighted().alpha(0.25);
            let iter = state.ids.selected_rectangles.iter().zip(&selected_rects);
            for (&selected_rectangle_id, selected_rect) in iter {
                widget::Rectangle::fill(selected_rect.dim())
                    .xy(selected_rect.xy())
                    .color(selected_rect_color)
                    .graphics_for(id)
                    .parent(id)
                    .set(selected_rectangle_id, ui);
            }
        }

        events
    }
}

impl<'a, T> Colorable for TextEdit<'a, T> {
    builder_method!(color { style.color = Some(Color) });
}
