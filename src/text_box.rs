use color::Color;
use dimensions::Dimensions;
use graphics;
use graphics::{
    Graphics,
};
use graphics::character::CharacterCache;
use label;
use label::FontSize;
use mouse::Mouse;
use piston::input::keyboard::Key::{
    Backspace,
    Left,
    Right,
    Return,
};
use point::Point;
use rectangle;
use std::num::Float;
use clock_ticks::precise_time_s;
use ui_context::{
    Id,
    UIID,
    UiContext,
};
use vecmath::{
    vec2_add,
    vec2_sub,
};
use widget::{ DefaultWidgetState, Widget };
use std::cmp;
use Callback;
use FrameColor;
use FrameWidth;
use Position;
use Size;

pub type Idx = usize;
pub type CursorX = f64;

/// Represents the state of the text_box widget.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum State {
    Capturing(Selection),
    Uncaptured(Uncaptured),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Selection {
    anchor: Anchor,
    start: Idx,
    end: Idx,
}

impl Selection {
    fn from_index(idx: Idx) -> Selection {
        Selection {anchor: Anchor::Start, start: idx, end: idx }
    }

    fn from_range(start: Idx, end: Idx) -> Selection {
        if start < end {
            Selection { anchor: Anchor::Start, start: start, end: end }
        } else {
            Selection { anchor: Anchor::End, start: end, end: start }
        }
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

widget_fns!(TextBox, State, Widget::TextBox(State::Uncaptured(Uncaptured::Normal)));

static TEXT_PADDING: f64 = 5f64;

/// Find the position of a character in a text box.
fn cursor_position<C: CharacterCache>(uic: &mut UiContext<C>,
                 mut idx: usize,
                 mut text_x: f64,
                 font_size: FontSize,
                 text: &str) -> (Idx, CursorX) {
    if idx == 0 { return (0, text_x); }
    let text_len = text.len();
    if idx > text_len { idx = text_len; }
    for (i, ch) in text.chars().enumerate() {
        if i >= idx { break; }
        text_x += uic.get_character(font_size, ch).width();
    }
    (idx, text_x)
}

/// Check if cursor is over the pad and if so, which
fn over_elem<C: CharacterCache>(uic: &mut UiContext<C>,
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
                let (idx, _) = closest_idx(uic, mouse_pos, text_pos[0], text_w, font_size, text);
                Element::Char(idx)
            },
        },
    }
}

