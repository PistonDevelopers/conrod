use input::Button::Keyboard;
use input::keyboard::{self, ModifierKey, Key};
use input::Button::Mouse;
use input::mouse::MouseButton;
use input::{Input, Motion};
use position::{Point, Scalar};
use events::conrod_event::{ConrodEvent, MouseClick, MouseDrag, Scroll};
use widget::{Id, Index};
use super::*;

#[test]
fn scroll_events_should_have_modifier_keys() {
    let mut input = GlobalInput::new();

    input.push_event(ConrodEvent::Raw(Input::Press(Keyboard(Key::LShift))));
    input.push_event(ConrodEvent::Raw(Input::Move(Motion::MouseScroll(0.0, 50.0))));

    let scroll = input.scroll().expect("expected to get a scroll event");
    assert_eq!(keyboard::SHIFT, scroll.modifiers);
}

#[test]
fn event_aggregator_should_track_widget_currently_capturing_keyboard() {
    let mut aggregator = GlobalInput::new();

    let idx: Index = Index::Public(Id(5));
    aggregator.push_event(ConrodEvent::WidgetCapturesKeyboard(idx));

    assert_eq!(Some(idx), aggregator.currently_capturing_keyboard());

    aggregator.push_event(ConrodEvent::WidgetUncapturesKeyboard(idx));
    assert!(aggregator.currently_capturing_keyboard().is_none());

    let new_idx: Index = Index::Public(Id(5));
    aggregator.push_event(ConrodEvent::WidgetCapturesKeyboard(new_idx));
    assert_eq!(Some(new_idx), aggregator.currently_capturing_keyboard());
}

#[test]
fn event_aggregator_should_track_widget_currently_capturing_mouse() {
    let mut aggregator = GlobalInput::new();

    let idx: Index = Index::Public(Id(5));
    aggregator.push_event(ConrodEvent::WidgetCapturesMouse(idx));

    assert_eq!(Some(idx), aggregator.currently_capturing_mouse());

    aggregator.push_event(ConrodEvent::WidgetUncapturesMouse(idx));
    assert!(aggregator.currently_capturing_mouse().is_none());

    let new_idx: Index = Index::Public(Id(5));
    aggregator.push_event(ConrodEvent::WidgetCapturesMouse(new_idx));
    assert_eq!(Some(new_idx), aggregator.currently_capturing_mouse());
}

#[test]
fn event_aggregator_should_track_current_mouse_position() {
    let mut aggregator = GlobalInput::new();

    aggregator.push_event(mouse_move_event(50.0, 77.7));
    assert_eq!([50.0, 77.7], aggregator.current_mouse_position());
}

#[test]
fn entered_text_should_be_aggregated_from_multiple_events() {
    let mut aggregator = GlobalInput::new();

    aggregator.push_event(ConrodEvent::Raw(Input::Text("Phil ".to_string())));
    aggregator.push_event(ConrodEvent::Raw(Input::Text("is a".to_string())));
    aggregator.push_event(ConrodEvent::Raw(Input::Text("wesome".to_string())));

    let actual_text = aggregator.text_just_entered().expect("expected to get a String, got None");
    assert_eq!("Phil is awesome".to_string(), actual_text);
}

#[test]
fn drag_event_should_still_be_created_if_reset_is_called_between_press_and_release() {
    let mut aggregator = GlobalInput::new();

    aggregator.push_event(ConrodEvent::Raw(Input::Press(Mouse(MouseButton::Left))));
    aggregator.push_event(mouse_move_event(50.0, 77.7));
    aggregator.reset();
    aggregator.push_event(ConrodEvent::Raw(Input::Release(Mouse(MouseButton::Left))));

    assert!(aggregator.mouse_left_drag().is_some());
}

#[test]
fn click_event_should_still_be_created_if_reset_is_called_between_press_and_release() {
    let mut aggregator = GlobalInput::new();

    aggregator.push_event(ConrodEvent::Raw(Input::Press(Mouse(MouseButton::Left))));
    aggregator.reset();
    aggregator.push_event(ConrodEvent::Raw(Input::Release(Mouse(MouseButton::Left))));

    assert!(aggregator.mouse_left_click().is_some());
}

