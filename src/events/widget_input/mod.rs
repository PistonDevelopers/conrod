use widget::{Index, Id};
use events::{InputState, ConrodEvent, GlobalInput, WidgetEvents, RelativePosition};
use position::{Point, Range, Rect};
use input::keyboard::ModifierKey;

pub struct WidgetInput {
    events: Vec<ConrodEvent>,
    start_state: InputState,
    widget: Index,
    widget_area: Rect,
}

impl WidgetInput {
    pub fn for_widget(widget: Index, widget_area: Rect, global_input: &GlobalInput) -> WidgetInput {
        let start_state = global_input.starting_state();
        let mut current_state = start_state.clone();
        let widget_events = global_input.all_events().iter()
            .filter(move |evt| {
                current_state.update(evt);
                should_provide_event(widget, widget_area, &evt, &current_state)
            })
            .map(|evt| evt.relative_to(widget_area.xy()))
            .collect::<Vec<ConrodEvent>>();

        WidgetInput{
            events: widget_events,
            start_state: global_input.start_state,
            widget: widget,
            widget_area: widget_area
        }
    }
}

impl WidgetEvents for WidgetInput {
    fn all_events(&self) -> &Vec<ConrodEvent> {
        &self.events
    }
}

fn should_provide_event(widget: Index,
                        widget_area: Rect,
                        event: &ConrodEvent,
                        current_state: &InputState) -> bool {
    let is_keyboard = event.is_keyboard_event();
    let is_mouse = event.is_mouse_event();

    (is_keyboard && current_state.widget_capturing_keyboard == Some(widget))
            || (is_mouse && should_provide_mouse_event(widget, widget_area, event, current_state))
            || (!is_keyboard && !is_mouse)
}

fn should_provide_mouse_event(widget: Index,
                            widget_area: Rect,
                            event: &ConrodEvent,
                            current_state: &InputState) -> bool {
    let capturing_mouse = current_state.widget_capturing_mouse;
    match capturing_mouse {
        Some(idx) if idx == widget => true,
        None => mouse_event_is_over_widget(widget_area, event, current_state),
        _ => false
    }
}

fn mouse_event_is_over_widget(widget_area: Rect, event: &ConrodEvent, current_state: &InputState) -> bool {
    match *event {
        ConrodEvent::MouseClick(click) => widget_area.is_over(click.location),
        ConrodEvent::MouseDrag(drag) => {
            widget_area.is_over(drag.start) || widget_area.is_over(drag.end)
        },
        _ => widget_area.is_over(current_state.mouse_position)
    }
}

#[test]
fn widget_input_should_provide_any_mouse_events_over_the_widgets_area_if_nothing_is_capturing_mouse() {
    use input::Input;
    use input::mouse::MouseButton;
    use input::keyboard::NO_MODIFIER;
    use events::MouseClick;

    let mut global_input = GlobalInput::new();
    global_input.push_event(ConrodEvent::MouseClick(MouseClick{
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
    use input::Input;

    let mut global_input = GlobalInput::new();

    let some_rect = Rect::from_corners([0.0, 0.0], [40.0, 40.0]);
    let widget_4 = Index::Public(Id(4));
    global_input.push_event(ConrodEvent::WidgetCapturesKeyboard(widget_4));
    global_input.push_event(ConrodEvent::Raw(Input::Text("some text".to_string())));

    let widget_4_input = WidgetInput::for_widget(widget_4, some_rect, &global_input);
    let widget_4_text = widget_4_input.text_just_entered();
    assert_eq!(Some("some text".to_string()), widget_4_text);

    let another_widget_input = WidgetInput::for_widget(Index::Public(Id(7)),
            some_rect,
            &global_input);
    assert!(another_widget_input.text_just_entered().is_none());
}

#[test]
fn widget_input_events_should_be_relative_to_widget_position() {
    use input::mouse::MouseButton;
    use input::keyboard::NO_MODIFIER;
    use events::MouseClick;

    let idx = Index::Public(Id(5));
    let mut global_input = GlobalInput::new();
    global_input.push_event(ConrodEvent::WidgetCapturesMouse(idx));
    global_input.push_event(ConrodEvent::MouseClick(MouseClick{
        button: MouseButton::Left,
        location: [10.0, 10.0],
        modifier: NO_MODIFIER
    }));

    let rect = Rect::from_corners([0.0, 0.0], [20.0, 20.0]);
    let widget_input = WidgetInput::for_widget(idx, rect, &global_input);
    let widget_click = widget_input.mouse_left_click().expect("widget click should not be null");
    assert_eq!([0.0, 0.0], widget_click.location);
}
