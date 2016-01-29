mod tests;

use widget::{Index, Id};
use events::{InputState, ConrodEvent, GlobalInput, EventProvider, RelativePosition};
use position::{Point, Range, Rect};
use input::keyboard::ModifierKey;

/// Holds any events meant to be given to a `Widget`. This is what widgets will interface with
/// when handling events in their `update` method. All events returned from methods on `WidgetInput`
/// will be relative to the widget's own (0,0) origin. Additionally, `WidgetInput` will not provide
/// mouse or keyboard events that do not directly pertain to the widget.
pub struct WidgetInput(Vec<ConrodEvent>);

impl WidgetInput {
    /// Returns a `WidgetInput` with events specifically for the given widget.
    /// Filters out only the events that directly pertain to the widget.
    /// All events will also be made relative to the widget's own (0,0) origin.
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

        WidgetInput(widget_events)
    }
}

impl EventProvider for WidgetInput {
    fn all_events(&self) -> &Vec<ConrodEvent> {
        &self.0
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
