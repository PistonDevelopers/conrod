#[cfg(test)]
mod tests;

use events::{InputState, ButtonMap, ConrodEvent, MouseClick, MouseDrag, Scroll, EventProvider};
use input::{Input, MouseButton, Motion, Button};
use input::keyboard::{ModifierKey, Key};
use position::{Point, Scalar};
use widget::Index;

/// Global input event handler that also implements `EventProvider`. The `Ui` passes all events
/// to it's `GlobalInput` instance, which aggregates and interprets the events to provide
/// so-called 'high-level' events to widgets. This input gets reset after every update by the `Ui`.
pub struct GlobalInput {
    /// The `InputState` as it was at the end of the last update cycle.
    pub start_state: InputState,
    /// The most recent `InputState`, with updates from handling all the events
    /// this update cycle
    pub current_state: InputState,
    events: Vec<ConrodEvent>,
    drag_threshold: Scalar,
}

impl EventProvider for GlobalInput {
    fn all_events(&self) -> &Vec<ConrodEvent> {
        &self.events
    }
}

impl GlobalInput {

    /// Returns a fresh new `GlobalInput`
    pub fn new() -> GlobalInput {
        GlobalInput{
            events: Vec::new(),
            drag_threshold: 4.0,
            start_state: InputState::new(),
            current_state: InputState::new(),
        }
    }

    /// Adds a new event and updates the internal state.
    pub fn push_event(&mut self, event: ConrodEvent) {
        use input::Input::{Press, Release, Move};
        use input::Motion::MouseRelative;
        use input::Motion::MouseScroll;
        use input::Button::Mouse;

        let maybe_new_event = match event {
            ConrodEvent::Raw(Release(Mouse(button))) => self.handle_mouse_release(button),
            ConrodEvent::Raw(Move(MouseRelative(x, y))) => self.handle_mouse_move([x, y]),
            ConrodEvent::Raw(Move(MouseScroll(x, y))) => self.mouse_scroll(x, y),
            _ => None
        };

        self.current_state.update(&event);
        self.events.push(event);
        if let Some(new_event) = maybe_new_event {
            self.push_event(new_event);
        }
    }

    /// Called at the end of every update cycle in order to prepare the `GlobalInput` to
    /// handle events for the next one.
    pub fn reset(&mut self) {
        self.events.clear();
    }

    pub fn current_mouse_position(&self) -> Point {
        self.current_state.mouse_position
    }

    pub fn starting_state(&self) -> &InputState {
        &self.start_state
    }

    pub fn currently_capturing_mouse(&self) -> Option<Index> {
        self.current_state.widget_capturing_mouse
    }

    pub fn currently_capturing_keyboard(&self) -> Option<Index> {
        self.current_state.widget_capturing_keyboard
    }


    fn mouse_scroll(&self, x: f64, y: f64) -> Option<ConrodEvent> {
        Some(ConrodEvent::Scroll(Scroll{
            x: x,
            y: y,
            modifiers: self.current_state.modifiers
        }))
    }

    fn handle_mouse_move(&self, move_to: Point) -> Option<ConrodEvent> {
        self.current_state.mouse_buttons.pressed_button().and_then(|btn_and_point| {
            if self.is_drag(btn_and_point.1, move_to) {
                Some(ConrodEvent::MouseDrag(MouseDrag{
                    button: btn_and_point.0,
                    start: btn_and_point.1,
                    end: move_to,
                    in_progress: true,
                    modifier: self.current_state.modifiers
                }))
            } else {
                None
            }
        })
    }

    fn handle_mouse_release(&self, button: MouseButton) -> Option<ConrodEvent> {
        self.current_state.mouse_buttons.get(button).map(|point| {
            if self.is_drag(point, self.current_state.mouse_position) {
                ConrodEvent::MouseDrag(MouseDrag{
                    button: button,
                    start: point,
                    end: self.current_state.mouse_position,
                    modifier: self.current_state.modifiers,
                    in_progress: false
                })
            } else {
                ConrodEvent::MouseClick(MouseClick {
                    button: button,
                    location: point,
                    modifier: self.current_state.modifiers
                })
            }
        })
    }

    fn is_drag(&self, a: Point, b: Point) -> bool {
        distance_between(a, b) > self.drag_threshold
    }
}

impl IntoIterator for GlobalInput {
    type Item = ConrodEvent;
    type IntoIter = ::std::vec::IntoIter<ConrodEvent>;

    fn into_iter(self) -> Self::IntoIter {
        self.all_events().clone().into_iter()
    }
}

fn distance_between(a: Point, b: Point) -> Scalar {
    let dx_2 = (a[0] - b[0]).powi(2);
    let dy_2 = (a[1] - b[1]).powi(2);
    (dx_2 + dy_2).abs().sqrt()
}

fn get_modifier(key: Key) -> Option<ModifierKey> {
    use input::keyboard::{CTRL, SHIFT, ALT, GUI};

    match key {
        Key::LCtrl | Key::RCtrl => Some(CTRL),
        Key::LShift | Key::RShift => Some(SHIFT),
        Key::LAlt | Key::RAlt => Some(ALT),
        Key::LGui | Key::RGui => Some(GUI),
        _ => None
    }
}
