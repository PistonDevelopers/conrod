use input::{Input, Motion};
use input::keyboard::NO_MODIFIER;
use input::mouse::MouseButton;
use events::{UiEvent, MouseClick, GlobalInput, WidgetInput, InputProvider};
use widget::{Index, Id};
use position::Rect;

#[test]
fn maybe_mouse_position_should_return_position_if_mouse_is_over_the_widget() {
    let widget_area = Rect::from_corners([10.0, 10.0], [50.0, 50.0]);
    let mut global_input = GlobalInput::new(4.0);
    global_input.push_event(UiEvent::Raw(Input::Move(Motion::MouseRelative(30.0, 30.0))));

    let widget_input = WidgetInput::for_widget(Index::Public(Id(2)), widget_area, &global_input);

    let maybe_mouse_position = widget_input.maybe_mouse_position();
    assert_eq!(Some([0.0, 0.0]), maybe_mouse_position);
}

#[test]
fn maybe_mouse_position_should_return_none_if_mouse_is_not_over_the_widget() {
    let widget_area = Rect::from_corners([10.0, 10.0], [50.0, 50.0]);
    let mut global_input = GlobalInput::new(4.0);
    global_input.push_event(UiEvent::Raw(Input::Move(Motion::MouseRelative(-10.0, -10.0))));

    let widget_input = WidgetInput::for_widget(Index::Public(Id(2)), widget_area, &global_input);

    let maybe_mouse_position = widget_input.maybe_mouse_position();
    assert!(maybe_mouse_position.is_none());
}

#[test]
fn mouse_is_over_widget_should_be_true_if_mouse_is_over_the_widget_area() {
    let widget_area = Rect::from_corners([10.0, 10.0], [50.0, 50.0]);
    let mut global_input = GlobalInput::new(4.0);
    global_input.push_event(UiEvent::Raw(Input::Move(Motion::MouseRelative(30.0, 30.0))));

    let widget_input = WidgetInput::for_widget(Index::Public(Id(2)), widget_area, &global_input);

    assert!(widget_input.mouse_is_over_widget());
}

#[test]
fn mouse_is_over_widget_should_be_false_if_mouse_is_not_over_widget() {
    let widget_area = Rect::from_corners([10.0, 10.0], [50.0, 50.0]);
    let mut global_input = GlobalInput::new(4.0);
    global_input.push_event(UiEvent::Raw(Input::Move(Motion::MouseRelative(90.0, 90.0))));

    let widget_input = WidgetInput::for_widget(Index::Public(Id(2)), widget_area, &global_input);

    assert!(!widget_input.mouse_is_over_widget());
}

#[test]
fn input_state_should_be_provided_relative_to_the_widget_area() {
    let widget_area = Rect::from_corners([10.0, 10.0], [50.0, 50.0]);
    let mut global_input = GlobalInput::new(4.0);
    global_input.push_event(UiEvent::Raw(Input::Move(Motion::MouseRelative(30.0, 30.0))));

    let widget_input = WidgetInput::for_widget(Index::Public(Id(2)), widget_area, &global_input);

    assert_eq!([0.0, 0.0], widget_input.mouse_position());
}

#[test]
fn scroll_events_should_be_provided_if_widget_captures_mouse_but_not_keyboard() {
    let mut global_input = GlobalInput::new(4.0);
    let widget = Index::Public(Id(1));
    global_input.push_event(UiEvent::WidgetCapturesMouse(widget));
    global_input.push_event(UiEvent::Raw(Input::Move(Motion::MouseScroll(0.0, -76.0))));

    let some_rect = Rect::from_corners([5.0, 5.0], [40.0, 40.0]);
    let widget_input = WidgetInput::for_widget(widget, some_rect, &global_input);
    assert!(widget_input.scroll().is_some());
}

