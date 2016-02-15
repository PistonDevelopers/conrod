use input::Button::Keyboard;
use input::keyboard::{self, ModifierKey, Key};
use input::Button::Mouse;
use input::mouse::MouseButton;
use input::{Input, Motion};
use position::Scalar;
use events::{UiEvent, MouseClick, MouseDrag, Scroll, InputProvider, GlobalInput};
use widget::{Id, Index};

#[test]
fn resetting_input_should_set_starting_state_to_current_state() {
    let mut input = GlobalInput::new(0.0);
    input.push_event(UiEvent::Raw(Input::Press(Keyboard(Key::LShift))));
    input.push_event(UiEvent::Raw(Input::Move(Motion::MouseScroll(0.0, 50.0))));

    let expected_start_state = input.current_state.clone();
    input.reset();
    assert_eq!(expected_start_state, input.start_state);
}

#[test]
fn resetting_input_should_clear_out_all_events() {
    let mut input = GlobalInput::new(0.0);
    input.push_event(UiEvent::Raw(Input::Press(Keyboard(Key::LShift))));
    input.push_event(UiEvent::Raw(Input::Move(Motion::MouseScroll(0.0, 50.0))));
    input.reset();
    assert!(input.all_events().next().is_none());
}

#[test]
fn scroll_events_should_have_modifier_keys() {
    let mut input = GlobalInput::new(0.0);

    input.push_event(UiEvent::Raw(Input::Press(Keyboard(Key::LShift))));
    input.push_event(UiEvent::Raw(Input::Move(Motion::MouseScroll(0.0, 50.0))));

    let scroll = input.scroll().expect("expected to get a scroll event");
    assert_eq!(keyboard::SHIFT, scroll.modifiers);
}

#[test]
fn global_input_should_track_widget_currently_capturing_keyboard() {
    let mut input = GlobalInput::new(0.0);

    let idx: Index = Index::Public(Id(5));
    input.push_event(UiEvent::WidgetCapturesKeyboard(idx));

    assert_eq!(Some(idx), input.currently_capturing_keyboard());

    input.push_event(UiEvent::WidgetUncapturesKeyboard(idx));
    assert!(input.currently_capturing_keyboard().is_none());

    let new_idx: Index = Index::Public(Id(5));
    input.push_event(UiEvent::WidgetCapturesKeyboard(new_idx));
    assert_eq!(Some(new_idx), input.currently_capturing_keyboard());
}

#[test]
fn global_input_should_track_widget_currently_capturing_mouse() {
    let mut input = GlobalInput::new(0.0);

    let idx: Index = Index::Public(Id(5));
    input.push_event(UiEvent::WidgetCapturesMouse(idx));

    assert_eq!(Some(idx), input.currently_capturing_mouse());

    input.push_event(UiEvent::WidgetUncapturesMouse(idx));
    assert!(input.currently_capturing_mouse().is_none());

    let new_idx: Index = Index::Public(Id(5));
    input.push_event(UiEvent::WidgetCapturesMouse(new_idx));
    assert_eq!(Some(new_idx), input.currently_capturing_mouse());
}

#[test]
fn global_input_should_track_current_mouse_position() {
    let mut input = GlobalInput::new(0.0);

    input.push_event(mouse_move_event(50.0, 77.7));
    assert_eq!([50.0, 77.7], input.mouse_position());
}

#[test]
fn entered_text_should_be_aggregated_from_multiple_events() {
    let mut input = GlobalInput::new(0.0);

    input.push_event(UiEvent::Raw(Input::Text("Phil ".to_string())));
    input.push_event(UiEvent::Raw(Input::Text("is a".to_string())));
    input.push_event(UiEvent::Raw(Input::Text("wesome".to_string())));

    let actual_text = input.text_just_entered().expect("expected to get a String, got None");
    assert_eq!("Phil is awesome".to_string(), actual_text);
}

#[test]
fn drag_event_should_still_be_created_if_reset_is_called_between_press_and_release() {
    let mut input = GlobalInput::new(4.0);

    input.push_event(UiEvent::Raw(Input::Press(Mouse(MouseButton::Left))));
    input.push_event(mouse_move_event(50.0, 77.7));
    input.reset();
    input.push_event(UiEvent::Raw(Input::Release(Mouse(MouseButton::Left))));

    assert!(input.mouse_left_drag().is_some());
}

#[test]
fn click_event_should_still_be_created_if_reset_is_called_between_press_and_release() {
    let mut input = GlobalInput::new(4.0);

    input.push_event(UiEvent::Raw(Input::Press(Mouse(MouseButton::Left))));
    input.reset();
    input.push_event(UiEvent::Raw(Input::Release(Mouse(MouseButton::Left))));

    assert!(input.mouse_left_click().is_some());
}

#[test]
fn no_events_should_be_returned_after_reset_is_called() {
    let mut input = GlobalInput::new(0.0);
    input.push_event(UiEvent::Raw(Input::Press(Keyboard(Key::RShift))));
    input.push_event(UiEvent::Raw(Input::Move(Motion::MouseScroll(7.0, 88.5))));
    input.push_event(UiEvent::Raw(Input::Press(Mouse(MouseButton::Left))));
    input.push_event(mouse_move_event(60.0, 30.0));
    input.push_event(UiEvent::Raw(Input::Release(Mouse(MouseButton::Left))));

    input.reset();

    assert!(input.all_events().next().is_none());
}

