use input::{Input, Motion, Button};
use input::keyboard::NO_MODIFIER;
use input::mouse::MouseButton;
use events::{UiEvent, MouseClick, GlobalInput, WidgetInput, InputProvider};
use widget::{Index, Id};
use position::Rect;

// Pushes an event onto the given global input with a default drag threshold.
fn push_event(input: &mut GlobalInput, event: UiEvent) {
    input.push_event(event);
}

#[test]
fn mouse_button_down_should_return_none_if_mouse_is_not_over_widget() {
    let widget_area = Rect::from_corners([10.0, 10.0], [50.0, 50.0]);
    let mut global_input = GlobalInput::new();
    // mouse position stays at (0,0)
    push_event(&mut global_input, UiEvent::Raw(Input::Press(Button::Mouse(MouseButton::Left))));

    let widget_input = WidgetInput::for_widget(Index::Public(Id(2)), widget_area, &global_input);

    assert!(widget_input.mouse_left_button_down().is_none());
}

#[test]
fn mouse_button_down_should_return_none_if_another_widget_is_capturing_mouse() {
    let widget_area = Rect::from_corners([10.0, 10.0], [50.0, 50.0]);
    let mut global_input = GlobalInput::new();
    push_event(&mut global_input, UiEvent::WidgetCapturesMouse(Index::Public(Id(999))));
    push_event(&mut global_input, UiEvent::Raw(Input::Move(Motion::MouseRelative(30.0, 30.0))));
    push_event(&mut global_input, UiEvent::Raw(Input::Press(Button::Mouse(MouseButton::Left))));

    let widget_input = WidgetInput::for_widget(Index::Public(Id(2)), widget_area, &global_input);

    assert!(widget_input.mouse_left_button_down().is_none());
}

#[test]
fn maybe_mouse_position_should_return_none_if_mouse_is_not_over_the_widget() {
    let widget_area = Rect::from_corners([10.0, 10.0], [50.0, 50.0]);
    let mut global_input = GlobalInput::new();
    push_event(&mut global_input, UiEvent::Raw(Input::Move(Motion::MouseRelative(-10.0, -10.0))));

    let widget_input = WidgetInput::for_widget(Index::Public(Id(2)), widget_area, &global_input);

    let maybe_mouse_position = widget_input.maybe_mouse_position();
    assert!(maybe_mouse_position.is_none());
}

#[test]
fn mouse_is_over_widget_should_be_false_if_mouse_is_not_over_widget() {
    let widget_area = Rect::from_corners([10.0, 10.0], [50.0, 50.0]);
    let mut global_input = GlobalInput::new();
    push_event(&mut global_input, UiEvent::Raw(Input::Move(Motion::MouseRelative(90.0, 90.0))));

    let widget_input = WidgetInput::for_widget(Index::Public(Id(2)), widget_area, &global_input);

    assert!(!widget_input.mouse_is_over_widget());
}

#[test]
fn widget_input_should_provide_any_mouse_events_over_the_widgets_area_if_nothing_is_capturing_mouse() {
    let mut global_input = GlobalInput::new();
    push_event(&mut global_input, UiEvent::MouseClick(MouseClick{
        button: MouseButton::Left,
        xy: [10.0, 10.0],
        modifiers: NO_MODIFIER
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
fn mouse_clicks_should_be_relative_to_widget_position() {
    let idx = Index::Public(Id(5));
    let mut global_input = GlobalInput::new();
    push_event(&mut global_input, UiEvent::MouseClick(MouseClick{
        button: MouseButton::Left,
        xy: [10.0, 10.0],
        modifiers: NO_MODIFIER
    }));

    let rect = Rect::from_corners([0.0, 0.0], [20.0, 20.0]);
    let widget_input = WidgetInput::for_widget(idx, rect, &global_input);
    let widget_click = widget_input.mouse_left_click().expect("widget click should not be null");
    assert_eq!([0.0, 0.0], widget_click.xy);
}

#[test]
fn mouse_drags_should_be_relative_to_widget_position() {
    use events::MouseDrag;

    let idx = Index::Public(Id(5));
    let mut global_input = GlobalInput::new();
    push_event(&mut global_input, UiEvent::MouseDrag(MouseDrag{
        button: MouseButton::Left,
        start: [5.0, 5.0],
        end: [10.0, 10.0],
        modifiers: NO_MODIFIER,
    }));

    let rect = Rect::from_corners([0.0, 0.0], [20.0, 20.0]);
    let widget_input = WidgetInput::for_widget(idx, rect, &global_input);
    let drag = widget_input.mouse_left_drag().expect("expected a mouse drag event");
    assert_eq!([-5.0, -5.0], drag.start);
    assert_eq!([0.0, 0.0], drag.end);

}
