
use Scalar;
use position::Point;

/// The width of the scrollbar when visible.
pub const SCROLLBAR_WIDTH: f64 = 10.0;


/// State related to scrolling.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State {
    /// The current interaction with the Scrollbar.
    interaction: Interaction,
    /// The current scroll position as an offset from the top left.
    offset: Point,
    /// The maximum possible offset for the handle.
    max_offset: Point,
}


/// The current interaction with the 
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Interaction {
    /// No interaction with the Scrollbar.
    Normal,
    /// Part of the scrollbar is highlighted.
    Highlighted(Elem),
    /// Part of the scrollbar is clicked.
    Clicked(Elem),
}


/// The elements that make up a ScrollBar.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Elem {
    /// The draggable bar and the mouse's position.
    Handle(Point),
    /// The track along which the bar can be dragged.
    Track(Point),
}




