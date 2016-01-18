mod mouse_button_map;

use self::mouse_button_map::ButtonMap;
use input::{self, Input, MouseButton};
use input::keyboard::ModifierKey;
use position::{Point, Scalar};

#[derive(Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum ConrodEvent {
    Raw(Input),
    MouseClick(MouseButton, Point, ModifierKey),
    MouseDrag(MouseDragEvent),
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub struct MouseDragEvent{
    button: MouseButton,
    start: Point,
    end: Point,
    modifier: ModifierKey,
    in_progress: bool,
}


#[allow(missing_docs)]
pub trait ConrodEventHandler {
    fn push_event(&mut self, event: ConrodEvent);
    fn all_events<'a>(&'a self) -> &'a Vec<ConrodEvent>;
}


#[allow(missing_docs)]
pub struct EventHandlerImpl {
    events: Vec<ConrodEvent>,
    mouse_buttons: ButtonMap,
    mouse_position: Point,
    drag_threshold: Scalar,
}

#[allow(missing_docs)]
impl EventHandlerImpl {

    pub fn new() -> EventHandlerImpl {
        EventHandlerImpl{
            events: Vec::new(),
            mouse_buttons: ButtonMap::new(),
            mouse_position: [0.0, 0.0],
            drag_threshold: 4.0,
        }
    }

    fn handle_mouse_move(&mut self, move_to: Point) -> Option<ConrodEvent> {
        self.mouse_position = move_to;
        self.mouse_buttons.pressed_button().and_then(|btn_and_point| {
            let drag_distance = distance_between(move_to, btn_and_point.1);
            if drag_distance > self.drag_threshold {
                let drag = MouseDragEvent{
                    button: btn_and_point.0,
                    start: btn_and_point.1,
                    end: move_to,
                    in_progress: true,
                    modifier: ModifierKey::default()
                };
                Some(ConrodEvent::MouseDrag(drag))
            } else {
                None
            }
        })
    }

    fn handle_mouse_release(&mut self, button: MouseButton) -> Option<ConrodEvent> {
        self.mouse_buttons.take(button).map(|point| {
            ConrodEvent::MouseClick(button, point, ModifierKey::default())
        })
    }

    fn handle_mouse_press(&mut self, button: MouseButton) -> Option<ConrodEvent> {
        self.mouse_buttons.set(button, Some(self.mouse_position));
        None
    }
}

fn distance_between(a: Point, b: Point) -> Scalar {
    let dx_2 = (a[0] - b[0]).powi(2);
    let dy_2 = (a[1] - b[1]).powi(2);
    (dx_2 + dy_2).abs().sqrt()
}


impl ConrodEventHandler for EventHandlerImpl {

    fn push_event(&mut self, event: ConrodEvent) {
        use input::Input::{Press, Release, Move};
        use input::Motion::MouseCursor;
        use input::Button::Mouse;

        let maybe_new_event = match event {
            ConrodEvent::Raw(Press(Mouse(button))) => self.handle_mouse_press(button),
            ConrodEvent::Raw(Release(Mouse(button))) => self.handle_mouse_release(button),
            ConrodEvent::Raw(Move(MouseCursor(x, y))) => self.handle_mouse_move([x, y]),
            _ => None
        };

        self.events.push(event);
        if let Some(new_event) = maybe_new_event {
            self.push_event(new_event);
        }
    }

    fn all_events<'a>(&'a self) -> &'a Vec<ConrodEvent> {
        &self.events
    }
}

#[test]
fn mouse_button_pressed_moved_released_creates_final_drag_event() {
    use input::Button::Mouse;
    use input::mouse::MouseButton as MB;
    use input::Motion;

    let mut handler = EventHandlerImpl::new();

    let press = ConrodEvent::Raw(Input::Press(Mouse(MB::Left)));
    let mouse_move = mouse_move_event(20.0, 10.0);
    let release = ConrodEvent::Raw(Input::Release(Mouse(MB::Left)))
    handler.push_event(press.clone());
    handler.push_event(mouse_move .clone());
    handler.push_event(release.clone());

    let events = handler.all_events();

    //TODO: fix this test!
    
    assert_eq!(5, events.len());

    match events[3].clone() {
        ConrodEvent::MouseDrag(drag) => {
            assert_eq!(MB::Left, drag.button);
            assert_eq!([0.0, 0.0], drag.start);
            assert_eq!([30.0, 10.0], drag.end);
            assert!(drag.in_progress);
        },
        wrong_event => panic!("Expected MouseDrag, got: {:?}", wrong_event)
    };

}

