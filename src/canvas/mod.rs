
use elmesque::Element;
use position::{Padding, Point};

pub mod floating;
pub mod split;

/// Unique canvas identifier. Each canvas must use a unique `CanvasId` so that it's state can be
/// cached within the `Ui` type. The reason we use a usize is because canvasses are cached within
/// a `Vec`, which is limited to a size of `usize` elements.
pub type CanvasId = usize;

/// The kind of Canvas.
#[derive(Clone, Debug)]
pub struct Canvas {
    /// The Kind of Canvas (i.e. Split, Floating, etc).
    pub kind: Kind,
    /// The position of the Canvas within the window.
    pub xy: Point,
    /// The Element used for drawing the Canvas.
    pub element: Element,
    /// Padding for the Canvas describes the distance between each edge and its Widget's.
    pub padding: Padding,
    /// Has the Canvas been set since the last time the Ui was drawn?
    pub has_updated: bool,
}

impl Canvas {

    /// An empty Canvas 
    pub fn empty() -> Canvas {
        Canvas {
            xy: [0.0, 0.0],
            element: ::elmesque::element::empty(),
            padding: Padding::none(),
            kind: Kind::NoCanvas,
            has_updated: false,
        }
    }

}

/// The different kinds of Canvases available to the user.
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    /// Represents a placeholder Canvas.
    NoCanvas,
    /// A split of another Canvas.
    Split(split::State),
    /// A floating / draggable Canvas.
    Floating(floating::State),
}

impl Kind {
    /// Do the Kind variants match?
    pub fn matches(&self, other: &Kind) -> bool {
        match (self, other) {
            (&Kind::NoCanvas, &Kind::NoCanvas) => true,
            (&Kind::Split(_), &Kind::Split(_)) => true,
            (&Kind::Floating(_), &Kind::Floating(_)) => true,
            _ => false,
        }
    }
}

