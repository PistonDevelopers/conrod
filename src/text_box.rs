
use color::Color;
use frame::{
    Framing,
    Frame,
    NoFrame,
};
use graphics::{
    AddColor,
    AddEllipse,
    AddLine,
    AddRoundBorder,
    Context,
    Draw,
};
use label;
use label::{
    NoLabel,
    Label,
    FontSize,
    Labeling,
};
use mouse_state::{
    MouseState,
    Up,
    Down,
};
use opengl_graphics::Gl;
use piston::input::keyboard::{
    Key,
    Backspace,
    Left,
    Right,
    Return,
};
use point::Point;
use rectangle;
use rectangle::{
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
};
use std::num::abs;
use time::precise_time_s;
use ui_context::{
    UIID,
    UIContext,
};
use utils::{
    clamp,
    map_range,
    val_to_string,
};
use widget::TextBox;

pub type Idx = uint;
pub type CursorX = f64;

/// Represents the state of the text_box widget.
#[deriving(Show, PartialEq)]
pub struct State(WidgetState, Capturing);

/// Represents the next tier of state.
#[deriving(Show, PartialEq)]
pub enum WidgetState {
    Normal,
    Highlighted(Element),
    Clicked(Element),
}

/// Whether the textbox is currently captured or not.
#[deriving(Show, PartialEq)]
pub enum Capturing {
    Uncaptured,
    Captured(Idx, CursorX),
}

/// Represents an element of the TextBox widget.
#[deriving(Show, PartialEq)]
pub enum Element {
    Nill,
    Rect,
    Text(Idx, CursorX),
}

impl State {
    /// Return the associated Rectangle state.
    fn as_rectangle_state(&self) -> rectangle::State {
        match self {
            &State(state, capturing) => match capturing {
                Captured(_, _) => rectangle::Normal,
                Uncaptured => match state {
                    Normal => rectangle::Normal,
                    Highlighted(_) => rectangle::Highlighted,
                    Clicked(_) => rectangle::Clicked,
                },
            }
        }
    }
}

widget_fns!(TextBox, State, TextBox(State(Normal, Uncaptured)))

static TEXT_PADDING: f64 = 5f64;

/// Draw the text_box widget.
pub fn draw(gl: &mut Gl,
            uic: &mut UIContext,
            ui_id: UIID,
            pos: Point<f64>,
            width: f64,
            height: f64,
            font_size: FontSize,
            frame: Framing,
            color: Color,
            text: &mut String,
            callback: |&mut String|) {

    let mouse = uic.get_mouse_state();
    let state = *get_state(uic, ui_id);

    // Rect.
    let frame_w = match frame { Frame(w, _) => w, NoFrame => 0.0 };
    let frame_w2 = frame_w * 2.0;
    let pad_pos = pos + Point::new(frame_w, frame_w, 0.0);
    let pad_w = width - frame_w2;
    let pad_h = height - frame_w2;
    let text_pos = Point::new(pad_pos.x + TEXT_PADDING,
                              pad_pos.y + (pad_h - font_size as f64) / 2.0,
                              0.0);
    let text_w = label::width(uic, font_size, text.as_slice());
    let over_elem = over_elem(uic, pos, mouse.pos, width, height,
                              pad_pos, pad_w, pad_h,
                              text_pos, text_w, font_size, text.as_slice());
    let mut new_state = get_new_state(over_elem, state, mouse);

    rectangle::draw(uic.win_w, uic.win_h, gl, new_state.as_rectangle_state(), pos, width, height, frame, color);
    label::draw(gl, uic, text_pos, font_size, color.plain_contrast(), text.as_slice());

    let new_state = match new_state { State(w_state, capturing) => match capturing {
        Uncaptured => new_state,
        Captured(idx, cursor_x) => {
            draw_cursor(uic.win_w, uic.win_h, gl, uic, color, cursor_x, pad_pos.y, pad_h);
            let mut new_idx = idx;
            let mut new_cursor_x = cursor_x;

            // Check for entered text.
            let entered_text = uic.get_entered_text();
            for t in entered_text.iter() {
                let mut entered_text_width = 0f64;
                for ch in t.as_slice().chars() {
                    let c = uic.get_character(font_size, ch);
                    entered_text_width += (c.glyph.advance().x >> 16) as f64;
                }
                if new_cursor_x + entered_text_width < pad_pos.x + pad_w - TEXT_PADDING {
                    new_cursor_x += entered_text_width;
                }
                else {
                    break;
                }
                let new_text = String::from_str(text.as_slice().slice_to(idx))
                    .append(t.as_slice()).append(text.as_slice().slice_from(idx));
                *text = new_text;
                new_idx += t.len();
            }

            // Check for control keys.
            let pressed_keys = uic.get_pressed_keys();
            for key in pressed_keys.iter() {
                match *key {
                    Backspace => {
                        if text.len() > 0u
                        && text.len() >= idx
                        && idx > 0u {
                            let rem_idx = idx - 1u;
                            new_cursor_x -= uic.get_character_w(font_size, text.as_slice().char_at(rem_idx));
                            let new_text = String::from_str(text.as_slice().slice_to(rem_idx))
                                .append(text.as_slice().slice_from(idx));
                            *text = new_text;
                            new_idx = rem_idx;
                        }
                    },
                    Left => {
                        if idx > 0 {
                            new_cursor_x -= uic.get_character_w(font_size, text.as_slice().char_at(idx - 1u));
                            new_idx -= 1u;
                        }
                    },
                    Right => {
                        if text.len() > idx {
                            new_cursor_x += uic.get_character_w(font_size, text.as_slice().char_at(idx));
                            new_idx += 1u;
                        }
                    },
                    Return => if text.len() > 0u { callback(text) },
                    _ => (),
                }
            }

            State(w_state, Captured(new_idx, new_cursor_x))
        },
    }};

    set_state(uic, ui_id, new_state);

}

