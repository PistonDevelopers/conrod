//! Contains all the logic for filtering input events and making them relative to widgets.
//! The core of this module is the `WidgetInput::for_widget` method, which creates an
//! `InputProvider` that provides input events for a specific widget.

use widget::Index;
use events::{InputState, ConrodEvent, GlobalInput, InputProvider, RelativePosition};
use position::{Point, Rect};

/// Holds any events meant to be given to a `Widget`. This is what widgets will interface with
/// when handling events in their `update` method. All events returned from methods on `WidgetInput`
/// will be relative to the widget's own (0,0) origin. Additionally, `WidgetInput` will not provide
/// mouse or keyboard events that do not directly pertain to the widget.
pub struct WidgetInput {
    events: Vec<ConrodEvent>,
    current_state: InputState,
    widget_area: Rect,
}

impl WidgetInput {
    /// Returns a `WidgetInput` with events specifically for the given widget.
    /// Filters out only the events that directly pertain to the widget.
    /// All events will also be made relative to the widget's own (0,0) origin.
    pub fn for_widget(widget: Index, widget_area: Rect, global_input: &GlobalInput) -> WidgetInput {
        let widget_xy =  widget_area.xy();
        // we start out using the start_state instead of current_state so that each event
        // will be interpreted in the correct context. Otherwise, if a user clicked and then moved
        // the mouse very quickly, we would end up registering the click in the wrong location.
        let mut current_state = global_input.start_state.clone();
        let widget_events = global_input.all_events()
            .filter(move |evt| {
                // filtering is done using the window coordinate system
                current_state.update(evt);
                should_provide_event(widget, widget_area, &evt, &current_state)
            })
            // then make events relative to widget coordinates
            .map(|evt| evt.relative_to(widget_xy))
            .collect::<Vec<ConrodEvent>>();

        WidgetInput {
            events: widget_events,
            current_state: global_input.current_state.relative_to(widget_xy),
            widget_area: widget_area,
        }
    }

    /// Returns true if the mouse is currently over the widget, otherwise false
    pub fn mouse_is_over_widget(&self) -> bool {
        self.point_is_over(self.current_mouse_position())
    }

    /// If the mouse is over the widget and no other widget is capturing the mouse, then
    /// this will return the position of the mouse relative to the widget. Otherwise, it
    /// will return `None`
    pub fn maybe_mouse_position(&self) -> Option<Point> {
        if self.mouse_is_over_widget() {
            Some(self.current_mouse_position())
        } else {
            None
        }
    }

    fn point_is_over(&self, point: Point) -> bool {
        self.widget_relative_rect().is_over(point)
    }

    fn widget_relative_rect(&self) -> Rect {
        let widget_dim = self.widget_area.dim();
        Rect::from_xy_dim([0.0, 0.0], widget_dim)
    }
}

pub type WidgetInputEventIterator<'a> = ::std::slice::Iter<'a, ConrodEvent>;

impl<'a> InputProvider<'a, WidgetInputEventIterator<'a>> for WidgetInput {
    fn all_events(&'a self) -> WidgetInputEventIterator<'a> {
        self.events.iter()
    }

    fn current_state(&self) -> &InputState {
        &self.current_state
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
