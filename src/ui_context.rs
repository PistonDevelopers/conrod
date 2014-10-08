
use glyph_cache::{
    GlyphCache,
    Character,
};
use label::FontSize;
use mouse_state::{
    Up,
    Down,
    MouseState,
};
use piston::input;
use piston::event::{
    GenericEvent,
    MouseCursorEvent,
    PressEvent,
    ReleaseEvent,
    RenderEvent,
    TextEvent,
};
use point::Point;
use widget;
use widget::Widget;

/// User Interface Identifier. Each unique `widget::draw` call
/// should pass it's own unique UIID so that UIContext can keep
/// track of it's state.
pub type UIID = u64;

/// UIContext retains the state of all widgets and
/// data relevant to the draw_widget functions.
pub struct UIContext {
    data: Vec<(Widget, widget::Placing)>,
    pub mouse: MouseState,
    pub keys_just_pressed: Vec<input::keyboard::Key>,
    pub keys_just_released: Vec<input::keyboard::Key>,
    pub text_just_entered: Vec<String>,
    glyph_cache: GlyphCache,
    prev_event_was_render: bool,
    /// Window width.
    pub win_w: f64,
    /// Window height.
    pub win_h: f64,
    /// The UIID of the widget drawn previously.
    prev_uiid: u64,
}

impl UIContext {

    /// Constructor for a UIContext.
    pub fn new(font_file: &str) -> UIContext {
        UIContext {
            data: Vec::from_elem(512, (widget::NoWidget, widget::NoPlace)),
            mouse: MouseState::new(Point::new(0f64, 0f64, 0f64), Up, Up, Up),
            keys_just_pressed: Vec::with_capacity(10u),
            keys_just_released: Vec::with_capacity(10u),
            text_just_entered: Vec::with_capacity(10u),
            glyph_cache: GlyphCache::new(font_file),
            prev_event_was_render: false,
            win_w: 0f64,
            win_h: 0f64,
            prev_uiid: 0u64,
        }
    }

    /// Handle game events and update the state.
    pub fn handle_event<E: GenericEvent>(&mut self, event: &E) {
        if self.prev_event_was_render {
            self.flush_input();
            self.prev_event_was_render = false;
        }
        event.render(|args| {
            self.win_w = args.width as f64;
            self.win_h = args.height as f64;
            self.prev_event_was_render = true;
        });
        event.mouse_cursor(|x, y| {
            self.mouse.pos = Point::new(x, y, 0.0);
        });
        event.press(|button_type| {
            match button_type {
                input::Mouse(button) => {
                    *match button {
                        input::mouse::Left => &mut self.mouse.left,
                        _/*input::mouse::Right*/ => &mut self.mouse.right,
                        //Middle => &mut self.mouse.middle,
                    } = Down;
                },
                input::Keyboard(key) => self.keys_just_pressed.push(key),
            }
        });
        event.release(|button_type| {
            match button_type {
                input::Mouse(button) => {
                    *match button {
                        input::mouse::Left => &mut self.mouse.left,
                        _/*input::mouse::Right*/ => &mut self.mouse.right,
                        //Middle => &mut self.mouse.middle,
                    } = Up;
                },
                input::Keyboard(key) => self.keys_just_released.push(key),
            }
        });
        event.text(|text| {
            self.text_just_entered.push(text.to_string())
        });
    }

    /// Return the current mouse state.
    pub fn get_mouse_state(&self) -> MouseState {
        self.mouse.clone()
    }

    /// Return the vector of recently pressed keys.
    pub fn get_pressed_keys(&self) -> Vec<input::keyboard::Key> {
        self.keys_just_pressed.clone()
    }

    /// Return the vector of recently entered text.
    pub fn get_entered_text(&self) -> Vec<String> {
        self.text_just_entered.clone()
    }

    /// Return a mutable reference to the widget that matches the given ui_id
    pub fn get_widget(&mut self, ui_id: UIID, default: Widget) -> &mut Widget {
        let ui_id_idx = ui_id as uint;
        if self.data.len() > ui_id_idx {
            match *self.data.get_mut(ui_id_idx) {
                (widget::NoWidget, _) => {
                    match *self.data.get_mut(ui_id_idx) {
                        (ref mut widget, _) => {
                            *widget = default; widget
                        }
                    }
                },
                _ => {
                    match *self.data.get_mut(ui_id_idx) {
                        (ref mut widget, _) => widget
                    }
                },
            }
        } else {
            if ui_id_idx >= self.data.len() {
                let num_to_push = ui_id_idx - self.data.len();
                let mut vec = Vec::from_elem(num_to_push, (widget::NoWidget, widget::NoPlace));
                vec.push((default, widget::NoPlace));
                self.data.extend(vec.into_iter());
            } else {
                *self.data.get_mut(ui_id_idx) = (default, widget::NoPlace);
            }
            match *self.data.get_mut(ui_id_idx) {
                (ref mut widget, _) => widget,
            }
        }
    }

    /// Set the Placing for a particular widget.
    pub fn set_place(&mut self, ui_id: UIID, x: f64, y: f64, w: f64, h: f64) {
        match *self.data.get_mut(ui_id as uint) {
            (_, ref mut placing) => {
                *placing = widget::Place(x, y, w, h)
            }
        }
        self.prev_uiid = ui_id;
    }

    /// Get the UIID of the previous widget.
    pub fn get_prev_uiid(&self) -> UIID { self.prev_uiid }

    /// Get the Placing for a particular widget.
    pub fn get_placing(&self, ui_id: UIID) -> widget::Placing {
        if ui_id as uint >= self.data.len() { widget::NoPlace }
        else {
            match self.data[ui_id as uint] { (_, ref placing) => *placing }
        }
    }

    /// Return a reference to a `Character` from the GlyphCache.
    pub fn get_character(&mut self, size: FontSize, ch: char) -> &Character {
        self.glyph_cache.get_character(size, ch)
    }

    /// Return the width of a 'Character'.
    pub fn get_character_w(&mut self, size: FontSize, ch: char) -> f64 {
        (self.get_character(size, ch).glyph.advance().x >> 16) as f64
    }

    /// Flush all stored keys.
    pub fn flush_input(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.text_just_entered.clear();
    }

}
