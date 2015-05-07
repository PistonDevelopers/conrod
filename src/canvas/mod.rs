
use elmesque::Element;
use position::{Padding, Point};

pub mod split;

/// Unique identifier for Canvasses.
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
#[derive(Clone, Debug)]
pub enum Kind {
    /// Represents a placeholder Canvas.
    NoCanvas,
    /// A split of another Canvas.
    Split(split::State),
    // /// A floating / draggable Canvas.
    // Floating(floating::State),
}

impl Kind {
    /// Do the Kind variants match?
    pub fn matches(&self, other: &Kind) -> bool {
        match (self, other) {
            (&Kind::NoCanvas, &Kind::NoCanvas) => true,
            (&Kind::Split(_), &Kind::Split(_)) => true,
            _ => false,
        }
    }
}
