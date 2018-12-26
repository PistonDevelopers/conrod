//! Contains all types used to describe the input events that `Widget`s may handle.
//!
//! The two primary types of this module are:
//!
//! - `Input`: conrod's input type passed by the user to `Ui::handle_event` in order to drive the
//! `Ui`.
//! - `Event`: enumerates all possible events interpreted by conrod that may be propagated to
//! widgets.
//!
//! The Event System
//! ----------------
//!
//! Conrod's event system looks like this:
//!
//! *Input -> Ui -> Event -> Widget*
//!
//! The **Ui** receives **Input**s such as `Press` and `Release` via the `Ui::handle_event` method.
//! It interprets these **Input**s to create higher-level **Event**s such as `DoubleClick`,
//! `WidgetCapturesKeyboard`, etc. These **Event**s are stored and then fed to each **Widget** when
//! `Ui::set_widgets` is called. At the end of `Ui::set_widgets` the stored **Event**s are flushed
//! ready for the next incoming **Input**s.
//!
//! Conrod uses the `pistoncore-input` crate's `Input` type. There are a few reasons for this:
//!
//! 1. This `Input` type already provides a number of useful variants of events that we wish to
//!    provide and handle within conrod, and we do not yet see any great need to re-write it and
//!    duplicate code.
//! 2. The `Input` type is already compatible with all `pistoncore-window` backends including
//!    `glfw_window`, `sdl2_window` and `glutin_window`. That said, co-ordinates and scroll
//!    directions may need to be translated to conrod's orientation.
//! 3. The `pistoncore-input` crate also provides a `GenericEvent` trait which allows us to easily
//!    provide a blanket implementation of `ToRawEvent` for all event types that already implement
//!    this trait.
//!
//! Because we use the `pistoncore-input` `Event` type, we also re-export its associated data
//! types (`Button`, `ControllerAxisArgs`, `Key`, etc).

use input;
use position::{Dimensions, Point};
use utils::vec2_sub;
use widget;


/// The event type that is used by conrod to track inputs from the world. Events yielded by polling
/// window backends should be converted to this type. This can be thought of as the event type
/// which is supplied by the window backend to drive the state of the `Ui` forward.
///
/// This type is solely used within the `Ui::handle_event` method.  The `Input` events are
/// interpreted to create higher level `Event`s (such as DoubleClick, WidgetCapturesKeyboard, etc)
/// which are stored for later processing by `Widget`s, which will occur during the call to
/// `Ui::set_widgets`.
///
/// **Note:** `Input` events that contain co-ordinates must be oriented with (0, 0) at the middle
/// of the window with the *y* axis pointing upwards (Cartesian co-ordinates). All co-ordinates and
/// dimensions must be given as `Scalar` (DPI agnostic) values. Many windows provide coordinates
/// with the origin in the top left with *y* pointing down, so you might need to translate these
/// co-ordinates when converting to this event. Also be sure to invert the *y* axis of MouseScroll
/// events.
#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    /// A button on some input device was pressed.
    Press(input::Button),
    /// A button on some input device was released.
    Release(input::Button),
    /// The window was received to the given dimensions.
    Resize(f64, f64),
    /// Some motion input was received (e.g. moving mouse or joystick axis).
    Motion(input::Motion),
    /// Input from a touch surface/screen.
    Touch(input::Touch),
    /// Text input was received, usually via the keyboard.
    Text(String),
    /// The window was focused or lost focus.
    Focus(bool),
    /// The backed requested to redraw.
    Redraw,
}


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
#[derive(Clone, PartialEq, Debug)]
pub enum Ui {
    /// Entered text, along with the widget that was capturing the keyboard at the time.
    Text(Option<widget::Id>, Text),
    /// Some button was pressed, along with the widget that was capturing the device whose button
    /// was pressed.
    Press(Option<widget::Id>, Press),
    /// Some button was released, along with the widget that was capturing the device whose button
    /// was released.
    Release(Option<widget::Id>, Release),
    /// Represents all forms of motion input, alongside with the widget that was capturing the
    /// mouse at the time.
    Motion(Option<widget::Id>, Motion),
    /// Interaction with a touch screen/surface.
    Touch(Option<widget::Id>, input::Touch),
    /// The window's dimensions were resized.
    WindowResized(Dimensions),
    /// Represents a pointing device being pressed and subsequently released while over the same
    /// location.
    Click(Option<widget::Id>, Click),
    /// Two `Click` events with the same `button` and `xy` occurring within a duration that is less
    /// that the `theme.double_click_threshold`.
    DoubleClick(Option<widget::Id>, DoubleClick),
    /// A user tapped a touch screen/surface.
    Tap(Option<widget::Id>, Tap),
    /// Represents a pointing device button being pressed and a subsequent movement of the mouse.
    Drag(Option<widget::Id>, Drag),
    /// A generic scroll event.
    ///
    /// `Scroll` does not necessarily have to get created by a mouse wheel, it could be generated
    /// from a keypress, or as a response to handling some other event.
    ///
    /// Received `Scroll` events are first applied to all scrollable widgets under the mouse from
    /// top to bottom. The remainder will then be applied to either 1. whatever widget captures the
    /// device from which the scroll was emitted or 2. whatever widget was specified.
    Scroll(Option<widget::Id>, Scroll),
    /// Indicates that the given widget has captured the given user input source.
    WidgetCapturesInputSource(widget::Id, input::Source),
    /// Indicates that the given widget has released the given user input source.
    WidgetUncapturesInputSource(widget::Id, input::Source),
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
    /// Entered text.
    Text(Text),
    /// Represents all forms of motion input.
    Motion(Motion),
    /// Interaction with a touch screen.
    Touch(input::Touch),
    /// Some button was pressed.
    Press(Press),
    /// Some button was released.
    Release(Release),
    /// Represents a pointing device being pressed and subsequently released while over the same
    /// location.
    Click(Click),
    /// Two `Click` events with the same `button` and `xy` occurring within a duration that is less
    /// that the `theme.double_click_threshold`.
    DoubleClick(DoubleClick),
    /// A user tapped the widget on a touch screen/surface.
    Tap(Tap),
    /// Represents a pointing device button being pressed and a subsequent movement of the mouse.
    Drag(Drag),
    /// Represents the amount of scroll that has been applied to this widget.
    Scroll(Scroll),
    /// The window's dimensions were resized.
    WindowResized(Dimensions),
    /// The widget has captured the given input source.
    CapturesInputSource(input::Source),
    /// The widget has released the input source from capturing.
    UncapturesInputSource(input::Source),
}