#[test]
fn scroll_events_should_be_provided_if_widget_captures_keyboard_but_not_mouse() {
    let mut global_input = GlobalInput::new(4.0);
    let widget = Index::Public(Id(1));
    global_input.push_event(UiEvent::WidgetCapturesKeyboard(widget));
    global_input.push_event(UiEvent::Raw(Input::Move(Motion::MouseScroll(0.0, -76.0))));

    let some_rect = Rect::from_corners([5.0, 5.0], [40.0, 40.0]);
    let widget_input = WidgetInput::for_widget(widget, some_rect, &global_input);
    assert!(widget_input.scroll().is_some());
}

#[test]
fn widget_input_should_provide_any_mouse_events_over_the_widgets_area_if_nothing_is_capturing_mouse() {
    let mut global_input = GlobalInput::new(4.0);
    global_input.push_event(UiEvent::MouseClick(MouseClick{
        button: MouseButton::Left,
        location: [10.0, 10.0],
        modifier: NO_MODIFIER
    }));
    assert!(global_input.currently_capturing_mouse().is_none());

    let widget = Index::Public(Id(4));
    let widget_area = Rect::from_corners([0.0, 0.0], [40.0, 40.0]);
    let widget_input = WidgetInput::for_widget(widget, widget_area, &global_input);

    widget_input.mouse_left_click().expect("Expected to get a mouse click event");

    let another_widget = Index::Public(Id(7));
    let another_area = Rect::from_corners([-20.0, -20.0], [0.0, 0.0]);
    let another_widget_input = WidgetInput::for_widget(another_widget, another_area, &global_input);

    assert!(another_widget_input.mouse_left_click().is_none());
}

#[test]
fn widget_input_should_only_provide_keyboard_input_to_widget_that_has_focus() {
    let mut global_input = GlobalInput::new(4.0);

    let some_rect = Rect::from_corners([0.0, 0.0], [40.0, 40.0]);
    let widget_4 = Index::Public(Id(4));
    global_input.push_event(UiEvent::WidgetCapturesKeyboard(widget_4));
    global_input.push_event(UiEvent::Raw(Input::Text("some text".to_string())));

    let widget_4_input = WidgetInput::for_widget(widget_4, some_rect, &global_input);
    let widget_4_text = widget_4_input.text_just_entered();
    assert_eq!(Some("some text".to_string()), widget_4_text);

    let another_widget_input = WidgetInput::for_widget(Index::Public(Id(7)),
            some_rect,
            &global_input);
    assert!(another_widget_input.text_just_entered().is_none());
}

#[test]
fn mouse_clicks_should_be_relative_to_widget_position() {
    let idx = Index::Public(Id(5));
    let mut global_input = GlobalInput::new(4.0);
    global_input.push_event(UiEvent::MouseClick(MouseClick{
        button: MouseButton::Left,
        location: [10.0, 10.0],
        modifier: NO_MODIFIER
    }));

    let rect = Rect::from_corners([0.0, 0.0], [20.0, 20.0]);
    let widget_input = WidgetInput::for_widget(idx, rect, &global_input);
    let widget_click = widget_input.mouse_left_click().expect("widget click should not be null");
    assert_eq!([0.0, 0.0], widget_click.location);
}

#[test]
fn mouse_drags_should_be_relative_to_widget_position() {
    use events::MouseDrag;

    let idx = Index::Public(Id(5));
    let mut global_input = GlobalInput::new(4.0);
    global_input.push_event(UiEvent::MouseDrag(MouseDrag{
        button: MouseButton::Left,
        start: [5.0, 5.0],
        end: [10.0, 10.0],
        modifier: NO_MODIFIER,
        in_progress: false
    }));

    let rect = Rect::from_corners([0.0, 0.0], [20.0, 20.0]);
    let widget_input = WidgetInput::for_widget(idx, rect, &global_input);
    let drag = widget_input.mouse_left_drag().expect("expected a mouse drag event");
    assert_eq!([-5.0, -5.0], drag.start);
    assert_eq!([0.0, 0.0], drag.end);

}