#[test]
fn no_events_should_be_returned_after_reset_is_called() {
    let mut aggregator = GlobalInput::new();
    aggregator.push_event(ConrodEvent::Raw(Input::Press(Keyboard(Key::RShift))));
    aggregator.push_event(ConrodEvent::Raw(Input::Move(Motion::MouseScroll(7.0, 88.5))));
    aggregator.push_event(ConrodEvent::Raw(Input::Press(Mouse(MouseButton::Left))));
    aggregator.push_event(mouse_move_event(60.0, 30.0));
    aggregator.push_event(ConrodEvent::Raw(Input::Release(Mouse(MouseButton::Left))));

    aggregator.reset();

    let events = aggregator.all_events();
    assert!(events.is_empty());
}

#[test]
fn drag_with_modifer_key_should_include_modifiers_in_drag_event() {
    use input::keyboard::SHIFT;

    let mut aggregator = GlobalInput::new();
    aggregator.push_event(ConrodEvent::Raw(Input::Press(Keyboard(Key::RShift))));
    aggregator.push_event(ConrodEvent::Raw(Input::Move(Motion::MouseScroll(7.0, 88.5))));

    let scroll = aggregator.scroll().expect("expected a scroll event");

    assert_eq!(SHIFT, scroll.modifiers);
}

#[test]
fn click_with_modifier_key_should_include_modifiers_in_click_event() {
    use input::keyboard::CTRL;

    let mut aggregator = GlobalInput::new();
    aggregator.push_event(ConrodEvent::Raw(Input::Press(Keyboard(Key::LCtrl))));
    aggregator.push_event(ConrodEvent::Raw(Input::Press(Mouse(MouseButton::Left))));
    aggregator.push_event(ConrodEvent::Raw(Input::Release(Mouse(MouseButton::Left))));

    let click = aggregator.mouse_left_click().expect("expected mouse left click event");
    let expected = MouseClick {
        button: MouseButton::Left,
        location: [0.0, 0.0],
        modifier: CTRL
    };
    assert_eq!(expected, click);
}

#[test]
fn keys_just_released_should_return_vec_of_keys_just_released() {
    let mut aggregator = GlobalInput::new();

    aggregator.push_event(ConrodEvent::Raw(Input::Release(Keyboard(Key::D))));
    aggregator.push_event(ConrodEvent::Raw(Input::Release(Keyboard(Key::O))));
    aggregator.push_event(ConrodEvent::Raw(Input::Release(Keyboard(Key::R))));
    aggregator.push_event(ConrodEvent::Raw(Input::Release(Keyboard(Key::K))));

    let expected = vec![Key::D, Key::O, Key::R, Key::K];
    let actual = aggregator.keys_just_released();
    assert_eq!(expected, actual);

}

#[test]
fn keys_just_pressed_should_return_vec_of_keys_just_pressed() {
    let mut aggregator = GlobalInput::new();

    aggregator.push_event(ConrodEvent::Raw(Input::Press(Keyboard(Key::N))));
    aggregator.push_event(ConrodEvent::Raw(Input::Press(Keyboard(Key::E))));
    aggregator.push_event(ConrodEvent::Raw(Input::Press(Keyboard(Key::R))));
    aggregator.push_event(ConrodEvent::Raw(Input::Press(Keyboard(Key::D))));

    let expected = vec![Key::N, Key::E, Key::R, Key::D];
    let actual = aggregator.keys_just_pressed();
    assert_eq!(expected, actual);
}

#[test]
fn scroll_events_should_be_aggregated_into_one_when_scroll_is_called() {
    let mut aggregator = GlobalInput::new();

    aggregator.push_event(ConrodEvent::Raw(Input::Move(Motion::MouseScroll(10.0, 33.0))));
    aggregator.push_event(ConrodEvent::Raw(Input::Move(Motion::MouseScroll(10.0, 33.0))));
    aggregator.push_event(ConrodEvent::Raw(Input::Move(Motion::MouseScroll(10.0, 33.0))));

    let expected_scroll = Scroll {
        x: 30.0,
        y: 99.0,
        modifiers: ModifierKey::default()
    };

    let actual = aggregator.scroll().expect("expected a scroll event");
    assert_eq!(expected_scroll, actual);

}

