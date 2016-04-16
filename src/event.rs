//! Contains all the structs and enums to describe all of the input events that `Widget`s
//! can handle.
//!
//! The core of this module is the `Event` enum, which encapsulates all of those events.

use input::{keyboard, MouseButton};
use position::Point;
use utils::vec2_sub;
use widget;

#[doc(inline)]
pub use backend::event::{Input, Motion};


/// Enum containing all the events that the `Ui` may provide.
#[derive(Clone, PartialEq, Debug)]
pub enum Event {
    /// Represents a raw `input::Input` event.
    Raw(Input),
    /// Events that have been interpreted from `backend::RawEvent`s by the `Ui`.
    ///
    /// Most events usually 
    Ui(Ui)
}

/// Represents all events interpreted by the `Ui`.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Ui {
    /// Represents a pointing device being pressed and subsequently released while over the same
    /// location.
    Click(Option<widget::Index>, Click),
    /// Two `Click` events with the same `button` and `xy` occurring within a duration that is less
    /// that the `theme.double_click_threshold`.
    DoubleClick(Option<widget::Index>, DoubleClick),
    /// Represents a pointing device button being pressed and a subsequent movement of the mouse.
    Drag(Option<widget::Index>, Drag),
    /// This is a generic scroll event. This is different from the `input::Motion::MouseScroll`
    /// event in several aspects.
    ///
    /// For one, it does not necessarily have to get created by a mouse wheel, it could be
    /// generated from a keypress, or as a response to handling some other event.
    ///
    /// Secondly, it contains a field holding the `input::keyboard::ModifierKey` that was held
    /// while the scroll occured.
    Scroll(Option<widget::Index>, Scroll),
    /// Indicates that the given widget has captured the mouse.
    WidgetCapturesMouse(widget::Index),
    /// Indicates that the given widget has released the mouse from capturing.
    WidgetUncapturesMouse(widget::Index),
    /// Indicates that the given widget has captured the keyboard.
    WidgetCapturesKeyboard(widget::Index),
    /// Indicates that the given widget has released the keyboard from capturing.
    WidgetUncapturesKeyboard(widget::Index),
}

/// Events that apply to a specific widget.
///
/// Rather than delivering entire `event::Event`s to the widget (with a lot of redundant
/// information), this `event::Widget` is used as a refined, widget-specific event.
///
/// All `Widget` event co-ordinates will be relative to the centre of the `Widget` to which they
/// are delivered.
#[derive(Clone, PartialEq, Debug)]
pub enum Widget {
    /// Raw `Input` events that occurred while the `Widget` was capturing the associated `Input`
    /// source device.
    ///
    /// For example, if the widget was capturing the `Keyboard` while an `Input::Text` event
    /// occurs, the widget will receive that event.
    Raw(Input),
    /// Represents a pointing device being pressed and subsequently released while over the same
    /// location.
    Click(Click),
    /// Two `Click` events with the same `button` and `xy` occurring within a duration that is less
    /// that the `theme.double_click_threshold`.
    DoubleClick(DoubleClick),
    /// Represents a pointing device button being pressed and a subsequent movement of the mouse.
    Drag(Drag),
    /// Represents the amount of scroll that has been applied to this widget.
    Scroll(Scroll),
    /// The widget has captured the mouse.
    CapturesMouse,
    /// The widget has released the mouse from capturing.
    UncapturesMouse,
    /// The widget has captured the keyboard.
    CapturesKeyboard,
    /// Indicates that the given widget has released the keyboard from capturing.
    UncapturesKeyboard,
}

/// Contains all the relevant information for a mouse drag.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Drag {
    /// Which mouse button was being held during the drag
    pub button: MouseButton,
    /// The point from which the current series of drag events began.
    ///
    /// This will be the position of the pointing device whenever the dragging press began.
    pub origin: Point,
    /// The point from which this drag event began.
    pub from: Point,
    /// The point at which this drag event ended.
    pub to: Point,
    /// The magnitude of the vector between `from` and `to`.
    pub delta_xy: Point,
    /// The magnitude of the vector between `origin` and `to`.
    pub total_delta_xy: Point,
    /// Which modifier keys are being held during the mouse drag.
    pub modifiers: keyboard::ModifierKey,
}

/// Contains all the relevant information for a mouse click.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Click {
    /// Which mouse button was clicked
    pub button: MouseButton,
    /// The position at which the mouse was released.
    pub xy: Point,
    /// Which modifier keys, if any, that were being held down when the user clicked
    pub modifiers: keyboard::ModifierKey,
}

/// Contains all the relevant information for a double click.
///
/// When handling this event, be sure to check that you are handling the intended `button` too.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct DoubleClick {
    /// Which mouse button was clicked
    pub button: MouseButton,
    /// The position at which the mouse was released.
    pub xy: Point,
    /// Which modifier keys, if any, that were being held down when the user clicked
    pub modifiers: keyboard::ModifierKey,
}

/// Holds all the relevant information about a scroll event
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Scroll {
    /// The amount of scroll along the x axis.
    pub x: f64,
    /// The amount of scroll along the y axis.
    pub y: f64,
    /// Which modifier keys, if any, that were being held down while the scroll occured
    pub modifiers: keyboard::ModifierKey,
}

impl Click {
    /// Returns a copy of the Click relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> Click {
        Click {
            xy: vec2_sub(self.xy, xy),
            ..*self
        }
    }
}

impl DoubleClick {
    /// Returns a copy of the DoubleClick relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> DoubleClick {
        DoubleClick {
            xy: vec2_sub(self.xy, xy),
            ..*self
        }
    }
}

impl Drag {
    /// Returns a copy of the Drag relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> Drag {
        Drag{
            origin: vec2_sub(self.origin, xy),
            from: vec2_sub(self.from, xy),
            to: vec2_sub(self.to, xy),
            ..*self
        }
    }
}


impl From<Ui> for Event {
    fn from(ui: Ui) -> Self {
        Event::Ui(ui)
    }
}

impl From<Input> for Event {
    fn from(input: Input) -> Self {
        Event::Raw(input)
    }
}

impl From<Input> for Widget {
    fn from(input: Input) -> Self {
        Widget::Raw(input)
    }
}

impl From<Click> for Widget {
    fn from(click: Click) -> Self {
        Widget::Click(click)
    }
}

impl From<DoubleClick> for Widget {
    fn from(double_click: DoubleClick) -> Self {
        Widget::DoubleClick(double_click)
    }
}

impl From<Scroll> for Widget {
    fn from(scroll: Scroll) -> Self {
        Widget::Scroll(scroll)
    }
}

impl From<Drag> for Widget {
    fn from(drag: Drag) -> Self {
        Widget::Drag(drag)
    }
}
