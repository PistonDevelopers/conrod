
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
use piston::{
    GameEvent,
    Input,
    input,
};
use point::Point;
use std::collections::HashMap;
use widget::Widget;

/// User Interface Identifier. Each unique `widget::draw` call
/// should pass it's own unique UIID so that UIContext can keep
/// track of it's state.
pub type UIID = u64;

/// UIContext retains the state of all widgets and
/// data relevant to the draw_widget functions.
pub struct UIContext {
    data: HashMap<UIID, Widget>,
    pub mouse: MouseState,
    glyph_cache: GlyphCache,
}

impl UIContext {

    /// Constructor for a UIContext.
    pub fn new(font_file: &str) -> UIContext {
        UIContext {
            data: HashMap::new(),
            mouse: MouseState::new(Point::new(0f64, 0f64, 0f64), Up, Up, Up),
            glyph_cache: GlyphCache::new(font_file),
        }
    }

    /// Handle game events and update the state.
    pub fn event(&mut self, event: &mut GameEvent) {
        match *event {
            Input(input::MouseMove { x, y, .. }) => {
                self.mouse.pos = Point::new(x, y, 0f64);
            },
            Input(input::MousePress { button, .. }) => {
                *match button {
                    input::mouse::Left => &mut self.mouse.left,
                    _/*input::mouse::Right*/ => &mut self.mouse.right,
                    //Middle => &mut self.mouse.middle,
                } = Down;
            },
            Input(input::MouseRelease { button, .. }) => {
                *match button {
                    input::mouse::Left => &mut self.mouse.left,
                    _/*input::mouse::Right*/ => &mut self.mouse.right,
                    //Middle => &mut self.mouse.middle,
                } = Up;
            },
            _ => (),
        }
    }

    /// Return the current mouse state.
    pub fn get_mouse_state(&self) -> MouseState {
        self.mouse.clone()
    }

    /// Return a mutable reference to the widget that matches the given ui_id
    pub fn get_widget(&mut self, ui_id: UIID, default: Widget) -> &mut Widget {
        self.data.find_or_insert(ui_id, default)
    }

    /// Return a reference to a `Character` from the GlyphCache.
    pub fn get_character(&mut self, size: FontSize, ch: char) -> &Character {
        self.glyph_cache.get_character(size, ch)
    }

}
