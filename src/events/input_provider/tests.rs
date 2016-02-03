use super::InputProvider;
use events::{ConrodEvent, Scroll, MouseClick, MouseDrag};
use input::{Input, Motion, Button};
use input::keyboard::{Key, ModifierKey, NO_MODIFIER};
use input::mouse::MouseButton;
use position::Point;

struct ProviderImpl(Vec<ConrodEvent>);
impl InputProvider for ProviderImpl {
    fn all_events(&self) -> &Vec<ConrodEvent> {
        &self.0
    }
}

#[test]
fn mouse_button_releases_should_be_collected_into_a_vec() {
    use input::mouse::MouseButton::{Left, Right, Middle};
    let input = ProviderImpl(vec![
        ConrodEvent::Raw(Input::Release(Button::Mouse(Left))),
        ConrodEvent::Raw(Input::Release(Button::Mouse(Middle))),
        ConrodEvent::Raw(Input::Release(Button::Mouse(Right))),
    ]);

    let expected = vec![Left, Middle, Right];
    assert_eq!(expected, input.mouse_buttons_just_released());
}

#[test]
fn mouse_button_presses_should_be_collected_into_a_vec() {
    use input::mouse::MouseButton::{Left, Right, Middle};
    let input = ProviderImpl(vec![
        ConrodEvent::Raw(Input::Press(Button::Mouse(Left))),
        ConrodEvent::Raw(Input::Press(Button::Mouse(Middle))),
        ConrodEvent::Raw(Input::Press(Button::Mouse(Right))),
    ]);

    let expected = vec![Left, Middle, Right];
    assert_eq!(expected, input.mouse_buttons_just_pressed());
}

#[test]
fn key_releases_should_be_collected_into_a_vec() {
    let input = ProviderImpl(vec![
        ConrodEvent::Raw(Input::Release(Button::Keyboard(Key::LShift))),
        ConrodEvent::Raw(Input::Release(Button::Keyboard(Key::H))),
        ConrodEvent::Raw(Input::Release(Button::Keyboard(Key::I))),
        ConrodEvent::Raw(Input::Release(Button::Keyboard(Key::J))),
    ]);

    let expected = vec![Key::LShift, Key::H, Key::I, Key::J];
    assert_eq!(expected, input.keys_just_released());
}

#[test]
fn key_presses_should_be_collected_into_a_vec() {
    let input = ProviderImpl(vec![
        ConrodEvent::Raw(Input::Press(Button::Keyboard(Key::LShift))),
        ConrodEvent::Raw(Input::Press(Button::Keyboard(Key::H))),
        ConrodEvent::Raw(Input::Press(Button::Keyboard(Key::I))),
        ConrodEvent::Raw(Input::Press(Button::Keyboard(Key::J))),
    ]);

    let expected = vec![Key::LShift, Key::H, Key::I, Key::J];
    assert_eq!(expected, input.keys_just_pressed());
}

#[test]
fn mouse_clicks_should_be_filtered_by_mouse_button() {
    let input = ProviderImpl(vec![
        ConrodEvent::MouseClick(MouseClick{
            button: MouseButton::Left,
            location: [50.0, 40.0],
            modifier: NO_MODIFIER
        }),
        ConrodEvent::MouseClick(MouseClick{
            button: MouseButton::Right,
            location: [70.0, 30.0],
            modifier: NO_MODIFIER
        }),
        ConrodEvent::MouseClick(MouseClick{
            button: MouseButton::Middle,
            location: [90.0, 20.0],
            modifier: NO_MODIFIER
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
    let input = ProviderImpl(vec![
        drag_event(MouseButton::Left, [20.0, 10.0], [30.0, 20.0]),
        drag_event(MouseButton::Left, [20.0, 10.0], [40.0, 30.0]),
        drag_event(MouseButton::Left, [20.0, 10.0], [50.0, 40.0])
    ]);

    let expected_drag = MouseDrag{
        button: MouseButton::Left,
        start: [20.0, 10.0],
        end: [50.0, 40.0],
        modifier: NO_MODIFIER,
        in_progress: true
    };
    let actual_drag = input.mouse_left_drag().expect("expected a mouse drag event");
    assert_eq!(expected_drag, actual_drag);
}

#[test]
fn scroll_events_should_be_aggregated_into_one_when_scroll_is_called() {
    let input = ProviderImpl(vec![
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

fn drag_event(mouse_button: MouseButton, start: Point, end: Point) -> ConrodEvent {
    ConrodEvent::MouseDrag(MouseDrag{
        button: mouse_button,
        start: start,
        end: end,
        modifier: NO_MODIFIER,
        in_progress: true
    })
}

fn scroll_event(x: f64, y: f64) -> ConrodEvent {
    ConrodEvent::Scroll(Scroll{
        x: x,
        y: y,
        modifiers: NO_MODIFIER
    })
}
