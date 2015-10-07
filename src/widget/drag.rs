//! 
//! Types and functionality related to the dragging behaviour of Widgets.
//!

use mouse::Mouse;
use position::{Point, Rect};



/// The current drag interaction for the Widget.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    /// Idle drag state.
    Normal,
    /// The drag area is highlighted.
    Highlighted,
    /// The drag area is clicked at the given point.
    Clicked(Point),
}


/// Drag the widget from its position `xy` and return the new position.
pub fn drag_widget(xy: Point, rel_rect: Rect, state: State, mouse: Mouse) -> (Point, State) {
    use self::State::{Normal, Highlighted, Clicked};
    use mouse::ButtonPosition::{Up, Down};

    // Find the absolute position of the draggable area.
    let abs_rect = rel_rect.shift(xy);

    // Check whether or not the cursor is over the drag area.
    let is_over = abs_rect.is_over(mouse.xy);

    // Determine the new drag state.
    let new_state = match (is_over, state, mouse.left.position) {
        (true,  Normal,     Down) => Normal,
        (true,  _,          Up)   => Highlighted,
        (true,  _,          Down) |
        (false, Clicked(_), Down) => Clicked(mouse.xy),
        _                         => Normal,
    };

    // Drag the Canvas if the TitleBar remains clicked.
    let new_xy = match (state, new_state) {
        //(Clicked(ax, ay), Clicked(bx, by)) => ::vecmath::vec2_add(xy, [bx - ax, by - ay]),
        (Clicked(a), Clicked(b)) => ::vecmath::vec2_add(xy, ::vecmath::vec2_sub(b, a)),
        _                        => xy,
    };

    (new_xy, new_state)
}