/// Contains all relevant information for a Text event.
#[derive(Clone, PartialEq, Debug)]
pub struct Text {
    /// All text that was entered as a part of the event.
    pub string: String,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}

/// Contains all relevant information for a Motion event.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Motion {
    /// The type of `Motion` that occurred.
    pub motion: input::Motion,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}

/// The different kinds of `Button`s that may be `Press`ed or `Release`d.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Button {
    /// A keyboard button.
    Keyboard(input::Key),
    /// A mouse button along with the location at which it was `Press`ed/`Release`d.
    Mouse(input::MouseButton, Point),
    /// A controller button.
    Controller(input::ControllerButton),
}

/// Contains all relevant information for a Press event.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Press {
    /// The `Button` that was pressed.
    pub button: Button,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}

/// Contains all relevant information for the event where a mouse button was pressed.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MousePress {
    /// The mouse button that was pressed.
    pub button: input::MouseButton,
    /// The location at which the mouse was pressed.
    pub xy: Point,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}

/// Contains all relevant information for the event where a keyboard button was pressed.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct KeyPress {
    /// The key that was pressed.
    pub key: input::Key,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}

/// Contains all relevant information for a Release event.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Release {
    /// The `Button` that was released.
    pub button: Button,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}

/// Contains all relevant information for the event where a mouse button was released.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MouseRelease {
    /// The mouse button that was released.
    pub button: input::MouseButton,
    /// The location at which the mouse was released.
    pub xy: Point,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}

/// Contains all relevant information for the event where a keyboard button was release.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct KeyRelease {
    /// The key that was release.
    pub key: input::Key,
    /// The modifier keys that were down at the time.
    pub modifiers: input::keyboard::ModifierKey,
}

/// Contains all the relevant information for a mouse drag.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Drag {
    /// Which mouse button was being held during the drag
    pub button: input::MouseButton,
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
    pub modifiers: input::keyboard::ModifierKey,
}

/// Contains all the relevant information for a mouse click.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Click {
    /// Which mouse button was clicked
    pub button: input::MouseButton,
    /// The position at which the mouse was released.
    pub xy: Point,
    /// Which modifier keys, if any, that were being held down when the user clicked
    pub modifiers: input::keyboard::ModifierKey,
}

