
use button;
use drop_down_list;
use number_dialer;
use toggle;
use slider;

/// Algebraic widget type for storing in ui_context
/// and for ease of state-matching.
pub enum Widget {
    Button(button::State),
    DropDownList(drop_down_list::State),
    NumberDialer(number_dialer::State),
    Toggle(toggle::State),
    Slider(slider::State),
}

/*
/// Location at which the widget will be added
/// to it's parent after the previous child. The
/// `uint` in the four directional variants specifies the
/// padding between this widget and the previous.
/// The `Relative` variant specifies a location relative
/// to the location of the previous widget.
/// The `Specific` variant specifies an exact location
/// for the Widget if this is preferred.
#[deriving(Show, Clone)]
pub enum RelativePosition {
    Up(uint),
    Down(uint),
    Left(uint),
    Right(uint),
    Relative(Point<int>),
    Specific(Point<int>),
}

impl RelativePosition {
    /// Returns an absolute position as a `Point` determined by using
    /// the RelativePosition in association with a previous `Point`.
    fn as_point(&self, prev: Point<int>) -> Point<int> {
        match self {
            &Up(pad) => prev + Point::new(0, -(pad as int), 0),
            &Down(pad) => prev + Point::new(0, pad as int, 0),
            &Left(pad) => prev + Point::new(-(pad as int), 0, 0),
            &Right(pad) => prev + Point::new(pad as int, 0, 0),
            &Relative(p) => prev + p,
            &Specific(p) => p,
        }
    }
}
*/

