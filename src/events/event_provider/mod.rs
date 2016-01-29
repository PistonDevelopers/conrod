//! Contains the `EventProvider` trait, which is used to provide input events to widgets.

#[cfg(test)]
mod tests;

use events::{ConrodEvent, Scroll, MouseClick, MouseDrag};
use input::{Input, Motion, Button};
use input::keyboard::{Key, ModifierKey};
use input::mouse::MouseButton;


/// Trait for something that provides events to be consumed by a widget.
/// Provides a bunch of convenience methods for filtering out specific types of events.
pub trait EventProvider {
    /// This is the only method that needs to be implemented.
    /// Just provided a reference to a `Vec<ConrodEvent>` that contains
    /// all the events for this update cycle.
    fn all_events(&self) -> &Vec<ConrodEvent>;

    /// Returns a `String` containing _all_ the text that was entered since
    /// the last update cycle.
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

    /// Returns all of the `Key`s that were released since the last update.
    fn keys_just_released(&self) -> Vec<Key> {
        use input::Button::Keyboard;

        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::Raw(Input::Release(Keyboard(key))) => Some(key),
                _ => None
            }
        }).collect::<Vec<Key>>()
    }

    /// Returns all of the keyboard `Key`s that were pressed since the last update.
    fn keys_just_pressed(&self) -> Vec<Key> {
        use input::Button::Keyboard;

        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::Raw(Input::Press(Keyboard(key))) => Some(key),
                _ => None
            }
        }).collect::<Vec<Key>>()
    }

    /// Returns all of the `MouseButton`s that were pressed since the last update.
    fn mouse_buttons_just_pressed(&self) -> Vec<MouseButton> {
        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::Raw(Input::Press(Button::Mouse(button))) => Some(button),
                _ => None
            }
        }).collect::<Vec<MouseButton>>()
    }

    /// Returns all of the `MouseButton`s that were released since the last update.
    fn mouse_buttons_just_released(&self) -> Vec<MouseButton> {
        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::Raw(Input::Release(Button::Mouse(button))) => Some(button),
                _ => None
            }
        }).collect::<Vec<MouseButton>>()
    }

    /// Returns a `Scroll` struct if any scrolling was done since the last update.
    /// If multiple raw scroll events occured since the last update (which could very well
    /// happen if the user is scrolling quickly), then the `Scroll` returned will represent an
    /// aggregate total of all the scrolling.
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

    /// Convenience method to call `mouse_drag`, passing in `MouseButton::Left`.
    /// Saves widgets from having to `use input::mouse::MouseButton` if all they care
    /// about is the left mouse button.
    fn mouse_left_drag(&self) -> Option<MouseDrag> {
        self.mouse_drag(MouseButton::Left)
    }

    /// Returns a `MouseDrag` if one has occured involving the given mouse button.
    /// If multiple raw mouse movement events have
    /// occured since the last update (which will happen if the user moves the mouse quickly),
    /// then the returned `MouseDrag` will be only the _most recent_ one, which will contain
    /// the most recent mouse position.
    fn mouse_drag(&self, button: MouseButton) -> Option<MouseDrag> {
        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::MouseDrag(drag_evt) if drag_evt.button == button => Some(drag_evt),
                _ => None
            }
        }).last()
    }

    /// Convenience method to call `mouse_click`, passing in passing in `MouseButton::Left`.
    /// Saves widgets from having to `use input::mouse::MouseButton` if all they care
    /// about is the left mouse button.
    fn mouse_left_click(&self) -> Option<MouseClick> {
        self.mouse_click(MouseButton::Left)
    }

    /// Convenience method to call `mouse_click`, passing in passing in `MouseButton::Right`.
    /// Saves widgets from having to `use input::mouse::MouseButton` if all they care
    /// about is the left mouse button.
    fn mouse_right_click(&self) -> Option<MouseClick> {
        self.mouse_click(MouseButton::Right)
    }

    /// Returns a `MouseClick` if one has occured with the given mouse button.
    /// A _click_ is determined to have occured if a mouse button was pressed and subsequently
    /// released while the mouse was in roughly the same place.
    fn mouse_click(&self, button: MouseButton) -> Option<MouseClick> {
        self.all_events().iter().filter_map(|evt| {
            match *evt {
                ConrodEvent::MouseClick(click) if click.button == button => Some(click),
                _ => None
            }
        }).next()
    }

}
