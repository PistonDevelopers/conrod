#[cfg(test)]
mod tests;

use events::{InputState, ButtonMap, ConrodEvent, MouseClick, MouseDrag, Scroll};
use input::{Input, MouseButton, Motion, Button};
use input::keyboard::{ModifierKey, Key};
use position::{Point, Scalar};
use widget::Index;

#[allow(missing_docs)]
pub trait WidgetEvents {
    fn all_events(&self) -> &Vec<ConrodEvent>;
    fn starting_state(&self) -> &InputState;
    fn currently_capturing_mouse(&self) -> Option<Index>;
    fn currently_capturing_keyboard(&self) -> Option<Index>;

    fn text_just_entered(&self) -> Option<String> {
        let all_text: String = self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::Raw(Input::Text(ref text)) => Some(text),
                _ => None
            }
        }).fold(String::new(), |acc, item| {
            acc + item
        });

        if all_text.is_empty() {
            None
        } else {
            Some(all_text)
        }
    }

    fn keys_just_released(&self) -> Vec<Key> {
        use input::Button::Keyboard;

        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::Raw(Input::Release(Keyboard(key))) => Some(key),
                _ => None
            }
        }).collect::<Vec<Key>>()
    }

    fn keys_just_pressed(&self) -> Vec<Key> {
        use input::Button::Keyboard;

        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::Raw(Input::Press(Keyboard(key))) => Some(key),
                _ => None
            }
        }).collect::<Vec<Key>>()
    }

    fn scroll(&self) -> Option<Scroll> {
        let modifiers = self.modifiers();
        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::Raw(Input::Move(Motion::MouseScroll(x, y))) => {
                    Some(Scroll{
                        x: x,
                        y: y,
                        modifiers: modifiers
                    })
                },
                _ => None
            }
        }).fold(None, |maybe_scroll, scroll| {
            if maybe_scroll.is_some() {
                maybe_scroll.map(|acc| {
                    Scroll{
                        x: acc.x + scroll.x,
                        y: acc.y + scroll.y,
                        modifiers: modifiers
                    }
                })
            } else {
                Some(scroll)
            }
        })
    }

    fn modifiers(&self) -> ModifierKey {
        let mut mods = self.starting_state().modifiers;
        for event in self.all_events() {
            match *event {
                ConrodEvent::Raw(Input::Press(Button::Keyboard(key))) => {
                    get_modifier(key).map(|mod_key| mods.insert(mod_key));
                },
                ConrodEvent::Raw(Input::Release(Button::Keyboard(key))) => {
                    get_modifier(key).map(|mod_key| mods.remove(mod_key));
                },
                _ => {}
            }
        }
        mods
    }

    fn mouse_left_drag(&self) -> Option<MouseDrag> {
        self.mouse_drag(MouseButton::Left)
    }

    fn mouse_drag(&self, button: MouseButton) -> Option<MouseDrag> {
        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::MouseDrag(drag_evt) if drag_evt.button == button => Some(drag_evt),
                _ => None
            }
        }).last()
    }

    fn mouse_left_click(&self) -> Option<MouseClick> {
        self.mouse_click(MouseButton::Left)
    }

    fn mouse_right_click(&self) -> Option<MouseClick> {
        self.mouse_click(MouseButton::Right)
    }

    fn mouse_click(&self, button: MouseButton) -> Option<MouseClick> {
        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::MouseClick(click) if click.button == button => Some(click),
                _ => None
            }
        }).next()
    }

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

#[allow(missing_docs)]
pub struct GlobalInput {
    events: Vec<ConrodEvent>,
    drag_threshold: Scalar,
    pub start_state: InputState,
    pub current_state: InputState,
}

impl WidgetEvents for GlobalInput {

    fn all_events(&self) -> &Vec<ConrodEvent> {
        &self.events
    }

    fn starting_state(&self) -> &InputState {
        &self.start_state
    }

    fn currently_capturing_mouse(&self) -> Option<Index> {
        self.current_state.widget_capturing_mouse
    }

    fn currently_capturing_keyboard(&self) -> Option<Index> {
        self.current_state.widget_capturing_keyboard
    }
}

impl GlobalInput {

    pub fn new() -> GlobalInput {
        GlobalInput{
            events: Vec::new(),
            drag_threshold: 4.0,
            start_state: InputState::new(),
            current_state: InputState::new(),
        }
    }

    pub fn push_event(&mut self, event: ConrodEvent) {
        use input::Input::{Press, Release, Move};
        use input::Motion::MouseRelative;
        use input::Button::Mouse;

        let maybe_new_event = match event {
            ConrodEvent::Raw(Release(Mouse(button))) => self.handle_mouse_release(button),
            ConrodEvent::Raw(Move(MouseRelative(x, y))) => self.handle_mouse_move([x, y]),
            _ => None
        };

        self.current_state.update(&event);
        self.events.push(event);
        if let Some(new_event) = maybe_new_event {
            self.push_event(new_event);
        }
    }

    pub fn reset(&mut self) {
        self.events.clear();
    }

    pub fn current_mouse_position(&self) -> Point {
        self.current_state.mouse_position
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
