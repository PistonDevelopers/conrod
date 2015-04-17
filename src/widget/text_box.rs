
use clock_ticks::precise_time_s;
use color::{Color, Colorable};
use dimensions::Dimensions;
use frame::Frameable;
use graphics::{self, Graphics};
use graphics::character::CharacterCache;
use label::{self, FontSize};
use mouse::Mouse;
use num::Float;
use piston::input::keyboard::Key::{Backspace, Left, Right, Return};
use point::Point;
use position::Positionable;
use rectangle;
use shape::Shapeable;
use ui::{UiId, Ui};
use vecmath::{vec2_add, vec2_sub};
use widget::Kind;

pub type Idx = usize;
pub type CursorX = f64;

/// Represents the state of the text_box widget.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum State {
    Capturing(View),
    Uncaptured(Uncaptured),
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

impl State {
    /// Return the associated Rectangle state.
    fn as_rectangle_state(&self) -> rectangle::State {
        match *self {
            State::Capturing(_) => rectangle::State::Normal,
            State::Uncaptured(state) => match state {
                Uncaptured::Highlighted => rectangle::State::Highlighted,
                Uncaptured::Normal => rectangle::State::Normal,
            }
        }
    }
}

widget_fns!(TextBox, State, Kind::TextBox(State::Uncaptured(Uncaptured::Normal)));

static TEXT_PADDING: f64 = 5f64;

/// Find the position of a character in a text box.
fn cursor_position<C: CharacterCache>(ui: &mut Ui<C>,
                 idx: usize,
                 mut text_x: f64,
                 font_size: FontSize,
                 text: &str) -> CursorX {
    assert!(idx <= text.len());
    if idx == 0 {
         return text_x;
    }

    for (i, ch) in text.chars().enumerate() {
        if i >= idx { break; }
        text_x += ui.get_character(font_size, ch).width();
    }
    text_x
}

/// Check if cursor is over the pad and if so, which
fn over_elem<C: CharacterCache>(ui: &mut Ui<C>,
             pos: Point,
             mouse_pos: Point,
             rect_dim: Dimensions,
             pad_pos: Point,
             pad_dim: Dimensions,
             text_pos: Point,
             text_w: f64,
             font_size: FontSize,
             text: &str) -> Element {
    match rectangle::is_over(pos, mouse_pos, rect_dim) {
        false => Element::Nill,
        true => match rectangle::is_over(pad_pos, mouse_pos, pad_dim) {
            false => Element::Rect,
            true => {
                let (idx, _) = closest_idx(ui, mouse_pos, text_pos[0], text_w, font_size, text);
                Element::Char(idx)
            },
        },
    }
}

/// Check which character is closest to the mouse cursor.
fn closest_idx<C: CharacterCache>(ui: &mut Ui<C>,
               mouse_pos: Point,
               text_x: f64,
               text_w: f64,
               font_size: FontSize,
               text: &str) -> (Idx, f64) {
    if mouse_pos[0] <= text_x { return (0, text_x) }
    let mut x = text_x;
    let mut prev_x = x;
    let mut left_x = text_x;
    for (i, ch) in text.chars().enumerate() {
        let character = ui.get_character(font_size, ch);
        let char_w = character.width();
        x += char_w;
        let right_x = prev_x + char_w / 2.0;
        if mouse_pos[0] > left_x && mouse_pos[0] <= right_x { return (i, prev_x) }
        prev_x = x;
        left_x = right_x;
    }
    (text.len(), text_x + text_w)
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
                        Anchor::None => prev.cursor = Cursor::from_index(idx),
                        Anchor::Start => prev.cursor = Cursor::from_range(prev.cursor.start, idx),
                        Anchor::End => prev.cursor = Cursor::from_range(prev.cursor.end, idx),
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

/// Draw the text cursor.
fn draw_cursor<B: Graphics>(
    win_w: f64,
    win_h: f64,
    graphics: &mut B,
    color: Color,
    cursor_x: f64,
    pad_pos_y: f64,
    pad_h: f64
) {
    use color::Rgba;
    let draw_state = graphics::default_draw_state();
    let transform = graphics::abs_transform(win_w, win_h);
    let Rgba(r, g, b, a) = color.plain_contrast().to_rgb();
    graphics::Line::new_round([r, g, b, (a * (precise_time_s() * 2.5).sin() as f32).abs()], 0.5f64)
        .draw(
            [cursor_x, pad_pos_y, cursor_x, pad_pos_y + pad_h],
            draw_state,
            transform,
            graphics
        );
}

/// A widget for displaying and mutating a given one-line text `String`. It's callback is
/// triggered upon pressing of the `Enter`/`Return` key.
pub struct TextBox<'a, F> {
    ui_id: UiId,
    text: &'a mut String,
    font_size: u32,
    pos: Point,
    dim: Dimensions,
    maybe_callback: Option<F>,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
}

impl<'a, F> TextBox<'a, F> {

    /// Construct a TextBox widget.
    pub fn new(ui_id: UiId, text: &'a mut String) -> TextBox<'a, F> {
        TextBox {
            ui_id: ui_id,
            text: text,
            font_size: 24, // Default font_size.
            pos: [0.0, 0.0],
            dim: [192.0, 48.0],
            maybe_callback: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
        }
    }

    /// Set the font size of the text.
    pub fn font_size(self, font_size: FontSize) -> TextBox<'a, F> {
        TextBox { font_size: font_size, ..self }
    }

