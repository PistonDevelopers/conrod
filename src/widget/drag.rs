
use mouse::Mouse;
use position::{Dimensions, Point};


/// Represents the draggable area of a widget by its coordinates and dimensions.
#[derive(Copy, Clone, Debug)]
pub struct Area {
    /// The absolute center position of the area.
    pub xy: Point,
    /// The dimensions of the area.
    pub dim: Dimensions,
}

/// The current drag interaction for the Widget.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Normal,
    Highlighted,
    Clicked(Point),
}


/// Drag the widget from its position `xy` and return the new position.
pub fn drag_widget(xy: Point,
                   area: Area,
                   state: State,
                   mouse: Mouse) -> (Point, State)
{
    use self::State::{Normal, Highlighted, Clicked};
    use mouse::ButtonState::{Up, Down};
    use utils::is_over_rect;

    // Find the absolute position of the draggable area.
    let abs_area_xy = ::vecmath::vec2_add(xy, area.xy);

    // Check whether or not the cursor is over the drag area.
    let is_over = is_over_rect(abs_area_xy, mouse.xy, area.dim);

    // Determine the new drag state.
    let new_state = match (is_over, state, mouse.left) {
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



