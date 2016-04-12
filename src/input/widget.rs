//! Contains all the logic for filtering input events and making them relative to widgets.
//!
//! The core of this module is the `Widget::for_widget` method, which creates an
//! `InputProvider` that provides input events for a specific widget.

use {Point, Rect};
use widget;
use event;
use input;


/// Provides only events and input state that are relevant to a specific widget.
///
/// This type can be produced by calling the `UiCell::input` method with the target widget's
/// `widget::Index`. This is particularly useful
///
/// Unlike `input::Global`, `input::Widget` methods are tailored to the widget for which they are
/// produced.
#[derive(Clone)]
pub struct Widget<'a> {
    global: &'a input::Global,
    rect: Rect,
    idx: widget::Index,
}

/// A view of the `input::state::Mouse` that is specific to a single widget.
#[derive(Copy, Clone, Debug)]
pub struct Mouse<'a> {
    rect: Rect,
    mouse_abs_xy: Point,
    /// The state of each `MouseButton`.
    pub buttons: &'a input::state::mouse::ButtonMap,
}

/// An iterator yielding all events that are relevant to a specific widget.
///
/// All events provided by this Iterator will be filtered in accordance with input capturing. For
/// example: If the widget does not capture the mouse, it *will not* receive any mouse-related
/// events. If the widget captures the keyboard it *will* receive all keyboard events.
///
/// All mouse events will have their coordinates relative to the middle of the widget's `Rect`.
#[derive(Clone)]
pub struct Events<'a> {
    global_events: input::global::Events<'a>,
    capturing_keyboard: Option<widget::Index>,
    capturing_mouse: Option<widget::Index>,
    rect: Rect,
    idx: widget::Index,
}

/// An `Iterator` yielding all mouse clicks occuring within the given sequence of `widget::Event`s.
#[derive(Clone)]
pub struct Clicks<'a> {
    events: Events<'a>,
}

/// An `Iterator` yielding all mouse `button` clicks occuring within the given sequence of
/// `widget::Click`s.
#[derive(Clone)]
pub struct ButtonClicks<'a> {
    clicks: Clicks<'a>,
    button: input::MouseButton,
}

/// An iterator that yields all `event::Drag` events yielded by the `Events` iterator.
///
/// Only events that occurred while the widget was capturing the device that did the dragging will
/// be yielded.
#[derive(Clone)]
pub struct Drags<'a> {
    events: Events<'a>,
}

/// An `Iterator` yielding all mouse `button` drags occuring within the given sequence of
/// `widget::Drag`s.
#[derive(Clone)]
pub struct ButtonDrags<'a> {
    drags: Drags<'a>,
    button: input::MouseButton,
}

/// An iterator that yields all `Input::Text` events yielded by the `Events` iterator.
///
/// Only events that occurred while the widget was capturing the keyboard will be yielded.
#[derive(Clone)]
pub struct Texts<'a> {
    events: Events<'a>,
}


impl<'a> Widget<'a> {

    /// Returns a `Widget` with events specifically for the given widget.
    ///
    /// Filters out only the events that directly pertain to the widget.
    ///
    /// All events will also be made relative to the widget's own (0, 0) origin.
    pub fn for_widget(idx: widget::Index, rect: Rect, global: &'a input::Global) -> Self {
        Widget {
            global: global,
            rect: rect,
            idx: idx,
        }
    }

    /// If the widget is currently capturing the mouse, this returns the state of the mouse.
    ///
    /// Returns `None` if the widget is not capturing the mouse.
    pub fn mouse(&self) -> Option<Mouse> {
        if self.global.current.widget_capturing_mouse == Some(self.idx) {
            let mouse = Mouse {
                buttons: &self.global.current.mouse.buttons,
                mouse_abs_xy: self.global.current.mouse.xy,
                rect: self.rect,
            };
            return Some(mouse);
        }
        None
    }

