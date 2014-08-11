
use std::collections::HashMap;
use point::Point;
use widget::Widget;
use piston::{
    GameEvent,
    MouseMove,
    MousePress,
    MouseRelease,
};
use label::FontSize;
use mouse_state::{
    Up,
    Down,
    MouseState,
};
use glyph_cache::{
    GlyphCache,
    Character,
};

pub type UIID = uint;

/// UIContext retains the state of all widgets and
/// data relevant to the draw_widget functions.
pub struct UIContext {
    data: HashMap<UIID, Widget>,
    pub mouse: MouseState,
    glyph_cache: GlyphCache,
}

impl UIContext {

    /// Constructor for a UIContext.
    pub fn new() -> UIContext {
        UIContext {
            data: HashMap::new(),
            mouse: MouseState::new(Point::new(0f64, 0f64, 0f64), Up, Up, Up),
            glyph_cache: GlyphCache::new(),
        }
    }

    /// Handle game events and update the state.
    pub fn event(&mut self, event: &mut GameEvent) {
        match *event {
            MouseMove(args) => {
                self.mouse.pos = Point::new(args.x, args.y, 0f64);
            },
            MousePress(args) => {
                *match args.button {
                    /*Left*/ _ => &mut self.mouse.left,
                    //Right => &mut self.mouse.right,
                    //Middle => &mut self.mouse.middle,
                } = Down;
            },
            MouseRelease(args) => {
                *match args.button {
                    /*Left*/ _ => &mut self.mouse.left,
                    //Right => &mut self.mouse.right,
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
    pub fn get_widget(&mut self, ui_id: uint, default: Widget) -> &mut Widget {
        self.data.find_or_insert(ui_id, default)
    }

    /// Return a reference to a `Character` from the GlyphCache.
    pub fn get_character(&mut self, size: FontSize, ch: char) -> &Character {
        self.glyph_cache.get_character(size, ch)
    }

}
