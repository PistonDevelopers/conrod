use event::{self, Input};
use input::{self, Key, Motion, MouseButton};
use input::Button::Keyboard;
use input::Button::Mouse;
use position::Scalar;


// Pushes an event onto the given global input with a default drag threshold.
fn push_event(input: &mut input::Global, event: event::Event) {
    input.push_event(event);
}

fn mouse_move_event(x: Scalar, y: Scalar) -> event::Event {
    event::Event::Raw(Input::Motion(Motion::MouseRelative { x: x, y: y }))
}


#[test]
fn resetting_input_should_set_starting_state_to_current_state() {
    let mut input = input::Global::new();
    push_event(&mut input, event::Event::Raw(Input::Press(Keyboard(Key::LShift))));
    push_event(&mut input, event::Event::Raw(Input::Motion(Motion::Scroll { x: 0.0, y: 50.0 })));

    let expected_start = input.current.clone();
    input.clear_events_and_update_start_state();
    assert_eq!(expected_start, input.start);
}

#[test]
fn resetting_input_should_clear_out_events() {
    let mut input = input::Global::new();
    push_event(&mut input, event::Event::Raw(Input::Press(Keyboard(Key::LShift))));
    push_event(&mut input, event::Event::Raw(Input::Motion(Motion::Scroll { x: 0.0, y: 50.0 })));
    input.clear_events_and_update_start_state();
    assert!(input.events().next().is_none());
}


#[test]
fn no_events_should_be_returned_after_reset_is_called() {
    let mut input = input::Global::new();
    push_event(&mut input, event::Event::Raw(Input::Press(Keyboard(Key::RShift))));
    push_event(&mut input, event::Event::Raw(Input::Motion(Motion::Scroll { x: 7.0, y: 88.5 })));
    push_event(&mut input, event::Event::Raw(Input::Press(Mouse(MouseButton::Left))));
    push_event(&mut input, mouse_move_event(60.0, 30.0));
    push_event(&mut input, event::Event::Raw(Input::Release(Mouse(MouseButton::Left))));

    input.clear_events_and_update_start_state();

    assert!(input.events().next().is_none());
}

#[test]
fn events_should_return_all_inputs_in_order() {
    let mut input = input::Global::new();

    let evt1 = event::Event::Raw(Input::Press(Keyboard(Key::Z)));
    push_event(&mut input, evt1.clone());
    let evt2 = event::Event::Raw(Input::Press(Keyboard(Key::A)));
    push_event(&mut input, evt2.clone());

    let results = input.events().collect::<Vec<&event::Event>>();
    assert_eq!(2, results.len());
    assert_eq!(evt1, *results[0]);
    assert_eq!(evt2, *results[1]);
}