    /// Produces an iterator yielding all events that are relevant to a specific widget.
    ///
    /// All events provided by this Iterator will be filtered in accordance with input capturing. For
    /// example: If the widget does not capture the mouse, it *will not* receive any mouse-related
    /// events. If the widget captures the keyboard it *will* receive all keyboard events.
    ///
    /// All mouse events will have their coordinates relative to the middle of the widget's `Rect`.
    pub fn events(&self) -> Events {
        Events {
            global_events: self.global.events(),
            capturing_keyboard: self.global.start.widget_capturing_keyboard,
            capturing_mouse: self.global.start.widget_capturing_mouse,
            rect: self.rect,
            idx: self.idx,
        }
    }

    /// Filters all events yielded by `Self::events` for all `event::Click`s.
    ///
    /// A _click_ is determined to have occured if a pointing device button was both pressed and
    /// released over the widget.
    pub fn clicks(&self) -> Clicks {
        Clicks { events: self.events() }
    }

    /// Produces an iterator that yields all `event::Drag` events yielded by the `Events` iterator.
    ///
    /// Only events that occurred while the widget was capturing the device that did the dragging
    /// will be yielded.
    pub fn drags(&self) -> Drags {
        Drags { events: self.events() }
    }

    /// Produces an iterator that yields all `Input::Text` events that have occurred as `&str`s
    /// since the last time `Ui::set_widgets` was called.
    ///
    /// Only events that occurred while the widget was capturing the keyboard will be yielded.
    pub fn text(&self) -> Texts {
        Texts { events: self.events() }
    }

}

impl<'a> Mouse<'a> {
    /// The absolute position of the mouse within the window.
    pub fn abs_xy(&self) -> Point {
        self.mouse_abs_xy
    }

    /// The position of the mouse relative to the middle of the widget's `Rect`.
    pub fn rel_xy(&self) -> Point {
        ::vecmath::vec2_sub(self.mouse_abs_xy, self.rect.xy())
    }

    /// Is the mouse currently over the widget.
    pub fn is_over(&self) -> bool {
        self.rect.is_over(self.mouse_abs_xy)
    }
}

impl<'a> Clicks<'a> {

    /// Yield only the `Click`s that occurred from the given button.
    pub fn button(self, button: input::MouseButton) -> ButtonClicks<'a> {
        ButtonClicks {
            clicks: self,
            button: button,
        }
    }

    /// Yield only left mouse button `Click`s.
    pub fn left(self) -> ButtonClicks<'a> {
        self.button(input::MouseButton::Left)
    }

    /// Yields only middle mouse button `Click`s.
    pub fn middle(self) -> ButtonClicks<'a> {
        self.button(input::MouseButton::Middle)
    }

    /// Yield only right mouse button `Click`s.
    pub fn right(self) -> ButtonClicks<'a> {
        self.button(input::MouseButton::Right)
    }

}

impl<'a> Drags<'a> {

    /// Yield only the `Drag`s that occurred from the given button.
    pub fn button(self, button: input::MouseButton) -> ButtonDrags<'a> {
        ButtonDrags {
            drags: self,
            button: button,
        }
    }

    /// Yield only left mouse button `Drag`s.
    pub fn left(self) -> ButtonDrags<'a> {
        self.button(input::MouseButton::Left)
    }
    
    /// Yields only middle mouse button `Drag`s.
    pub fn middle(self) -> ButtonDrags<'a> {
        self.button(input::MouseButton::Middle)
    }

    /// Yield only right mouse button `Drag`s.
    pub fn right(self) -> ButtonDrags<'a> {
        self.button(input::MouseButton::Right)
    }

}


impl<'a> Iterator for Events<'a> {
    type Item = event::Widget;

