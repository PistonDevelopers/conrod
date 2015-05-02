
use clock_ticks::precise_time_s;
use color::{Color, Colorable};
use frame::Frameable;
use graphics::character::CharacterCache;
use label::{self, FontSize};
use mouse::Mouse;
use num::Float;
use piston::input::keyboard::Key::{Backspace, Left, Right, Return};
use position::{self, Depth, Dimensions, HorizontalAlign, Point, Position, VerticalAlign};
use ui::{UiId, Ui};
use vecmath::vec2_sub;
use widget::Kind;

pub type Idx = usize;
pub type CursorX = f64;

/// Represents the state of the text_box widget.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum State {
    Capturing(View),
    Uncaptured(Uncaptured),
}

impl State {
    /// Return the color associated with the current state.
    fn color(&self, color: Color) -> Color {
        match *self {
            State::Capturing(_) => color,
            State::Uncaptured(state) => match state {
                Uncaptured::Highlighted => color.highlighted(),
                Uncaptured::Normal => color,
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct View {
    cursor: Cursor,
    offset: f64,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Cursor {
    anchor: Anchor,
    start: Idx,
    end: Idx,
}

impl Cursor {
    fn from_index(idx: Idx) -> Cursor {
        Cursor { anchor: Anchor::Start, start: idx, end: idx }
    }

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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Anchor {
    None,
    Start,
    End,
}

/// The TextBox's state if it is uncaptured.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Uncaptured {
    Highlighted,
    Normal,
}

/// Represents an element of the TextBox widget.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Element {
    Char(Idx),
    Nill,
    Rect,
}

widget_fns!(TextBox, State, Kind::TextBox(State::Uncaptured(Uncaptured::Normal)));

static TEXT_PADDING: f64 = 5.0;

/// Find the position of a character in a text box.
fn cursor_position<C: CharacterCache>(ui: &mut Ui<C>,
                                      idx: usize,
                                      mut text_start_x: f64,
                                      font_size: FontSize,
                                      text: &str) -> CursorX {
    assert!(idx <= text.len());
    if idx == 0 {
         return text_start_x;
    }

    for (i, ch) in text.chars().enumerate() {
        if i >= idx { break; }
        text_start_x += ui.get_character(font_size, ch).width();
    }
    text_start_x
}

/// Check if cursor is over the pad and if so, which
fn over_elem<C: CharacterCache>(ui: &mut Ui<C>,
                                mouse_xy: Point,
                                dim: Dimensions,
                                pad_dim: Dimensions,
                                text_start_x: f64,
                                text_w: f64,
                                font_size: FontSize,
                                text: &str) -> Element {
    use utils::is_over_rect;
    if is_over_rect([0.0, 0.0], mouse_xy, dim) {
        if is_over_rect([0.0, 0.0], mouse_xy, pad_dim) {
            let (idx, _) = closest_idx(ui, mouse_xy, text_start_x, text_w, font_size, text);
            Element::Char(idx)
        } else {
            Element::Rect
        }
    } else {
        Element::Nill
    }
}

/// Check which character is closest to the mouse cursor.
fn closest_idx<C: CharacterCache>(ui: &mut Ui<C>,
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
        let character = ui.get_character(font_size, ch);
        let char_w = character.width();
        x += char_w;
        let right_x = prev_x + char_w / 2.0;
        if mouse_xy[0] > left_x && mouse_xy[0] <= right_x { return (i, prev_x) }
        prev_x = x;
        left_x = right_x;
    }
    (text.len(), text_start_x + text_w)
}

/// Check and return the current state of the TextBox.
fn get_new_state(over_elem: Element, prev_state: State, mouse: Mouse) -> State {
    use mouse::ButtonState::{ Down, Up };
    use self::State::{ Capturing, Uncaptured };
    use self::Uncaptured::{ Normal, Highlighted };

    match prev_state {
        State::Capturing(mut prev) => match mouse.left {
            Down => match over_elem {
                Element::Nill => if prev.cursor.anchor == Anchor::None {
                    Uncaptured(Normal)
                } else {
                    prev_state
                },
                Element::Rect =>  if prev.cursor.anchor == Anchor::None {
                    prev.cursor = Cursor::from_index(0);
                    Capturing(prev)
                } else {
                    prev_state
                },
                Element::Char(idx) => {
                    match prev.cursor.anchor {
                        Anchor::None  => prev.cursor = Cursor::from_index(idx),
                        Anchor::Start => prev.cursor = Cursor::from_range(prev.cursor.start, idx),
                        Anchor::End   => prev.cursor = Cursor::from_range(prev.cursor.end, idx),
                    }
                    Capturing(prev)
                },
            },
            Up => {
                prev.cursor.anchor = Anchor::None;
                Capturing(prev)
            },
        },

        State::Uncaptured(prev) => match mouse.left {
            Down => match over_elem {
                Element::Nill => Uncaptured(Normal),
                Element::Rect => match prev {
                    Normal => prev_state,
                    Highlighted => Capturing(View {
                         cursor: Cursor::from_index(0),
                         offset: 0.0,
                    })
                },
                Element::Char(idx) =>  match prev {
                    Normal => prev_state,
                    Highlighted => Capturing(View {
                        cursor: Cursor::from_index(idx),
                        offset: 0.0,
                    })
                },
            },
            Up => match over_elem {
                Element::Nill => Uncaptured(Normal),
                Element::Char(_) | Element::Rect => match prev {
                    Normal => Uncaptured(Highlighted),
                    Highlighted => prev_state,
                },
            },
        },
    }
}

/// A widget for displaying and mutating a given one-line text `String`. It's reaction is
/// triggered upon pressing of the `Enter`/`Return` key.
pub struct TextBox<'a, F> {
    text: &'a mut String,
    font_size: u32,
    pos: Position,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    dim: Dimensions,
    depth: Depth,
    maybe_react: Option<F>,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
}

impl<'a, F> TextBox<'a, F> {

    /// Construct a TextBox widget.
    pub fn new(text: &'a mut String) -> TextBox<'a, F> {
        TextBox {
            text: text,
            font_size: 24, // Default font_size.
            pos: Position::default(),
            maybe_h_align: None,
            maybe_v_align: None,
            dim: [192.0, 48.0],
            depth: 0.0,
            maybe_react: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
        }
    }

    /// Set the font size of the text.
    pub fn font_size(self, font_size: FontSize) -> TextBox<'a, F> {
        TextBox { font_size: font_size, ..self }
    }

    /// Set the reaction for the TextBox. It will be triggered upon pressing of the
    /// `Enter`/`Return` key.
    pub fn react(mut self, reaction: F) -> TextBox<'a, F> {
        self.maybe_react = Some(reaction);
        self
    }

    /// After building the TextBox, use this method to set its current state into the given `Ui`.
    /// It will use this state for rendering the next time `ui.draw(graphics)` is called.
    pub fn set<C>(mut self, ui_id: UiId, ui: &mut Ui<C>)
        where
            C: CharacterCache,
            F: FnMut(&mut String),
    {
        use elmesque::form::{collage, line, rect, solid, text};
        use elmesque::text::Text;

        let dim = self.dim;
        let h_align = self.maybe_h_align.unwrap_or(ui.theme.align.horizontal);
        let v_align = self.maybe_v_align.unwrap_or(ui.theme.align.vertical);
        let xy = ui.get_xy(self.pos, dim, h_align, v_align);
        let mouse = ui.get_mouse_state(ui_id).relative_to(xy);
        let state = *get_state(ui, ui_id);
        let frame_w = self.maybe_frame.unwrap_or(ui.theme.frame_width);
        let frame_w2 = frame_w * 2.0;
        let font_size = self.font_size;
        let pad_dim = vec2_sub(dim, [frame_w2; 2]);
        let half_pad_h = pad_dim[1] / 2.0;
        let text_w = label::width(ui, font_size, &self.text);
        let text_x = position::align_left_of(pad_dim[0], text_w) + TEXT_PADDING;
        let text_start_x = text_x - text_w / 2.0;
        let over_elem = over_elem(ui, mouse.xy, dim, pad_dim, text_start_x, text_w, font_size, &self.text);
        let mut new_state = get_new_state(over_elem, state, mouse);

        // Construct the frame and inner rectangle Forms.
        let color = new_state.color(self.maybe_color.unwrap_or(ui.theme.shape_color));
        let frame_color = self.maybe_frame_color.unwrap_or(ui.theme.frame_color);
        let frame_form = rect(dim[0], dim[1]).filled(frame_color);
        let inner_form = rect(pad_dim[0], pad_dim[1]).filled(color);

        // - Check cursor validity (and update new_state if necessary).
        // - Construct the cursor and text `Form`s.
        let (maybe_cursor_form, text_form) = if let State::Capturing(view) = new_state {
            let mut cursor = view.cursor;
            let mut v_offset = view.offset;

            // Ensure the cursor is still valid.
            cursor.limit_end_to(self.text.len());

            // This matters if the text is scrolled with the mouse.
            let cursor_idx = match cursor.anchor {
                Anchor::End => cursor.start,
                Anchor::Start | Anchor::None => cursor.end,
            };
            let cursor_x = cursor_position(ui, cursor_idx, text_start_x, self.font_size, &self.text);

            if cursor.is_cursor() || cursor.anchor != Anchor::None {
                let cursor_x_view = cursor_x - v_offset;
                let text_right = self.dim[0] - TEXT_PADDING - frame_w;

                if cursor_x_view < text_x {
                    v_offset += cursor_x_view - text_x;
                } else if cursor_x_view > text_right {
                    v_offset += cursor_x_view - text_right;
                }
            }

            // Set the updated state.
            new_state = State::Capturing(View { cursor: cursor, offset: v_offset });

            // Construct the Cursor's Form.
            let cursor_alpha = (precise_time_s() * 2.5).sin() as f32 * 0.5 + 0.5;
            let cursor_form = if cursor.is_cursor() {
                line(solid(color.plain_contrast()), 0.0, half_pad_h, 0.0, -half_pad_h)
                    .alpha(cursor_alpha)
                    .shift_x(cursor_x)
            } else {
                let (block_xy, dim) = {
                    let (start, end) = (cursor.start, cursor.end);
                    let cursor_x = cursor_position(ui, start, text_start_x, font_size, &self.text);
                    let htext: String = self.text.chars().skip(start).take(end - start).collect();
                    let htext_w = label::width(ui, font_size, &htext);
                    ([cursor_x + htext_w / 2.0, 0.0], [htext_w, dim[1]])
                };
                rect(dim[0], dim[1] - frame_w2).filled(color.highlighted())
                    .shift(block_xy[0], block_xy[1])
            };

            // Construct the text's Form.
            let text_form = text(Text::from_string(self.text.clone())
                                     .color(color.plain_contrast())
                                     .height(font_size as f64)).shift_x(text_x.floor());

            (Some(cursor_form), text_form)
        } else {
            // Construct the text's Form.
            let text_form = text(Text::from_string(self.text.clone())
                                     .color(color.plain_contrast())
                                     .height(font_size as f64)).shift_x(text_x.floor());
            (None, text_form)
        };

        // If TextBox is captured, check for recent input and update the text accordingly.
        if let State::Capturing(captured) = new_state {
            let mut cursor = captured.cursor;

            // Check for entered text.
            for text in ui.get_entered_text(ui_id).to_vec().iter() {
                if text.len() == 0 { continue; }

                let max_w = pad_dim[0] - TEXT_PADDING * 2.0;
                if text_w + label::width(ui, font_size, &text) > max_w { continue; }

                let end: String = self.text.chars().skip(cursor.end).collect();
                self.text.truncate(cursor.start);
                self.text.push_str(&text);
                self.text.push_str(&end);
                cursor.shift(text.len() as i32);
            }

            // Check for control keys.
            let pressed_keys = ui.get_pressed_keys(ui_id);
            for key in pressed_keys.iter() {
                match *key {
                    Backspace => if cursor.is_cursor() {
                        if cursor.start > 0 {
                            let end: String = self.text.chars().skip(cursor.end).collect();
                            self.text.truncate(cursor.start - 1);
                            self.text.push_str(&end);
                            cursor.shift(-1);
                        }
                    } else {
                        let end: String = self.text.chars().skip(cursor.end).collect();
                        self.text.truncate(cursor.start);
                        self.text.push_str(&end);
                        cursor.end = cursor.start;
                    },
                    Left => if cursor.is_cursor() {
                        cursor.shift(-1);
                    },
                    Right => if cursor.is_cursor() && self.text.len() > cursor.end {
                        cursor.shift(1);
                    },
                    Return => if self.text.len() > 0 {
                        let TextBox { ref mut maybe_react, ref mut text, .. } = self;
                        if let Some(ref mut react) = *maybe_react {
                            react(*text);
                        }
                    },
                    _ => (),
                }
            }

            new_state = State::Capturing(View { cursor: cursor, .. captured });
        }

        // Check whether or not we need to capture or uncapture the keyboard.
        match (state, new_state) {
            (State::Uncaptured(_), State::Capturing(_)) => ui.keyboard_captured_by(ui_id),
            (State::Capturing(_), State::Uncaptured(_)) => ui.keyboard_uncaptured_by(ui_id),
            _ => (),
        }

        // Chain the Forms and shift them into position.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(inner_form).into_iter())
            .chain(maybe_cursor_form.into_iter())
            .chain(Some(text_form).into_iter())
            .map(|form| form.shift(xy[0], xy[1]));

        // Collect the Forms into a renderable `Element`.
        let element = collage(dim[0] as i32, dim[1] as i32, form_chain.collect());

        // Store the TextBox's new state in the Ui.
        ui.update_widget(ui_id, Kind::TextBox(new_state), xy, self.depth, Some(element));

    }

}

impl<'a, F> Colorable for TextBox<'a, F> {
    fn color(mut self, color: Color) -> Self {
        self.maybe_color = Some(color);
        self
    }
}

impl<'a, F> Frameable for TextBox<'a, F> {
    fn frame(mut self, width: f64) -> Self {
        self.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, F> position::Positionable for TextBox<'a, F> {
    fn position(mut self, pos: Position) -> Self {
        self.pos = pos;
        self
    }
    #[inline]
    fn horizontal_align(self, h_align: HorizontalAlign) -> Self {
        TextBox { maybe_h_align: Some(h_align), ..self }
    }
    #[inline]
    fn vertical_align(self, v_align: VerticalAlign) -> Self {
        TextBox { maybe_v_align: Some(v_align), ..self }
    }
}

impl<'a, F> position::Sizeable for TextBox<'a, F> {
    #[inline]
    fn width(self, w: f64) -> Self {
        let h = self.dim[1];
        TextBox { dim: [w, h], ..self }
    }
    #[inline]
    fn height(self, h: f64) -> Self {
        let w = self.dim[0];
        TextBox { dim: [w, h], ..self }
    }
}
