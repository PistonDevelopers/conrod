//! This module contains all the logic for handling input events and providing them to widgets.
//!
//! All user input is provided to the `Ui` in the form of `input::Input` events, which are received
//! via the `Ui::handle_event` method. These raw input events tend to be fairly low level. The `Ui`
//! stores each of these `Input` events in it's `GlobalInput`, which keeps track of the state of
//! input for the entire `Ui`. `GlobalInput` will also aggregate the low level events into higher
//! level ones. For instance, two events indicating that a mouse button was pressed then released
//! would cause a new `UiEvent::MouseClick` to be generated. This saves individual widgets from
//! having to interpret these themselves, thus freeing them from also having to store input state.
//!
//! Whenever there's an update, all of the events that have occurred since the last update will be
//! available for widgets to process. `WidgetInput` is used to provide input events to a specific
//! widget. It filters events that do not apply to the widget. All events provided by `WidgetIput`
//! will have all coordinates in the widget's own local coordinate system, where `(0, 0)` is the
//! middle of the widget's bounding `Rect`. `GlobalInput`, on the other hand, will never filter out
//! any events, and will always provide them with coordinates relative to the window.

pub mod state;
pub mod widget;
pub mod global;

use Scalar;
pub use self::state::State;
pub use self::global::Global;
pub use self::touch::Touch;
pub use self::widget::Widget;

#[doc(inline)]
pub use piston_input::keyboard::ModifierKey;
#[doc(inline)]
pub use piston_input::{
    Button,
    ControllerButton,
    ControllerAxisArgs,
    keyboard,
    Key,
    MouseButton,
    RenderArgs,
};


/// Sources from which user input may be received.
///
/// We use these to track which sources of input are being captured by which widget.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Source {
    /// Mouse input (i.e. movement, buttons).
    Mouse,
    /// Keyboard input.
    Keyboard,
    /// Input from a finger on a touch screen/surface.
    Touch(self::touch::Id),
}

/// Different kinds of motion input.
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Motion {
    /// Absolute cursor position within the window.
    ///
    /// For more details on co-ordinate orientation etc, see the `Input` docs.
    MouseCursor { x: Scalar, y: Scalar },
    /// Relative mouse movement.
    MouseRelative { x: Scalar, y: Scalar },
    /// x and y in scroll ticks.
    Scroll { x: Scalar, y: Scalar },
    /// controller axis move event.
    ControllerAxis(ControllerAxisArgs),
}


/// Touch-related items.
pub mod touch {
    use Point;

    /// A type for uniquely identifying the source of a touch interaction.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Id(u64);

    /// The stage of the touch interaction.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
    pub enum Phase {
        /// The start of a touch interaction.
        Start,
        /// A touch moving across a surface.
        Move,
        /// The touch interaction was cancelled.
        Cancel,
        /// The end of a touch interaction.
        End,
    }

    /// Represents a touch interaction.
    ///
    /// Each time a user touches the surface with a new finger, a new series of `Touch` events
    /// `Start`, each with a unique identifier.
    ///
    /// For every `Id` there should be at least 2 events with `Start` and `End` (or `Cancel`led)
    /// `Phase`s.
    ///
    /// A `Start` input received with the same `Id` as a previously received `End` does *not*
    /// indicate that the same finger was used. `Id`s are only used to distinguish between
    /// overlapping touch interactions.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Touch {
        /// The stage of the touch interaction.
        pub phase: Phase,
        /// A unique identifier associated with the source of the touch interaction.
        pub id: Id,
        /// The location of the touch on the surface/screen. See `Input` docs for information on
        /// the co-ordinate system.
        pub xy: Point,
    }

    impl Id {

        /// Construct a new identifier.
        pub fn new(id: u64) -> Self {
            Id(id)
        }

    }

    impl Touch {

        /// Returns a copy of the `Touch` relative to the given `xy`.
        pub fn relative_to(&self, xy: Point) -> Self {
            Touch {
                xy: [self.xy[0] - xy[0], self.xy[1] - xy[1]],
                ..*self
            }
        }

    }

}
