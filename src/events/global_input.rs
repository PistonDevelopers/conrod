//! Handles all of the global input events and state.
//! The core of this module is the `GlobalInput` struct. It is responsible for aggregating
//! and interpreting raw input events into high-level semantic events.

use events::{InputState, UiEvent, MouseClick, MouseDrag, Scroll, InputProvider};
use input::MouseButton;
use position::{Point, Scalar};
use widget::Index;

/// Global input event handler that also implements `InputProvider`. The `Ui` passes all events
/// to it's `GlobalInput` instance, which aggregates and interprets the events to provide
/// so-called 'high-level' events to widgets. This input gets reset after every update by the `Ui`.
pub struct GlobalInput {
    /// The `InputState` as it was at the end of the last update cycle.
    pub start_state: InputState,
    /// The most recent `InputState`, with updates from handling all the events
    /// this update cycle
    pub current_state: InputState,
    /// The events that have occurred between two consecutive updates.
    events: Vec<UiEvent>,
}

/// Iterator over global `UiEvent`s. Unlike the `WidgetInputEventIterator`, this will
/// never filter out any events, and all coordinates will be reative to the (0,0) origin
/// of the window.
pub type GlobalInputEventIterator<'a> = ::std::slice::Iter<'a, UiEvent>;

impl<'a> InputProvider<'a> for GlobalInput {
    type Events = GlobalInputEventIterator<'a>;

    fn all_events(&'a self) -> Self::Events {
        self.events.iter()
    }

    fn current_state(&'a self) -> &'a InputState {
        &self.current_state
    }

    fn mouse_button_down(&self, button: MouseButton) -> Option<Point> {
         self.current_state().mouse_buttons.get(button).map(|_| {
             self.mouse_position()
         })
    }
}

impl GlobalInput {

    /// Returns a fresh new `GlobalInput`
    pub fn new() -> GlobalInput {
        GlobalInput{
            events: Vec::new(),
            start_state: InputState::new(),
            current_state: InputState::new(),
        }
    }

    /// Adds a new event and updates the internal state.
    pub fn push_event(&mut self, event: UiEvent, drag_threshold: Scalar) {
        use input::Input::{Release, Move};
        use input::Motion::{MouseRelative, MouseScroll};
        use input::Button::Mouse;

        let is_drag = |a, b| distance_between(a, b) > drag_threshold;

        // Check for a new drag event.
        let maybe_drag_event = match event {
            UiEvent::Raw(Move(MouseRelative(x, y))) => {
                let xy = [x, y];
                self.current_state.mouse_buttons.pressed_button().and_then(|btn_and_point| {
                    if is_drag(btn_and_point.1, xy) {
                        Some(UiEvent::MouseDrag(MouseDrag{
                            button: btn_and_point.0,
                            start: btn_and_point.1,
                            end: xy,
                            in_progress: true,
                            modifier: self.current_state.modifiers
                        }))
                    } else {
                        None
                    }
                })
            },
            _ => None,
        };

        // Check for a new click event.
        let maybe_click_event = if let UiEvent::Raw(Release(Mouse(button))) = event {
            self.current_state.mouse_buttons.get(button).map(|point| {
                let click = MouseClick {
                    button: button,
                    location: point,
                    modifier: self.current_state.modifiers
                };
                UiEvent::MouseClick(click)
            })
        } else {
            None
        };

        // Check for a new scroll event.
        let maybe_scroll_event = if let UiEvent::Raw(Move(MouseScroll(x, y))) = event {
            let scroll = Scroll{
                x: x,
                y: y,
                modifiers: self.current_state.modifiers
            };
            Some(UiEvent::Scroll(scroll))
        } else {
            None
        };

        // // Check to see if we need to generate any higher level events from this raw event.
        // let maybe_new_event = match event {

        //     UiEvent::Raw(Release(Mouse(button))) => {
        //         self.current_state.mouse_buttons.get(button).map(|point| {
        //             if is_drag(point, self.current_state.mouse_position) {
        //                 UiEvent::MouseDrag(MouseDrag{
        //                     button: button,
        //                     start: point,
        //                     end: self.current_state.mouse_position,
        //                     modifier: self.current_state.modifiers,
        //                     in_progress: false
        //                 })
        //             } else {
        //                 UiEvent::MouseClick(MouseClick {
        //                     button: button,
        //                     location: point,
        //                     modifier: self.current_state.modifiers
        //                 })
        //             }
        //         })
        //     },

        //     UiEvent::Raw(Move(MouseRelative(x, y))) => {
        //         let xy = [x, y];
        //         self.current_state.mouse_buttons.pressed_button().and_then(|btn_and_point| {
        //             if is_drag(btn_and_point.1, xy) {
        //                 Some(UiEvent::MouseDrag(MouseDrag{
        //                     button: btn_and_point.0,
        //                     start: btn_and_point.1,
        //                     end: xy,
        //                     in_progress: true,
        //                     modifier: self.current_state.modifiers
        //                 }))
        //             } else {
        //                 None
        //             }
        //         })
        //     },

        //     UiEvent::Raw(Move(MouseScroll(x, y))) => {
        //         Some(UiEvent::Scroll(Scroll{
        //             x: x,
        //             y: y,
        //             modifiers: self.current_state.modifiers
        //         }))
        //     },

        //     _ => None
        // };

        let events = ::std::iter::once(event)
            .chain(maybe_drag_event)
            .chain(maybe_click_event)
            .chain(maybe_scroll_event);

        for event in events {
            self.current_state.update(&event);
            self.events.push(event);
        }
    }

    /// Called at the end of every update cycle in order to prepare the `GlobalInput` to
    /// handle events for the next one.
    pub fn clear_events_and_update_start_state(&mut self) {
        self.events.clear();
        self.start_state = self.current_state.clone();
    }

    /// Returns the most up to date position of the mouse
    pub fn mouse_position(&self) -> Point {
        self.current_state.mouse_position
    }

    /// Returns the input state as it was after the last update
    pub fn starting_state(&self) -> &InputState {
        &self.start_state
    }

    /// Returns the most up to date info on which widget is capturing the mouse
    pub fn currently_capturing_mouse(&self) -> Option<Index> {
        self.current_state.widget_capturing_mouse
    }

    /// Returns the most up to date info on which widget is capturing the keyboard
    pub fn currently_capturing_keyboard(&self) -> Option<Index> {
        self.current_state.widget_capturing_keyboard
    }

}

fn distance_between(a: Point, b: Point) -> Scalar {
    let dx_2 = (a[0] - b[0]).powi(2);
    let dy_2 = (a[1] - b[1]).powi(2);
    (dx_2 + dy_2).abs().sqrt()
}