#[test]
fn drag_with_modifer_key_should_include_modifiers_in_drag_event() {
    use input::keyboard::SHIFT;

    let mut input = GlobalInput::new(4.0);
    input.push_event(UiEvent::Raw(Input::Press(Keyboard(Key::RShift))));
    input.push_event(UiEvent::Raw(Input::Move(Motion::MouseScroll(7.0, 88.5))));

    let scroll = input.scroll().expect("expected a scroll event");

    assert_eq!(SHIFT, scroll.modifiers);
}

#[test]
fn click_with_modifier_key_should_include_modifiers_in_click_event() {
    use input::keyboard::CTRL;

    let mut input = GlobalInput::new(4.0);
    input.push_event(UiEvent::Raw(Input::Press(Keyboard(Key::LCtrl))));
    input.push_event(UiEvent::Raw(Input::Press(Mouse(MouseButton::Left))));
    input.push_event(UiEvent::Raw(Input::Release(Mouse(MouseButton::Left))));

    let click = input.mouse_left_click().expect("expected mouse left click event");
    let expected = MouseClick {
        button: MouseButton::Left,
        location: [0.0, 0.0],
        modifier: CTRL
    };
    assert_eq!(expected, click);
}

#[test]
fn high_level_scroll_event_should_be_created_from_a_raw_mouse_scroll() {
    let mut input = GlobalInput::new(0.0);

    input.push_event(UiEvent::Raw(Input::Move(Motion::MouseScroll(10.0, 33.0))));

    let expected_scroll = Scroll{
        x: 10.0,
        y: 33.0,
        modifiers: ModifierKey::default()
    };
    let actual_scroll = input.scroll().expect("expected a scroll event");
    assert_eq!(expected_scroll, actual_scroll);
}

#[test]
fn mouse_button_pressed_moved_released_creates_final_drag_event() {
    let mut input = GlobalInput::new(4.0);

    input.push_event(UiEvent::Raw(Input::Press(Mouse(MouseButton::Left))));
    input.push_event(mouse_move_event(20.0, 10.0));
    input.push_event(UiEvent::Raw(Input::Release(Mouse(MouseButton::Left))));

    let expected_drag = MouseDrag{
        button: MouseButton::Left,
        start: [0.0, 0.0],
        end: [20.0, 10.0],
        modifier: ModifierKey::default(),
        in_progress: false
    };
    let mouse_drag = input.mouse_drag(MouseButton::Left).expect("Expected to get a mouse drag event");
    assert_eq!(expected_drag, mouse_drag);
}

#[test]
fn mouse_button_pressed_then_moved_creates_drag_event() {
    let mut input = GlobalInput::new(4.0);

    let press = UiEvent::Raw(Input::Press(Mouse(MouseButton::Left)));
    let mouse_move = mouse_move_event(20.0, 10.0);
    input.push_event(press.clone());
    input.push_event(mouse_move.clone());

    let expected_drag = MouseDrag{
        button: MouseButton::Left,
        start: [0.0, 0.0],
        end: [20.0, 10.0],
        modifier: ModifierKey::default(),
        in_progress: true
    };
    let mouse_drag = input.mouse_drag(MouseButton::Left).expect("Expected to get a mouse drag event");
    assert_eq!(expected_drag, mouse_drag);
}

#[test]
fn mouse_click_position_should_be_mouse_position_when_pressed() {
    let mut input = GlobalInput::new(4.0);

    input.push_event(mouse_move_event(4.0, 5.0));
    input.push_event(UiEvent::Raw(Input::Press(Mouse(MouseButton::Left))));
    input.push_event(mouse_move_event(5.0, 5.0));
    input.push_event(UiEvent::Raw(Input::Release(Mouse(MouseButton::Left))));

    let expected_click = MouseClick {
        button: MouseButton::Left,
        location: [4.0, 5.0],
        modifier: ModifierKey::default()
    };
    let actual_click = input.mouse_click(MouseButton::Left).expect("expected a mouse click event");

    assert_eq!(expected_click, actual_click);

}

#[test]
fn mouse_button_pressed_then_released_should_create_mouse_click_event() {
    let mut input = GlobalInput::new(4.0);

    let press = UiEvent::Raw(Input::Press(Mouse(MouseButton::Left)));
    let release = UiEvent::Raw(Input::Release(Mouse(MouseButton::Left)));
    input.push_event(press.clone());
    input.push_event(release.clone());

    let expected_click = MouseClick {
        button: MouseButton::Left,
        location: [0.0, 0.0],
        modifier: ModifierKey::default()
    };
    let actual_click = input.mouse_click(MouseButton::Left).expect("expected a mouse click event");

    assert_eq!(expected_click, actual_click);
}

#[test]
fn all_events_should_return_all_inputs_in_order() {
    let mut input = GlobalInput::new(0.0);

    let evt1 = UiEvent::Raw(Input::Press(Keyboard(Key::Z)));
    input.push_event(evt1.clone());
    let evt2 = UiEvent::Raw(Input::Press(Keyboard(Key::A)));
    input.push_event(evt2.clone());

    let results = input.all_events().collect::<Vec<&UiEvent>>();
    assert_eq!(2, results.len());
    assert_eq!(evt1, *results[0]);
    assert_eq!(evt2, *results[1]);
}

fn mouse_move_event(x: Scalar, y: Scalar) -> UiEvent {
    UiEvent::Raw(Input::Move(Motion::MouseRelative(x, y)))
}
