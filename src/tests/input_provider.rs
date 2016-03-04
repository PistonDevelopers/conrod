use events::{UiEvent, Scroll, MouseClick, MouseDrag, InputState, InputProvider};
use input::{Input, Button};
use input::keyboard::{Key, ModifierKey, NO_MODIFIER};
use input::mouse::MouseButton;
use position::Point;

#[test]
fn mouse_position_should_return_mouse_position_from_current_state() {
    let position = [5.0, 7.0];
    let mut input_state = InputState::new();
    input_state.mouse.xy = position;
    let input = ProviderImpl::with_input_state(input_state);
    assert_eq!(position, input.mouse_position());
}

#[test]
fn mouse_button_down_should_return_true_if_button_is_pressed() {
    let mut input_state = InputState::new();
    input_state.mouse.buttons.press(MouseButton::Right, [0.0, 0.0]);
    let input = ProviderImpl::with_input_state(input_state);
    assert_eq!(Some([0.0, 0.0]), input.mouse_button_down(MouseButton::Right));
    assert!(input.mouse_left_button_down().is_none());
}

#[test]
fn mouse_button_releases_should_be_collected_into_a_vec() {
    use input::mouse::MouseButton::{Left, Right, Middle};
    let input = ProviderImpl::with_events(vec![
        UiEvent::Raw(Input::Release(Button::Mouse(Left))),
        UiEvent::Raw(Input::Release(Button::Mouse(Middle))),
        UiEvent::Raw(Input::Release(Button::Mouse(Right))),
    ]);

    let expected = vec![Left, Middle, Right];
    assert_eq!(expected, input.mouse_buttons_just_released().collect::<Vec<MouseButton>>());
}

#[test]
fn mouse_button_presses_should_be_collected_into_a_vec() {
    use input::mouse::MouseButton::{Left, Right, Middle};
    let input = ProviderImpl::with_events(vec![
        UiEvent::Raw(Input::Press(Button::Mouse(Left))),
        UiEvent::Raw(Input::Press(Button::Mouse(Middle))),
        UiEvent::Raw(Input::Press(Button::Mouse(Right))),
    ]);

    let expected = vec![Left, Middle, Right];
    assert_eq!(expected, input.mouse_buttons_just_pressed().collect::<Vec<MouseButton>>());
}

#[test]
fn key_releases_should_be_collected_into_a_vec() {
    let input = ProviderImpl::with_events(vec![
        UiEvent::Raw(Input::Release(Button::Keyboard(Key::LShift))),
        UiEvent::Raw(Input::Release(Button::Keyboard(Key::H))),
        UiEvent::Raw(Input::Release(Button::Keyboard(Key::I))),
        UiEvent::Raw(Input::Release(Button::Keyboard(Key::J))),
    ]);

    let expected = vec![Key::LShift, Key::H, Key::I, Key::J];
    assert_eq!(expected, input.keys_just_released().collect::<Vec<Key>>());
}

#[test]
fn key_presses_should_be_collected_into_a_vec() {
    let input = ProviderImpl::with_events(vec![
        UiEvent::Raw(Input::Press(Button::Keyboard(Key::LShift))),
        UiEvent::Raw(Input::Press(Button::Keyboard(Key::H))),
        UiEvent::Raw(Input::Press(Button::Keyboard(Key::I))),
        UiEvent::Raw(Input::Press(Button::Keyboard(Key::J))),
    ]);

    let expected = vec![Key::LShift, Key::H, Key::I, Key::J];
    assert_eq!(expected, input.keys_just_pressed().collect::<Vec<Key>>());
}

#[test]
fn mouse_clicks_should_be_filtered_by_mouse_button() {
    let input = ProviderImpl::with_events(vec![
        UiEvent::MouseClick(MouseClick{
            button: MouseButton::Left,
            xy: [50.0, 40.0],
            modifiers: NO_MODIFIER
        }),
        UiEvent::MouseClick(MouseClick{
            button: MouseButton::Right,
            xy: [70.0, 30.0],
            modifiers: NO_MODIFIER
        }),
        UiEvent::MouseClick(MouseClick{
            button: MouseButton::Middle,
            xy: [90.0, 20.0],
            modifiers: NO_MODIFIER
        }),
    ]);

    let r_click = input.mouse_click(MouseButton::Right).expect("expected a right click event");
    assert_eq!(MouseButton::Right, r_click.button);
    let l_click = input.mouse_click(MouseButton::Left).expect("expected a left click event");
    assert_eq!(MouseButton::Left, l_click.button);
    let m_click = input.mouse_click(MouseButton::Middle).expect("expected a middle click event");
    assert_eq!(MouseButton::Middle, m_click.button);

}

#[test]
fn only_the_last_drag_event_should_be_returned() {
    let input = ProviderImpl::with_events(vec![
        drag_event(MouseButton::Left, [20.0, 10.0], [30.0, 20.0]),
        drag_event(MouseButton::Left, [20.0, 10.0], [40.0, 30.0]),
        drag_event(MouseButton::Left, [20.0, 10.0], [50.0, 40.0])
    ]);

    let expected_drag = MouseDrag{
        button: MouseButton::Left,
        start: [20.0, 10.0],
        end: [50.0, 40.0],
        modifiers: NO_MODIFIER,
    };
    let actual_drag = input.mouse_left_drag().expect("expected a mouse drag event");
    assert_eq!(expected_drag, actual_drag);
}

#[test]
fn scroll_events_should_be_aggregated_into_one_when_scroll_is_called() {
    let input = ProviderImpl::with_events(vec![
        scroll_event(10.0, 33.0),
        scroll_event(10.0, 33.0),
        scroll_event(10.0, 33.0)
    ]);

    let expected_scroll = Scroll {
        x: 30.0,
        y: 99.0,
        modifiers: ModifierKey::default()
    };

    let actual = input.scroll().expect("expected a scroll event");
    assert_eq!(expected_scroll, actual);
}

fn drag_event(mouse_button: MouseButton, start: Point, end: Point) -> UiEvent {
    UiEvent::MouseDrag(MouseDrag{
        button: mouse_button,
        start: start,
        end: end,
        modifiers: NO_MODIFIER,
    })
}

fn scroll_event(x: f64, y: f64) -> UiEvent {
    UiEvent::Scroll(Scroll{
        x: x,
        y: y,
        modifiers: NO_MODIFIER
    })
}

/// This is just a basic struct that implements the `InputProvider` Trait so that
/// the default trait methods are easy to test
struct ProviderImpl{
    events: Vec<UiEvent>,
    current: InputState,
}

impl ProviderImpl {
    fn new(events: Vec<UiEvent>, state: InputState) -> ProviderImpl {
        ProviderImpl{
            events: events,
            current: state,
        }
    }

    fn with_events(events: Vec<UiEvent>) -> ProviderImpl {
        ProviderImpl::new(events, InputState::new())
    }

    fn with_input_state(state: InputState) -> ProviderImpl {
        ProviderImpl::new(vec!(), state)
    }
}

pub type TestInputEventIterator<'a> = ::std::slice::Iter<'a, UiEvent>;


impl<'a> InputProvider<'a> for ProviderImpl {
    type Events = TestInputEventIterator<'a>;

    fn events(&'a self) -> Self::Events {
        self.events.iter()
    }

    fn current(&self) -> &InputState {
        &self.current
    }

    fn mouse_button_down(&self, button: MouseButton) -> Option<Point> {
        self.current().mouse.buttons[button].xy_if_down().map(|_| {
            self.mouse_position()
        })
    }
}