    fn next(&mut self) -> Option<event::Widget> {
        use event::{Input, Motion};
        use input::Button;
        let widget_xy = self.rect.xy();

        // Loop through all events in the `global_events` until we find one associated with our
        // widget that we can return.
        while let Some(event) = self.global_events.next() {
            let is_capturing_mouse = self.capturing_mouse == Some(self.idx);
            let is_capturing_keyboard = self.capturing_keyboard == Some(self.idx);

            match *event {

                // Raw input events.
                event::Event::Raw(ref input) => match *input {

                    Input::Move(ref motion) => match *motion {
                        Motion::MouseCursor(x, y) if is_capturing_mouse => {
                            let rel = ::vecmath::vec2_sub([x, y], widget_xy);
                            return Some(Input::Move(Motion::MouseCursor(rel[0], rel[1])).into())
                        },
                        Motion::MouseRelative(_, _) |
                        Motion::MouseScroll(_, _) if is_capturing_mouse =>
                            return Some(input.clone().into()),
                        _ => (),
                    },

                    Input::Text(_) if is_capturing_keyboard =>
                        return Some(input.clone().into()),

                    Input::Press(Button::Keyboard(_)) |
                    Input::Release(Button::Keyboard(_)) if is_capturing_keyboard =>
                        return Some(input.clone().into()),

                    Input::Press(Button::Mouse(_)) |
                    Input::Release(Button::Mouse(_)) if is_capturing_mouse =>
                        return Some(input.clone().into()),

                    Input::Cursor(_) if is_capturing_mouse =>
                        return Some(input.clone().into()),

                    Input::Resize(_, _) | Input::Focus(_) =>
                        return Some(input.clone().into()),

                    _ => (),
                },

                // Interpreted events.
                event::Event::Ui(ref ui_event) => match *ui_event {

                    // Mouse capturing.
                    event::Ui::WidgetCapturesMouse(idx) => {
                        self.capturing_mouse = Some(idx);
                        if idx == self.idx {
                            return Some(event::Widget::CapturesMouse);
                        }
                    },
                    event::Ui::WidgetUncapturesMouse(idx) => {
                        if Some(idx) == self.capturing_mouse {
                            self.capturing_mouse = None;
                        }
                        if idx == self.idx {
                            return Some(event::Widget::UncapturesMouse);
                        }
                    },

                    // Keyboard capturing.
                    event::Ui::WidgetCapturesKeyboard(idx) => {
                        self.capturing_keyboard = Some(idx);
                        if idx == self.idx {
                            return Some(event::Widget::CapturesKeyboard);
                        }
                    },
                    event::Ui::WidgetUncapturesKeyboard(idx) => {
                        if Some(idx) == self.capturing_keyboard {
                            self.capturing_keyboard = None;
                        }
                        if idx == self.idx {
                            return Some(event::Widget::UncapturesKeyboard);
                        }
                    },

                    event::Ui::Click(idx, ref click) if idx == Some(self.idx) =>
                        return Some(click.clone().into()),

                    event::Ui::Drag(idx, ref drag) if idx == Some(self.idx) =>
                        return Some(drag.clone().into()),

                    event::Ui::Scroll(idx, ref scroll) if idx == Some(self.idx) =>
                        return Some(scroll.clone().into()),

                    _ => (),
                },
            }
        }

        None
    }
}


impl<'a> Iterator for Clicks<'a> {
    type Item = event::Click;
    fn next(&mut self) -> Option<event::Click> {
        while let Some(event) = self.events.next() {
            if let event::Widget::Click(click) = event {
                return Some(click);
            }
        }
        None
    }
}

impl<'a> Iterator for ButtonClicks<'a> {
    type Item = event::Click;
    fn next(&mut self) -> Option<event::Click> {
        while let Some(click) = self.clicks.next() {
            if self.button == click.button {
                return Some(click);
            }
        }
        None
    }
}

impl<'a> Iterator for Drags<'a> {
    type Item = event::Drag;
    fn next(&mut self) -> Option<event::Drag> {
        while let Some(event) = self.events.next() {
            if let event::Widget::Drag(drag) = event {
                return Some(drag);
            }
        }
        None
    }
}

impl<'a> Iterator for ButtonDrags<'a> {
    type Item = event::Drag;
    fn next(&mut self) -> Option<event::Drag> {
        while let Some(drag) = self.drags.next() {
            if self.button == drag.button {
                return Some(drag);
            }
        }
        None
    }
}

impl<'a> Iterator for Texts<'a> {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        while let Some(event) = self.events.next() {
            if let event::Widget::Raw(event::Input::Text(string)) = event {
                return Some(string);
            }
        }
        None
    }
}
