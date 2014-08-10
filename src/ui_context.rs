
use std::collections::HashMap;
use point::Point;
use widget::Widget;
use piston::{
    AssetStore,
    GameEvent,
    MouseMove,
    MousePress,
    MouseRelease,
};
use freetype;

type UIID = uint;
type State = uint;

/// Represents the current state of a mouse button.
#[deriving(Show, Clone)]
pub enum MouseButtonState {
    Up,
    Down,
}

/// Represents the current state of the Mouse.
#[deriving(Show, Clone)]
pub struct MouseState {
    pub pos: Point<f64>,
    pub left: MouseButtonState,
    pub middle: MouseButtonState,
    pub right: MouseButtonState,
}

impl MouseState {
    /// Constructor for a MouseState struct.
    pub fn new(pos: Point<f64>,
               left: MouseButtonState,
               middle: MouseButtonState,
               right: MouseButtonState) -> MouseState {
        MouseState { pos: pos, left: left, middle: middle, right: right }
    }
}

/// UIContext retains the state of all widgets and
/// data relevant to the draw_widget functions.
pub struct UIContext {
    data: HashMap<UIID, Widget>,
    pub mouse: MouseState,
    freetype: freetype::Library,
    pub face: freetype::Face,
}

impl UIContext {

    /// Constructor for a UIContext.
    pub fn new() -> UIContext {
        let freetype = freetype::Library::init().unwrap();
        let asset_store = AssetStore::from_folder("../assets");
        let font = asset_store.path("Dense-Regular.otf").unwrap();
        let face = freetype.new_face(font.as_str().unwrap(), 0).unwrap();
        UIContext {
            data: HashMap::new(),
            mouse: MouseState::new(Point::new(0f64, 0f64, 0f64), Up, Up, Up),
            freetype: freetype,
            face: face,
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

    /// Return the current state for the given ui_id.
    pub fn get_widget(&mut self, ui_id: uint, default: Widget) -> &mut Widget {
        self.data.find_or_insert(ui_id, default)
    }

}