/// Check if cursor is over the pad and if so, which
fn over_elem(uic: &mut UIContext,
             pos: Point<f64>,
             mouse_pos: Point<f64>,
             rect_w: f64,
             rect_h: f64,
             pad_pos: Point<f64>,
             pad_w: f64,
             pad_h: f64,
             text_pos: Point<f64>,
             text_w: f64,
             font_size: FontSize,
             text: &str) -> Element {
    match rectangle::is_over(pos, mouse_pos, rect_w, rect_h) {
        false => Nill,
        true => match rectangle::is_over(pad_pos, mouse_pos, pad_w, pad_h) {
            false => Rect,
            true => {
                let (idx, cursor_x) = closest_idx(uic, mouse_pos, pad_h, pad_pos.y, text_pos.x, text_w, font_size, text);
                Text(idx, cursor_x)
            },
        },
    }
}

/// Check which character is closest to the mouse cursor.
fn closest_idx(uic: &mut UIContext,
               mouse_pos: Point<f64>,
               pad_h: f64,
               pad_y: f64,
               text_x: f64,
               text_w: f64,
               font_size: FontSize,
               text: &str) -> (Idx, f64) {
    if mouse_pos.x <= text_x { return (0u, text_x) }
    let mut x = text_x;
    let mut prev_x = x;
    let mut left_x = text_x;
    for (i, ch) in text.chars().enumerate() {
        let character = uic.get_character(font_size, ch);
        let char_w = (character.glyph.advance().x >> 16) as f64;
        x += char_w;
        let right_x = prev_x + char_w / 2.0;
        if mouse_pos.x > left_x && mouse_pos.x < right_x { return (i, prev_x) }
        prev_x = x;
        left_x = right_x;
    }
    (text.len(), text_x + text_w)
}

/// Check and return the current state of the TextBox.
fn get_new_state(over_elem: Element,
                 prev_box_state: State,
                 mouse: MouseState) -> State {
    match prev_box_state {
        State(prev, Uncaptured) => {
            match (over_elem, prev, mouse.left) {
                (_, Normal, Down) => State(Normal, Uncaptured),
                (Nill, Normal, Up) | (Nill, Highlighted(_), Up) => State(Normal, Uncaptured),
                (_, Normal, Up) | (_, Highlighted(_), Up) => State(Highlighted(over_elem), Uncaptured),
                (_, Highlighted(p_elem), Down) | (_, Clicked(p_elem), Down) =>
                    State(Clicked(p_elem), Uncaptured),
                (Text(idx, x), Clicked(Text(_, _)), Up) => State(Highlighted(over_elem), Captured(idx, x)),
                (Nill, _, _) => State(Normal, Uncaptured),
                _ => prev_box_state,
            }
        },
        State(prev, Captured(p_idx, p_x)) => {
            match (over_elem, prev, mouse.left) {
                (Nill, Clicked(Nill), Up) => State(Normal, Uncaptured),
                (Text(idx, x), Clicked(Text(_, _)), Up) => State(Highlighted(over_elem), Captured(idx, x)),
                (_, Normal, Up) | (_, Highlighted(_), Up) | (_, Clicked(_), Up)  =>
                    State(Highlighted(over_elem), Captured(p_idx, p_x)),
                (_, Highlighted(p_elem), Down) | (_, Clicked(p_elem), Down) =>
                    State(Clicked(p_elem), Captured(p_idx, p_x)),
                _ => prev_box_state,
            }
        },
    }
}

/// Draw the text cursor.
fn draw_cursor(win_w: f64,
               win_h: f64,
               gl: &mut Gl,
               uic: &mut UIContext,
               color: Color,
               cursor_x: f64,
               pad_pos_y: f64,
               pad_h: f64) {
    let context = Context::abs(win_w, win_h);
    let (r, g, b, a) = color.plain_contrast().as_tuple();
    context
        .line(cursor_x, pad_pos_y, cursor_x, pad_pos_y + pad_h)
        .round_border_width(1f64)
        .rgba(r, g, b, abs(a * (precise_time_s() * 2.5).sin() as f32))
        .draw(gl);
}


