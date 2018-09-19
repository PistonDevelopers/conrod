//! Contains all the logic for filtering input events and making them relative to widgets.
//!
//! The core of this module is the `Widget::for_widget` method, which creates an
//! `InputProvider` that provides input events for a specific widget.

use {Point, Rect};
use event;
use input;
use utils;
use widget;


/// Provides only events and input state that are relevant to a specific widget.
///
/// This type can be produced by calling the `UiCell::input` method with the target widget's
/// `widget::Id`. This is particularly useful
///
/// Unlike `input::Global`, `input::Widget` methods are tailored to the widget for which they are
/// produced.
#[derive(Clone)]
pub struct Widget<'a> {
    global: &'a input::Global,
    rect: Rect,
    idx: widget::Id,
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
    ui_events: input::global::UiEvents<'a>,
    capturing_keyboard: Option<widget::Id>,
    capturing_mouse: Option<widget::Id>,
    rect: Rect,
    idx: widget::Id,
}

/// An `Iterator` yielding all button presses occuring within the given sequence of
/// `widget::Event`s.
#[derive(Clone)]
pub struct Presses<'a> {
    events: Events<'a>,
}

/// An `Iterator` yielding all mouse button presses occuring within the given sequence of `Presses`.
#[derive(Clone)]
pub struct MousePresses<'a> {
    presses: Presses<'a>,
}

/// An `Iterator` yielding all mouse button presses occuring within the given sequence of
/// `Presses` for some specific mouse button.
#[derive(Clone)]
pub struct MouseButtonPresses<'a> {
    mouse_presses: MousePresses<'a>,
    button: input::MouseButton,
}

/// An `Iterator` yielding all keyboard button presses occuring within the given sequence of
/// `Presses`.
#[derive(Clone)]
pub struct KeyPresses<'a> {
    presses: Presses<'a>,
}

/// An `Iterator` yielding all button releases occuring within the given sequence of
/// `widget::Event`s.
#[derive(Clone)]
pub struct Releases<'a> {
    events: Events<'a>,
}

/// An `Iterator` yielding all mouse button releases occuring within the given sequence of
/// `Releases` for some specific mouse button.
#[derive(Clone)]
pub struct MouseButtonReleases<'a> {
    mouse_releases: MouseReleases<'a>,
    button: input::MouseButton,
}

/// An `Iterator` yielding all mouse button releases occuring within the given sequence of
/// `Releases`.
#[derive(Clone)]
pub struct MouseReleases<'a> {
    releases: Releases<'a>,
}

/// An `Iterator` yielding all keyboard button releases occuring within the given sequence of
/// `Releases`.
#[derive(Clone)]
pub struct KeyReleases<'a> {
    releases: Releases<'a>,
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

/// An `Iterator` yielding all touch screen taps occuring within the given sequence of
/// `widget::Event`s.
#[derive(Clone)]
pub struct Taps<'a> {
    events: Events<'a>,
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

/// An iterator that yields all `Scroll` events yielded by the given `Events` iterator.
#[derive(Clone)]
pub struct Scrolls<'a> {
    events: Events<'a>,
}


impl<'a> Widget<'a> {

    /// Returns a `Widget` with events specifically for the given widget.
    ///
    /// Filters out only the events that directly pertain to the widget.
    ///
    /// All events will also be made relative to the widget's own (0, 0) origin.
    pub fn for_widget(idx: widget::Id, rect: Rect, global: &'a input::Global) -> Self {
        Widget {
            global: global,
            rect: rect,
            idx: idx,
        }
    }

    /// If the widget is currently capturing the mouse, this returns the state of the mouse.
    ///
    /// Returns `None` if the widget is not capturing the mouse.
    pub fn mouse(&self) -> Option<Mouse<'a>> {
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
    pub fn events(&self) -> Events<'a> {
        Events {
            ui_events: self.global.events().ui(),
            capturing_keyboard: self.global.start.widget_capturing_keyboard,
            capturing_mouse: self.global.start.widget_capturing_mouse,
            rect: self.rect,
            idx: self.idx,
        }
    }

    /// Filters all events yielded by `Self::events` other than `event::Press`es.
    pub fn presses(&self) -> Presses<'a> {
        Presses { events: self.events() }
    }

    /// Filters all events yielded by `Self::events` other than `event::Release`es.
    pub fn releases(&self) -> Releases<'a> {
        Releases { events: self.events() }
    }

    /// Filters all events yielded by `Self::events` for all `event::Click`s.
    ///
    /// A _click_ is determined to have occured if a pointing device button was both pressed and
    /// released over the widget.
    pub fn clicks(&self) -> Clicks<'a> {
        Clicks { events: self.events() }
    }

    /// Filters all events yielded by `Self::events` for all `event::Tap`s.
    ///
    /// A _tap_ is determined to have occured if a touch interaction both started and ended over
    /// the widget.
    pub fn taps(&self) -> Taps<'a> {
        Taps { events: self.events() }
    }

    /// Produces an iterator that yields all `event::Drag` events yielded by the `Events` iterator.
    ///
    /// Only events that occurred while the widget was capturing the device that did the dragging
    /// will be yielded.
    pub fn drags(&self) -> Drags<'a> {
        Drags { events: self.events() }
    }

    /// Produces an iterator that yields all `Input::Text` events that have occurred as `&str`s
    /// since the last time `Ui::set_widgets` was called.
    ///
    /// Only events that occurred while the widget was capturing the keyboard will be yielded.
    pub fn texts(&self) -> Texts<'a> {
        Texts { events: self.events() }
    }

    /// Produce an iterator that yields only the `Scroll` events yielded by the `Events` iterator.
    pub fn scrolls(&self) -> Scrolls<'a> {
        Scrolls { events: self.events() }
    }

}

