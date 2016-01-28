#[cfg(test)]
mod tests;

use events::{ConrodEvent, Scroll, MouseClick, MouseDrag};
use input::{Input, Motion, Button};
use input::keyboard::{Key, ModifierKey};
use input::mouse::MouseButton;


#[allow(missing_docs)]
pub trait EventProvider {
    fn all_events(&self) -> &Vec<ConrodEvent>;

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

    fn mouse_buttons_just_pressed(&self) -> Vec<MouseButton> {
        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::Raw(Input::Press(Button::Mouse(button))) => Some(button),
                _ => None
            }
        }).collect::<Vec<MouseButton>>()
    }

    fn mouse_buttons_just_released(&self) -> Vec<MouseButton> {
        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::Raw(Input::Release(Button::Mouse(button))) => Some(button),
                _ => None
            }
        }).collect::<Vec<MouseButton>>()
    }

    fn scroll(&self) -> Option<Scroll> {
        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::Scroll(scroll) => Some(scroll),
                _ => None
            }
        }).fold(None, |maybe_scroll, scroll| {
            if maybe_scroll.is_some() {
                maybe_scroll.map(|acc| {
                    Scroll{
                        x: acc.x + scroll.x,
                        y: acc.y + scroll.y,
                        modifiers: scroll.modifiers
                    }
                })
            } else {
                Some(scroll)
            }
        })
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