#[test]
fn handler_should_return_scroll_event_if_one_exists() {
    let mut aggregator = GlobalInput::new();

    aggregator.push_event(ConrodEvent::Raw(Input::Move(Motion::MouseScroll(10.0, 33.0))));

    let expected_scroll = Scroll{
        x: 10.0,
        y: 33.0,
        modifiers: ModifierKey::default()
    };
    let actual_scroll = aggregator.scroll().expect("expected a scroll event");
    assert_eq!(expected_scroll, actual_scroll);
}
#[test]
fn mouse_button_pressed_moved_released_creates_final_drag_event() {
    let mut aggregator = GlobalInput::new();

    aggregator.push_event(ConrodEvent::Raw(Input::Press(Mouse(MouseButton::Left))));
    aggregator.push_event(mouse_move_event(20.0, 10.0));
    aggregator.push_event(ConrodEvent::Raw(Input::Release(Mouse(MouseButton::Left))));

    let expected_drag = MouseDrag{
        button: MouseButton::Left,
        start: [0.0, 0.0],
        end: [20.0, 10.0],
        modifier: ModifierKey::default(),
        in_progress: false
    };
    let mouse_drag = aggregator.mouse_drag(MouseButton::Left).expect("Expected to get a mouse drag event");
    assert_eq!(expected_drag, mouse_drag);
}

#[test]
fn mouse_button_pressed_then_moved_creates_drag_event() {
    let mut aggregator = GlobalInput::new();

    let press = ConrodEvent::Raw(Input::Press(Mouse(MouseButton::Left)));
    let mouse_move = mouse_move_event(20.0, 10.0);
    aggregator.push_event(press.clone());
    aggregator.push_event(mouse_move.clone());

    let expected_drag = MouseDrag{
        button: MouseButton::Left,
        start: [0.0, 0.0],
        end: [20.0, 10.0],
        modifier: ModifierKey::default(),
        in_progress: true
    };
    let mouse_drag = aggregator.mouse_drag(MouseButton::Left).expect("Expected to get a mouse drag event");
    assert_eq!(expected_drag, mouse_drag);
}

#[test]
fn mouse_click_position_should_be_mouse_position_when_pressed() {
    let mut aggregator = GlobalInput::new();

    aggregator.push_event(mouse_move_event(4.0, 5.0));
    aggregator.push_event(ConrodEvent::Raw(Input::Press(Mouse(MouseButton::Left))));
    aggregator.push_event(mouse_move_event(5.0, 5.0));
    aggregator.push_event(ConrodEvent::Raw(Input::Release(Mouse(MouseButton::Left))));

    let expected_click = MouseClick {
        button: MouseButton::Left,
        location: [4.0, 5.0],
        modifier: ModifierKey::default()
    };
    let actual_click = aggregator.mouse_click(MouseButton::Left).expect("expected a mouse click event");

    assert_eq!(expected_click, actual_click);

}

#[test]
fn mouse_button_pressed_then_released_should_create_mouse_click_event() {
    let mut aggregator = GlobalInput::new();

    let press = ConrodEvent::Raw(Input::Press(Mouse(MouseButton::Left)));
    let release = ConrodEvent::Raw(Input::Release(Mouse(MouseButton::Left)));
    aggregator.push_event(press.clone());
    aggregator.push_event(release.clone());

    let expected_click = MouseClick {
        button: MouseButton::Left,
        location: [0.0, 0.0],
        modifier: ModifierKey::default()
    };
    let actual_click = aggregator.mouse_click(MouseButton::Left).expect("expected a mouse click event");

    assert_eq!(expected_click, actual_click);
}

#[test]
fn all_events_should_return_all_inputs_in_order() {
    let mut aggregator = GlobalInput::new();

    let evt1 = ConrodEvent::Raw(Input::Press(Keyboard(Key::Z)));
    aggregator.push_event(evt1.clone());
    let evt2 = ConrodEvent::Raw(Input::Press(Keyboard(Key::A)));
    aggregator.push_event(evt2.clone());

    let results = aggregator.all_events();
    assert_eq!(2, results.len());
    assert_eq!(evt1, results[0]);
    assert_eq!(evt2, results[1]);
}

fn mouse_move_event(x: Scalar, y: Scalar) -> ConrodEvent {
    ConrodEvent::Raw(Input::Move(Motion::MouseRelative(x, y)))
}