impl<'a> Mouse<'a> {

    /// The absolute position of the mouse within the window.
    pub fn abs_xy(&self) -> Point {
        self.mouse_abs_xy
    }

    /// The position of the mouse relative to the middle of the widget's `Rect`.
    pub fn rel_xy(&self) -> Point {
        utils::vec2_sub(self.mouse_abs_xy, self.rect.xy())
    }

    /// Is the mouse currently over the widget.
    pub fn is_over(&self) -> bool {
        self.rect.is_over(self.mouse_abs_xy)
    }

}

impl<'a> Presses<'a> {

    /// Produces an `Iterator` that yields only the press events that correspond with mouse buttons.
    pub fn mouse(self) -> MousePresses<'a> {
        MousePresses {
            presses: self,
        }
    }

    /// Produces an `Iterator` that yields only the press events that correspond with keyboard
    /// buttons.
    pub fn key(self) -> KeyPresses<'a> {
        KeyPresses {
            presses: self,
        }
    }

}

impl<'a> MousePresses<'a> {

    /// Produces an `Iterator` that yields only events associated with the given mouse button.
    pub fn button(self, button: input::MouseButton) -> MouseButtonPresses<'a> {
        MouseButtonPresses {
            mouse_presses: self,
            button: button,
        }
    }

    /// Produces an `Iterator` that yields only the left mouse button press events.
    pub fn left(self) -> MouseButtonPresses<'a> {
        self.button(input::MouseButton::Left)
    }

    /// Produces an `Iterator` that yields only the middle mouse button press events.
    pub fn middle(self) -> MouseButtonPresses<'a> {
        self.button(input::MouseButton::Middle)
    }

    /// Produces an `Iterator` that yields only the right mouse button press events.
    pub fn right(self) -> MouseButtonPresses<'a> {
        self.button(input::MouseButton::Right)
    }

}

impl<'a> Releases<'a> {

    /// Produces an `Iterator` that yields only the release events that correspond with mouse
    /// buttons.
    pub fn mouse(self) -> MouseReleases<'a> {
        MouseReleases {
            releases: self,
        }
    }

    /// Produces an `Iterator` that yields only the release events that correspond with keyboard
    /// buttons.
    pub fn key(self) -> KeyReleases<'a> {
        KeyReleases {
            releases: self,
        }
    }

}

impl<'a> MouseReleases<'a> {

