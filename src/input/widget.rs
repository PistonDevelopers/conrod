//! Contains all the logic for filtering input events and making them relative to widgets.
//!
//! The core of this module is the `Widget::for_widget` method, which creates an
//! `InputProvider` that provides input events for a specific widget.

use widget::Index;
use event::{self, UiEvent};
use position::{Point, Rect};
use input::{self, MouseButton, Provider};

/// Holds any events meant to be given to a `Widget`.
///
/// This is what widgets will interface with when handling events in their `update` method.
///
/// All events returned from methods on `Widget` will be relative to the widget's own (0,0)
/// origin. Additionally, `Widget` will not provide mouse or keyboard events that do not
/// directly pertain to the widget.
pub struct Widget<'a> {
    global_input: &'a input::Global,
    current: input::State,
    widget_area: Rect,
    widget_idx: Index,
}

impl<'a> Widget<'a> {

    /// Returns a `Widget` with events specifically for the given widget.
    ///
    /// Filters out only the events that directly pertain to the widget.
    ///
    /// All events will also be made relative to the widget's own (0,0) origin.
    pub fn for_widget(widget: Index, widget_area: Rect, global_input: &'a input::Global) -> Self {
        Widget {
            global_input: &global_input,
            widget_area: widget_area,
            widget_idx: widget,
            current: global_input.current.relative_to(widget_area.xy())
        }
    }

    /// Returns true if the mouse is currently over the widget, otherwise false
    pub fn mouse_is_over_widget(&self) -> bool {
        self.point_is_over(self.mouse_position())
    }

    /// If the mouse is over the widget and no other widget is capturing the mouse, then
    /// this will return the position of the mouse relative to the widget. Otherwise, it
    /// will return `None`
    pub fn maybe_mouse_position(&self) -> Option<Point> {
        if self.mouse_is_over_widget() {
            Some(self.mouse_position())
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

/// Alows iterating over events for a specific widget. All events provided by this Iterator
/// will be filtered, so that input intended for other widgets is excluded. In addition,
/// all mouse events will have their coordinates relative to the widget's own (0,0) origin.
#[derive(Clone)]
pub struct WidgetEventIterator<'a> {
    global_event_iter: input::GlobalEventIterator<'a>,
    current: input::State,
    widget_area: Rect,
    widget_idx: Index,
}

impl<'a> Iterator for WidgetEventIterator<'a> {
    type Item = &'a UiEvent;
    fn next(&mut self) -> Option<&'a UiEvent> {
        self.global_event_iter.next().and_then(|event| {
            if should_provide_event(self.widget_idx, self.widget_area, event, &self.current) {
                Some(event)
            } else {
                self.next()
            }
        })
    }
}


impl<'a> input::Provider<'a> for Widget<'a> {
    type Events = WidgetEventIterator<'a>;

    fn events(&'a self) -> Self::Events {
        WidgetEventIterator{
            global_event_iter: self.global_input.events(),
            current: self.global_input.start.relative_to(self.widget_area.xy()),
            widget_area: self.widget_area,
            widget_idx: self.widget_idx,
        }
    }

    fn current(&'a self) -> &'a input::State {
        &self.current
    }

    fn mouse_click(&'a self, button: MouseButton) -> Option<event::MouseClick> {
        self.events().filter_map(|event| {
            match *event {
                UiEvent::MouseClick(click) if click.button == button => {
                    Some(click.relative_to(self.widget_area.xy()))
                },
                _ => None
            }
        }).next()
    }

    fn mouse_drag(&'a self, button: MouseButton) -> Option<event::MouseDrag> {
        self.events().filter_map(|evt| {
            match *evt {
                UiEvent::MouseDrag(drag_evt) if drag_evt.button == button => {
                    Some(drag_evt.relative_to(self.widget_area.xy()))
                },
                _ => None
            }
        }).last()
    }

    fn mouse_button_down(&self, button: MouseButton) -> Option<Point> {
        let current = self.current();
        let self_is_capturing = || Some(self.widget_idx) == current.widget_capturing_mouse;
        current.mouse.buttons[button].xy_if_down()
            .and_then(|xy| if self_is_capturing() { Some(xy) } else { None })
    }

}

fn should_provide_event(widget: Index,
                        widget_area: Rect,
                        event: &UiEvent,
                        current: &input::State) -> bool {
    let is_keyboard = event.is_keyboard_event();
    let is_mouse = event.is_mouse_event();

    (is_keyboard && current.widget_capturing_keyboard == Some(widget))
            || (is_mouse && should_provide_mouse_event(widget, widget_area, event, current))
            || (!is_keyboard && !is_mouse)
}

fn should_provide_mouse_event(widget: Index,
                            widget_area: Rect,
                            event: &UiEvent,
                            current: &input::State) -> bool {
    let capturing_mouse = current.widget_capturing_mouse;
    match capturing_mouse {
        Some(idx) if idx == widget => true,
        None => mouse_event_is_over_widget(widget_area, event, current),
        _ => false
    }
}

fn mouse_event_is_over_widget(widget_area: Rect, event: &UiEvent, current: &input::State) -> bool {
    match *event {
        UiEvent::MouseClick(click) => widget_area.is_over(click.xy),
        UiEvent::MouseDrag(drag) => {
            widget_area.is_over(drag.start) || widget_area.is_over(drag.end)
        },
        _ => widget_area.is_over(current.mouse.xy)
    }
}