#[test]
fn mouse_button_pressed_then_moved_creates_drag_event() {
    use input::Button::Mouse;
    use input::mouse::MouseButton as MB;
    use input::Motion;

    let mut handler = EventHandlerImpl::new();

    let press = ConrodEvent::Raw(Input::Press(Mouse(MB::Left)));
    let mouse_move = mouse_move_event(20.0, 10.0);
    handler.push_event(press.clone());
    handler.push_event(mouse_move.clone());

    let events = handler.all_events();
    // Should have the two raw events, plus a new MouseClick event
    assert_eq!(3, events.len());

    match events[2].clone() {
        ConrodEvent::MouseDrag(drag) => {
            assert_eq!(MB::Left, drag.button);
            assert_eq!([0.0, 0.0], drag.start);
            assert_eq!([20.0, 10.0], drag.end);
            assert!(drag.in_progress);
        },
        wrong_event => panic!("Expected MouseDrag, got: {:?}", wrong_event)
    }
}

#[test]
fn mouse_click_position_should_be_mouse_position_when_pressed() {
    use input::Button::Mouse;
    use input::mouse::MouseButton as MB;

    let mut handler = EventHandlerImpl::new();

    handler.push_event(mouse_move_event(4.0, 5.0));
    handler.push_event(ConrodEvent::Raw(Input::Press(Mouse(MB::Left))));
    handler.push_event(mouse_move_event(5.0, 5.0));
    handler.push_event(ConrodEvent::Raw(Input::Release(Mouse(MB::Left))));

    let events = handler.all_events();
    assert_eq!(5, events.len());

    match events[4].clone() {
        ConrodEvent::MouseClick(MB::Left, point, _) => {
            assert_eq!([4.0, 5.0], point);
        },
        wrong_event => panic!("Expected MouseClick, got: {:?}", wrong_event)
    }

}

#[test]
fn mouse_button_pressed_then_released_should_create_mouse_click_event() {
    use input::Button::Mouse;
    use input::mouse::MouseButton as MB;

    let mut handler = EventHandlerImpl::new();

    let press = ConrodEvent::Raw(Input::Press(Mouse(MB::Left)));
    let release = ConrodEvent::Raw(Input::Release(Mouse(MB::Left)));
    handler.push_event(press.clone());
    handler.push_event(release.clone());

    let events = handler.all_events();
    // Should have the two raw events, plus a new MouseClick event
    assert_eq!(3, events.len());
    assert_eq!(press, events[0]);
    assert_eq!(release, events[1]);

    match events[2].clone() {
        ConrodEvent::MouseClick(MB::Left, _, _) => {},
        wrong_event => panic!("Expected MouseClick, got: {:?}", wrong_event)
    }
}

#[test]
fn all_events_should_return_all_inputs_in_order() {
    use input::Button::Keyboard;
    use input::keyboard::Key;

    let mut handler = EventHandlerImpl::new();

    let evt1 = ConrodEvent::Raw(Input::Press(Keyboard(Key::Z)));
    handler.push_event(evt1.clone());
    let evt2 = ConrodEvent::Raw(Input::Press(Keyboard(Key::A)));
    handler.push_event(evt2.clone());

    let results = handler.all_events();
    assert_eq!(2, results.len());
    assert_eq!(evt1, results[0]);
    assert_eq!(evt2, results[1]);
}

#[cfg(test)]
fn mouse_move_event(x: Scalar, y: Scalar) -> ConrodEvent {
    use input::Motion;
    ConrodEvent::Raw(Input::Move(Motion::MouseCursor(x, y)))
}