/// Check which character is closest to the mouse cursor.
fn closest_idx<C: CharacterCache>(uic: &mut UiContext<C>,
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
        let character = uic.get_character(font_size, ch);
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
        State::Capturing(prev) => match mouse.left {
            Down => match over_elem {
                Element::Nill => if prev.anchor == Anchor::None {
                    Uncaptured(Normal)
                } else {
                    prev_state
                },
                Element::Rect =>  if prev.anchor == Anchor::None {
                    Capturing(Selection::from_index(0))
                } else {
                    prev_state
                },
                Element::Char(idx) => match prev.anchor {
                    Anchor::None => Capturing(Selection::from_index(idx)),
                    Anchor::Start => Capturing(Selection::from_range(prev.start, idx)),
                    Anchor::End => Capturing(Selection::from_range(prev.end, idx)),
                },
            },
            Up => Capturing(Selection { anchor: Anchor::None, .. prev })
        },

        State::Uncaptured(prev) => match mouse.left {
            Down => match over_elem {
                Element::Nill => Uncaptured(Normal),
                Element::Rect => match prev {
                    Normal => prev_state,
                    Highlighted => Capturing(Selection::from_index(0)),
                },
                Element::Char(idx) =>  match prev {
                    Normal => prev_state,
                    Highlighted => Capturing(Selection::from_index(idx)),
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
    let draw_state = graphics::default_draw_state();
    let transform = graphics::abs_transform(win_w, win_h);
    let Color(color) = color.plain_contrast();
    let (r, g, b, a) = (color[0], color[1], color[2], color[3]);
    graphics::Line::new_round([r, g, b, (a * (precise_time_s() * 2.5).sin() as f32).abs()], 0.5f64)
        .draw(
            [cursor_x, pad_pos_y, cursor_x, pad_pos_y + pad_h],
            draw_state,
            transform,
            graphics
        );
}

/// A context on which the builder pattern can be implemented.
pub struct TextBox<'a, F> {
    ui_id: UIID,
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
    pub fn font_size(self, font_size: FontSize) -> TextBox<'a, F> {
        TextBox { font_size: font_size, ..self }
    }
}

impl<'a, F> TextBox<'a, F> {
    /// Initialise a TextBoxContext.
    pub fn new(ui_id: UIID, text: &'a mut String) -> TextBox<'a, F> {
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

    fn selection_rect<C: CharacterCache>
                     (&self, uic: &mut UiContext<C>, text_x: f64, start: Idx, end: Idx) ->
                     (Point, Dimensions) {
        let (_, pos) = cursor_position(uic, start, text_x, self.font_size, &self.text);
        let htext: String = self.text.chars().skip(start).take(end - start).collect();
        let htext_w = label::width(uic, self.font_size, &htext);
        ([pos, self.pos[1]], [htext_w, self.dim[1]])
    }
}

/*
quack! {
    tb: TextBox['a, F]
    get:
        fn () -> Size [] { Size(tb.dim) }
        fn () -> DefaultWidgetState [] {
            DefaultWidgetState(
                Widget::TextBox(State::Uncaptured(Uncaptured::Normal))
            )
        }
        fn () -> Id [] { Id(tb.ui_id) }
    set:
        fn (val: Color) [] { tb.maybe_color = Some(val) }
        fn (val: Callback<F>) [where F: FnMut(&mut String) + 'a] {
            tb.maybe_callback = Some(val.0)
        }
        fn (val: FrameColor) [] { tb.maybe_frame_color = Some(val.0) }
        fn (val: FrameWidth) [] { tb.maybe_frame = Some(val.0) }
        fn (val: Position) [] { tb.pos = val.0 }
        fn (val: Size) [] { tb.dim = val.0 }
    action:
}
*/

impl<'a, F> ::draw::Drawable for TextBox<'a, F>
    where
        F: FnMut(&mut String) + 'a
{

    #[inline]
    fn draw<B, C>(&mut self, uic: &mut UiContext<C>, graphics: &mut B)
        where
            B: Graphics<Texture = <C as CharacterCache>::Texture>,
            C: CharacterCache
    {
        let mouse = uic.get_mouse_state();
        let state = *get_state(uic, self.ui_id);

        // Rect.
        let color = self.maybe_color.unwrap_or(uic.theme.shape_color);
        let frame_w = self.maybe_frame.unwrap_or(uic.theme.frame_width);
        let frame_w2 = frame_w * 2.0;
        let maybe_frame = match frame_w > 0.0 {
            true => Some((frame_w, self.maybe_frame_color.unwrap_or(uic.theme.frame_color))),
            false => None,
        };
        let pad_pos = vec2_add(self.pos, [frame_w; 2]);
        let pad_dim = vec2_sub(self.dim, [frame_w2; 2]);
        let text_x = pad_pos[0] + TEXT_PADDING;
        let text_y = pad_pos[1] + (pad_dim[1] - self.font_size as f64) / 2.0;
        let text_pos = [text_x, text_y];
        let text_w = label::width(uic, self.font_size, &self.text);
        let over_elem = over_elem(uic, self.pos, mouse.pos, self.dim,
                                  pad_pos, pad_dim, text_pos, text_w,
                                  self.font_size, &self.text);
        let mut new_state = get_new_state(over_elem, state, mouse);

        rectangle::draw(uic.win_w, uic.win_h, graphics, new_state.as_rectangle_state(),
                        self.pos, self.dim, maybe_frame, color);

        if let State::Capturing(selection) = new_state {
            if selection.start != selection.end {
                let (pos, dim) = self.selection_rect(uic, text_x, selection.start, selection.end);
                rectangle::draw(uic.win_w, uic.win_h, graphics, new_state.as_rectangle_state(),
                                [pos[0], pos[1] + frame_w], [dim[0], dim[1] - frame_w2],
                                None, color.highlighted());
            }
        }

        uic.draw_text(graphics, text_pos, self.font_size, color.plain_contrast(), &self.text);

        if let State::Capturing(selection) = new_state {
            if selection.start == selection.end {
            let (idx, cursor_x) = cursor_position(uic, selection.start, text_x, self.font_size, &self.text);
            draw_cursor(uic.win_w, uic.win_h, graphics, color, cursor_x, pad_pos[1], pad_dim[1]);
            let mut new_idx = idx;
            let mut new_cursor_x = cursor_x;

            // Check for entered text.
            let entered_text = uic.get_entered_text();
            for t in entered_text.iter() {
                let mut entered_text_width = 0.0;
                for ch in t[..].chars() {
                    let c = uic.get_character(self.font_size, ch);
                    entered_text_width += c.width();
                }
                if new_cursor_x + entered_text_width < pad_pos[0] + pad_dim[0] - TEXT_PADDING {
                    new_cursor_x += entered_text_width;
                }
                else {
                    break;
                }
                let new_text = format!("{}{}{}", &self.text[..idx], t, &self.text[idx..]);
                *self.text = new_text;
                new_idx += t.len();
            }

            // Check for control keys.
            let pressed_keys = uic.get_pressed_keys();
            for key in pressed_keys.iter() {
                match *key {
                    Backspace => {
                        if self.text.len() > 0
                        && self.text.len() >= idx
                        && idx > 0 {
                            let rem_idx = idx - 1;
                            new_cursor_x -= uic.get_character_w(
                                self.font_size, self.text[..].char_at(rem_idx)
                            );
                            let new_text = format!("{}{}", &self.text[..rem_idx], &self.text[idx..]);
                            *self.text = new_text;
                            new_idx = rem_idx;
                        }
                    },
                    Left => {
                        if idx > 0 {
                            new_cursor_x -= uic.get_character_w(
                                self.font_size, self.text[..].char_at(idx - 1)
                            );
                            new_idx -= 1;
                        }
                    },
                    Right => {
                        if self.text.len() > idx {
                            new_cursor_x += uic.get_character_w(
                                self.font_size, self.text[..].char_at(idx)
                            );
                            new_idx += 1;
                        }
                    },
                    Return => if self.text.len() > 0 {
                        let TextBox { // borrowck
                            ref mut maybe_callback,
                            ref font_size,
                            ref mut text,
                            ..
                        } = *self;
                        match *maybe_callback {
                            Some(ref mut callback) => {
                                (*callback)(*text);
                                new_idx = cmp::min(new_idx, text.len());
                                let text = &*text;
                                new_cursor_x = text.chars()
                                                   // Add text_pos.x for padding
                                                   .fold(text_pos[0], |acc, c| {
                                    acc + uic.get_character_w(*font_size, c)
                                });
                            },
                            None => (),
                        }
                    },
                    _ => (),
                }
            }
            new_state = State::Capturing(Selection { start: new_idx, end: new_idx, .. selection });
        }}
        set_state(uic, self.ui_id, Widget::TextBox(new_state), self.pos, self.dim);
    }
}