    /// Produces an `Iterator` that yields only events associated with the given mouse button.
    pub fn button(self, button: input::MouseButton) -> MouseButtonReleases<'a> {
        MouseButtonReleases {
            mouse_releases: self,
            button: button,
        }
    }

    /// Produces an `Iterator` that yields only the left mouse button release events.
    pub fn left(self) -> MouseButtonReleases<'a> {
        self.button(input::MouseButton::Left)
    }

    /// Produces an `Iterator` that yields only the middle mouse button release events.
    pub fn middle(self) -> MouseButtonReleases<'a> {
        self.button(input::MouseButton::Middle)
    }

    /// Produces an `Iterator` that yields only the right mouse button release events.
    pub fn right(self) -> MouseButtonReleases<'a> {
        self.button(input::MouseButton::Right)
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
        // Loop through all events in the `ui_events` until we find one associated with our widget
        // that we can return.
        while let Some(ui_event) = self.ui_events.next() {
            match *ui_event {

                // Input source capturing.
                event::Ui::WidgetCapturesInputSource(idx, source) => {
                    self.capturing_mouse = Some(idx);
                    if idx == self.idx {
                        return Some(event::Widget::CapturesInputSource(source));
                    }
                },
                event::Ui::WidgetUncapturesInputSource(idx, source) => {
                    if Some(idx) == self.capturing_mouse {
                        self.capturing_mouse = None;
                    }
                    if idx == self.idx {
                        return Some(event::Widget::UncapturesInputSource(source));
                    }
                },

                event::Ui::WindowResized(dim) =>
                    return Some(event::Widget::WindowResized(dim)),

                event::Ui::Text(idx, ref text) if idx == Some(self.idx) =>
                    return Some(text.clone().into()),

                event::Ui::Motion(idx, ref motion) if idx == Some(self.idx) =>
                    return Some(motion.clone().into()),

                event::Ui::Touch(idx, ref touch) if idx == Some(self.idx) =>
                    return Some(touch.clone().relative_to(self.rect.xy()).into()),

                event::Ui::Press(idx, ref press) if idx == Some(self.idx) =>
                    return Some(press.clone().relative_to(self.rect.xy()).into()),
                
                event::Ui::Release(idx, ref release) if idx == Some(self.idx) =>
                    return Some(release.clone().relative_to(self.rect.xy()).into()),

                event::Ui::Click(idx, ref click) if idx == Some(self.idx) =>
                    return Some(click.clone().relative_to(self.rect.xy()).into()),

                event::Ui::DoubleClick(idx, ref double_click) if idx == Some(self.idx) =>
                    return Some(double_click.clone().relative_to(self.rect.xy()).into()),

                event::Ui::Tap(idx, ref tap) if idx == Some(self.idx) =>
                    return Some(tap.clone().relative_to(self.rect.xy()).into()),

                event::Ui::Drag(idx, ref drag) if idx == Some(self.idx) =>
                    return Some(drag.clone().relative_to(self.rect.xy()).into()),

                event::Ui::Scroll(idx, ref scroll) if idx == Some(self.idx) =>
                    return Some(scroll.clone().into()),

                _ => (),
                
            }
        }

        None
    }
}


impl<'a> Iterator for Presses<'a> {
    type Item = event::Press;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(event) = self.events.next() {
            if let event::Widget::Press(press) = event {
                return Some(press);
            }
        }
        None
    }
}

impl<'a> Iterator for MousePresses<'a> {
    type Item = event::MousePress;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(press) = self.presses.next() {
            if let Some(mouse_press) = press.mouse() {
                return Some(mouse_press);
            }
        }
        None
    }
}

impl<'a> Iterator for MouseButtonPresses<'a> {
    type Item = (Point, input::keyboard::ModifierKey);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mouse_press) = self.mouse_presses.next() {
            if self.button == mouse_press.button {
                return Some((mouse_press.xy, mouse_press.modifiers));
            }
        }
        None
    }
}

impl<'a> Iterator for KeyPresses<'a> {
    type Item = event::KeyPress;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(press) = self.presses.next() {
            if let Some(key_press) = press.key() {
                return Some(key_press);
            }
        }
        None
    }
}

impl<'a> Iterator for Releases<'a> {
    type Item = event::Release;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(event) = self.events.next() {
            if let event::Widget::Release(release) = event {
                return Some(release);
            }
        }
        None
    }
}

impl<'a> Iterator for MouseReleases<'a> {
    type Item = event::MouseRelease;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(release) = self.releases.next() {
            if let Some(mouse_release) = release.mouse() {
                return Some(mouse_release);
            }
        }
        None
    }
}

impl<'a> Iterator for MouseButtonReleases<'a> {
    type Item = (Point, input::keyboard::ModifierKey);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mouse_release) = self.mouse_releases.next() {
            if self.button == mouse_release.button {
                return Some((mouse_release.xy, mouse_release.modifiers));
            }
        }
        None
    }
}

impl<'a> Iterator for KeyReleases<'a> {
    type Item = event::KeyRelease;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(release) = self.releases.next() {
            if let Some(key_release) = release.key() {
                return Some(key_release);
            }
        }
        None
    }
}

impl<'a> Iterator for Clicks<'a> {
    type Item = event::Click;
    fn next(&mut self) -> Option<Self::Item> {
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
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(click) = self.clicks.next() {
            if self.button == click.button {
                return Some(click);
            }
        }
        None
    }
}

impl<'a> Iterator for Taps<'a> {
    type Item = event::Tap;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(event) = self.events.next() {
            if let event::Widget::Tap(tap) = event {
                return Some(tap);
            }
        }
        None
    }
}

impl<'a> Iterator for Drags<'a> {
    type Item = event::Drag;
    fn next(&mut self) -> Option<Self::Item> {
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
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(drag) = self.drags.next() {
            if self.button == drag.button {
                return Some(drag);
            }
        }
        None
    }
}

impl<'a> Iterator for Texts<'a> {
    type Item = event::Text;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(event) = self.events.next() {
            if let event::Widget::Text(text) = event {
                return Some(text);
            }
        }
        None
    }
}

impl<'a> Iterator for Scrolls<'a> {
    type Item = event::Scroll;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(event) = self.events.next() {
            if let event::Widget::Scroll(scroll) = event {
                return Some(scroll);
            }
        }
        None
    }
}