/// Contains all the relevant information for a double click.
///
/// When handling this event, be sure to check that you are handling the intended `button` too.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct DoubleClick {
    /// Which mouse button was clicked
    pub button: input::MouseButton,
    /// The position at which the mouse was released.
    pub xy: Point,
    /// Which modifier keys, if any, that were being held down when the user clicked
    pub modifiers: input::keyboard::ModifierKey,
}

/// All relevant information for a touch-screen tap event.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Tap {
    /// The unique identifier of the source of the touch.
    pub id: input::touch::Id,
    /// The position at which the finger left the screen.
    pub xy: Point,
}

/// Holds all the relevant information about a scroll event
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Scroll {
    /// The amount of scroll along the x axis.
    pub x: f64,
    /// The amount of scroll along the y axis.
    pub y: f64,
    /// Which modifier keys, if any, that were being held down while the scroll occured
    pub modifiers: input::keyboard::ModifierKey,
}

impl Motion {
    /// Returns a copy of the `Motion` relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> Motion {
        let motion = match self.motion {
            input::Motion::MouseCursor { x, y } =>
                input::Motion::MouseCursor { x: x - xy[0], y: y - xy[1] },
            motion => motion,
        };
        Motion {
            motion: motion,
            ..*self
        }
    }
}

impl Button {
    /// Returns a copy of the Button relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> Button {
        match *self {
            Button::Mouse(m_button, self_xy) => Button::Mouse(m_button, vec2_sub(self_xy, xy)),
            button => button,
        }
    }
}

impl Press {

    /// Returns a copy of the Press relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> Press {
        Press {
            button: self.button.relative_to(xy),
            ..*self
        }
    }

    /// If the `Press` event represents the pressing of a mouse button, return `Some`.
    pub fn mouse(self) -> Option<MousePress> {
        match self.button {
            Button::Mouse(button, xy) =>
                Some(MousePress {
                    button: button,
                    xy: xy,
                    modifiers: self.modifiers,
                }),
            _ => None,
        }
    }

    /// If the `Press` event represents the pressing of keyboard button, return `Some`.
    pub fn key(self) -> Option<KeyPress> {
        match self.button {
            Button::Keyboard(key) =>
                Some(KeyPress {
                    key: key,
                    modifiers: self.modifiers,
                }),
            _ => None,
        }
    }

}

impl Release {

    /// Returns a copy of the Release relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> Release {
        Release {
            button: self.button.relative_to(xy),
            ..*self
        }
    }

    /// If the `Release` event represents the releasing of a mouse button, return `Some`.
    pub fn mouse(self) -> Option<MouseRelease> {
        match self.button {
            Button::Mouse(button, xy) =>
                Some(MouseRelease {
                    button: button,
                    xy: xy,
                    modifiers: self.modifiers,
                }),
            _ => None,
        }
    }

    /// If the `Release` event represents the release of keyboard button, return `Some`.
    pub fn key(self) -> Option<KeyRelease> {
        match self.button {
            Button::Keyboard(key) =>
                Some(KeyRelease {
                    key: key,
                    modifiers: self.modifiers,
                }),
            _ => None,
        }
    }

}

impl Tap {
    /// Returns a copy of the `Tap` relative to the given `xy`
    pub fn relative_to(&self, xy: Point) -> Self {
        Tap {
            xy: vec2_sub(self.xy, xy),
            ..*self
        }
    }
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


impl From<input::Motion> for Input {
    fn from(motion: input::Motion) -> Self {
        Input::Motion(motion)
    }
}

impl From<input::Touch> for Input {
    fn from(touch: input::Touch) -> Self {
        Input::Touch(touch)
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

impl From<Text> for Widget {
    fn from(text: Text) -> Self {
        Widget::Text(text)
    }
}

impl From<Motion> for Widget {
    fn from(motion: Motion) -> Self {
        Widget::Motion(motion)
    }
}

impl From<input::Touch> for Widget {
    fn from(touch: input::Touch) -> Self {
        Widget::Touch(touch)
    }
}

impl From<Press> for Widget {
    fn from(press: Press) -> Self {
        Widget::Press(press)
    }
}

impl From<Release> for Widget {
    fn from(release: Release) -> Self {
        Widget::Release(release)
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

impl From<Tap> for Widget {
    fn from(tap: Tap) -> Self {
        Widget::Tap(tap)
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
