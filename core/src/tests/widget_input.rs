use event::{self, Input};
use input::{self, Button, Motion, MouseButton};
use input::keyboard::ModifierKey;
use widget;
use position::Rect;

// Pushes an event onto the given global input with a default drag threshold.
fn push_event(input: &mut input::Global, event: event::Event) {
    input.push_event(event);
}


#[test]
fn mouse_should_return_none_if_another_widget_is_capturing_mouse() {
    let widget_area = Rect::from_corners([10.0, 10.0], [50.0, 50.0]);
    let mut global_input = input::Global::new();
    let source = input::Source::Mouse;
    push_event(&mut global_input, event::Ui::WidgetCapturesInputSource(widget::Id::new(999), source).into());
    push_event(&mut global_input, event::Event::Raw(Input::Motion(Motion::MouseRelative { x: 30.0, y: 30. })));
    push_event(&mut global_input, event::Event::Raw(Input::Press(Button::Mouse(MouseButton::Left))));

    let widget_input = input::Widget::for_widget(widget::Id::new(2), widget_area, &global_input);

    assert!(widget_input.mouse().is_none());
}

#[test]
fn widget_input_should_provide_any_mouse_events_over_the_widgets_area_if_nothing_is_capturing_mouse() {
    let mut global_input = input::Global::new();
    let widget = widget::Id::new(4);

    push_event(&mut global_input, event::Ui::Click(Some(widget), event::Click{
        button: MouseButton::Left,
        xy: [10.0, 10.0],
        modifiers: ModifierKey::NO_MODIFIER,
    }).into());
    assert!(global_input.current.widget_capturing_mouse.is_none());

    let widget_area = Rect::from_corners([0.0, 0.0], [40.0, 40.0]);
    let widget_input = input::Widget::for_widget(widget, widget_area, &global_input);

    widget_input.clicks().left().next().expect("Expected to get a mouse click event");

    let another_widget = widget::Id::new(7);
    let another_area = Rect::from_corners([-20.0, -20.0], [0.0, 0.0]);
    let another_widget_input = input::Widget::for_widget(another_widget, another_area, &global_input);

    assert!(another_widget_input.clicks().left().next().is_none());
}
