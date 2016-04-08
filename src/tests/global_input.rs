use input::Button::Keyboard;
use input::keyboard::Key;
use input::Button::Mouse;
use input::mouse::MouseButton;
use input::{Input, Motion};
use position::Scalar;
use events::{UiEvent, InputProvider, GlobalInput};


// Pushes an event onto the given global input with a default drag threshold.
fn push_event(input: &mut GlobalInput, event: UiEvent) {
    input.push_event(event);
}

fn mouse_move_event(x: Scalar, y: Scalar) -> UiEvent {
    UiEvent::Raw(Input::Move(Motion::MouseRelative(x, y)))
}


#[test]
fn resetting_input_should_set_starting_state_to_current_state() {
    let mut input = GlobalInput::new();
    push_event(&mut input, UiEvent::Raw(Input::Press(Keyboard(Key::LShift))));
    push_event(&mut input, UiEvent::Raw(Input::Move(Motion::MouseScroll(0.0, 50.0))));

    let expected_start = input.current.clone();
    input.clear_events_and_update_start_state();
    assert_eq!(expected_start, input.start);
}

#[test]
fn resetting_input_should_clear_out_events() {
    let mut input = GlobalInput::new();
    push_event(&mut input, UiEvent::Raw(Input::Press(Keyboard(Key::LShift))));
    push_event(&mut input, UiEvent::Raw(Input::Move(Motion::MouseScroll(0.0, 50.0))));
    input.clear_events_and_update_start_state();
    assert!(input.events().next().is_none());
}

#[test]
fn mouse_button_down_should_return_none_if_button_is_not_pressed() {
    let pressed_button = MouseButton::Middle;
    let non_pressed_button = MouseButton::Right;

    let mut input = GlobalInput::new();
    push_event(&mut input, UiEvent::Raw(Input::Press(Mouse(pressed_button))));

    assert!(input.mouse_button_down(non_pressed_button).is_none());
}

#[test]
fn entered_text_should_be_aggregated_from_multiple_events() {
    let mut input = GlobalInput::new();

    push_event(&mut input, UiEvent::Raw(Input::Text("Phil ".to_string())));
    push_event(&mut input, UiEvent::Raw(Input::Text("is a".to_string())));
    push_event(&mut input, UiEvent::Raw(Input::Text("wesome".to_string())));

    let actual_text: String = input.text_just_entered().collect();
    assert_eq!("Phil is awesome".to_string(), actual_text);
}

#[test]
fn no_events_should_be_returned_after_reset_is_called() {
    let mut input = GlobalInput::new();
    push_event(&mut input, UiEvent::Raw(Input::Press(Keyboard(Key::RShift))));
    push_event(&mut input, UiEvent::Raw(Input::Move(Motion::MouseScroll(7.0, 88.5))));
    push_event(&mut input, UiEvent::Raw(Input::Press(Mouse(MouseButton::Left))));
    push_event(&mut input, mouse_move_event(60.0, 30.0));
    push_event(&mut input, UiEvent::Raw(Input::Release(Mouse(MouseButton::Left))));

    input.clear_events_and_update_start_state();

    assert!(input.events().next().is_none());
}

#[test]
fn events_should_return_all_inputs_in_order() {
    let mut input = GlobalInput::new();

    let evt1 = UiEvent::Raw(Input::Press(Keyboard(Key::Z)));
    push_event(&mut input, evt1.clone());
    let evt2 = UiEvent::Raw(Input::Press(Keyboard(Key::A)));
    push_event(&mut input, evt2.clone());

    let results = input.events().collect::<Vec<&UiEvent>>();
    assert_eq!(2, results.len());
    assert_eq!(evt1, *results[0]);
    assert_eq!(evt2, *results[1]);
}