    /// Set the callback for the TextBox. It will be triggered upon pressing of the
    /// `Enter`/`Return` key.
    pub fn callback(mut self, cb: F) -> TextBox<'a, F> {
        self.maybe_callback = Some(cb);
        self
    }

    fn cursor_rect<C: CharacterCache>
                     (&self, ui: &mut Ui<C>, text_x: f64, start: Idx, end: Idx) ->
                     (Point, Dimensions) {
        let pos = cursor_position(ui, start, text_x, self.font_size, &self.text);
        let htext: String = self.text.chars().skip(start).take(end - start).collect();
        let htext_w = label::width(ui, self.font_size, &htext);
        ([pos, self.pos[1]], [htext_w, self.dim[1]])
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

impl<'a, F> Positionable for TextBox<'a, F> {
    fn point(mut self, pos: Point) -> Self {
        self.pos = pos;
        self
    }
}

impl<'a, F> Shapeable for TextBox<'a, F> {
    fn get_dim(&self) -> Dimensions { self.dim }
    fn dim(mut self, dim: Dimensions) -> Self { self.dim = dim; self }
}

impl<'a, F> ::draw::Drawable for TextBox<'a, F>
    where
        F: FnMut(&mut String) + 'a
{

    #[inline]
    fn draw<B, C>(&mut self, ui: &mut Ui<C>, graphics: &mut B)
        where
            B: Graphics<Texture = <C as CharacterCache>::Texture>,
            C: CharacterCache
    {
        let mouse = ui.get_mouse_state();
        let state = *get_state(ui, self.ui_id);

        // Rect.
        let color = self.maybe_color.unwrap_or(ui.theme.shape_color);
        let frame_w = self.maybe_frame.unwrap_or(ui.theme.frame_width);
        let frame_w2 = frame_w * 2.0;
        let maybe_frame = match frame_w > 0.0 {
            true => Some((frame_w, self.maybe_frame_color.unwrap_or(ui.theme.frame_color))),
            false => None,
        };
        let pad_pos = vec2_add(self.pos, [frame_w; 2]);
        let pad_dim = vec2_sub(self.dim, [frame_w2; 2]);
        let text_x = pad_pos[0] + TEXT_PADDING;
        let text_y = pad_pos[1] + (pad_dim[1] - self.font_size as f64) / 2.0;
        let text_pos = [text_x, text_y];
        let text_w = label::width(ui, self.font_size, &self.text);
        let over_elem = over_elem(ui, self.pos, mouse.pos, self.dim,
                                  pad_pos, pad_dim, text_pos, text_w,
                                  self.font_size, &self.text);
        let mut new_state = get_new_state(over_elem, state, mouse);

        rectangle::draw(ui.win_w, ui.win_h, graphics, new_state.as_rectangle_state(),
                        self.pos, self.dim, maybe_frame, color);

        if let State::Capturing(captured) = new_state {
            let mut cursor = captured.cursor;

            // Ensure the cursor is still valid.
            if cursor.limit_end_to(self.text.len()) {
                new_state = State::Capturing(View { cursor: cursor, .. captured });
            }

            // Draw the cursor.
            if cursor.is_cursor() {
                let cursor_x = cursor_position(ui, cursor.start, text_x,
                                               self.font_size, &self.text);
                draw_cursor(ui.win_w, ui.win_h, graphics, color, cursor_x, pad_pos[1], pad_dim[1]);
            } else {
                let (pos, dim) = self.cursor_rect(ui, text_x, cursor.start, cursor.end);
                rectangle::draw(ui.win_w, ui.win_h, graphics, new_state.as_rectangle_state(),
                                [pos[0], pos[1] + frame_w], [dim[0], dim[1] - frame_w2],
                                None, color.highlighted());
            }
        }

        ui.draw_text(graphics, text_pos, self.font_size, color.plain_contrast(), &self.text);

        if let State::Capturing(captured) = new_state {
            let mut cursor = captured.cursor;

            // Check for entered text.
            for text in ui.get_entered_text().iter() {
                if text.len() == 0 { continue; }

                let end: String = self.text.chars().skip(cursor.end).collect();
                self.text.truncate(cursor.start);
                self.text.push_str(&text);
                self.text.push_str(&end);
                cursor.shift(text.len() as i32);
            }

            // Check for control keys.
            let pressed_keys = ui.get_pressed_keys();
            for key in pressed_keys.iter() {
                match *key {
                    Backspace => {
                        if cursor.is_cursor() {
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
                        }
                    },
                    Left => {
                        if cursor.is_cursor() {
                            cursor.shift(-1);
                        }
                    },
                    Right => {
                        if cursor.is_cursor() && self.text.len() > cursor.end {
                            cursor.shift(1);
                        }
                    },
                    Return => if self.text.len() > 0 {
                        let TextBox { // borrowck
                            ref mut maybe_callback,
                            ref mut text,
                            ..
                        } = *self;
                        match *maybe_callback {
                            Some(ref mut callback) => (*callback)(*text),
                            None => (),
                        }
                    },
                    _ => (),
                }
            }
            new_state = State::Capturing(View { cursor: cursor, .. captured });
        }
        set_state(ui, self.ui_id, Kind::TextBox(new_state), self.pos, self.dim);
    }
}
