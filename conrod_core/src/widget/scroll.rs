//! Scroll related types and logic.

use Ui;
use position::{Align, Point, Padding, Range, Rect, Scalar};
use std::marker::PhantomData;


/// Arguments given via a scrollable `Widget`'s builder methods for the scrolling along a single
/// axis.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scroll {
    maybe_initial_alignment: Option<Align>,
}

/// Scroll state calculated for a single axis.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State<A> {
    /// The distance that has been scrolled from the origin.
    ///
    /// A positive offset pushes the scrollable range that is under the kid_area upwards.
    ///
    /// A negative offset pushes the scrollable range that is under the kid_area downwards.
    pub offset: Scalar,
    /// The start and end bounds for the offset along the axis.
    pub offset_bounds: Range,
    /// The total range which may be "offset" from the "root" range (aka the `kid_area`).
    ///
    /// The `scrollable_range` is determined as the bounding range around both the `kid_area` and
    /// all **un-scrolled** **visible** children widgets.
    pub scrollable_range_len: Scalar,
    /// Whether or not the this axis is currently scrolling.
    pub is_scrolling: bool,
    /// The axis type used to instantiate this state.
    axis: PhantomData<A>,
}

/// Methods for distinguishing behaviour between both scroll axes at compile-time.
pub trait Axis {
    /// The range of the given `Rect` that is parallel with this `Axis`.
    fn parallel_range(Rect) -> Range;
    /// The range of the given `Rect` that is perpendicular with this `Axis`.
    fn perpendicular_range(Rect) -> Range;
    /// Given some rectangular `Padding`, return the `Range` that corresponds with this `Axis`.
    fn padding_range(Padding) -> Range;
    /// The coordinate of the given mouse position that corresponds with this `Axis`.
    fn mouse_scalar(mouse_xy: Point) -> Scalar;
    /// A `Scalar` multiplier representing the direction in which positive offset shifts the
    /// `scrollable_range` (either `-1.0` or `1.0).
    fn offset_direction() -> Scalar;
}

/// Behaviour for scrolling across the `X` axis.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum X {}

/// Behaviour for scrolling across the `Y` axis.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Y {}

/// State for scrolling along the **X** axis.
pub type StateX = State<X>;

/// State for scrolling along the **Y** axis.
pub type StateY = State<Y>;


impl Scroll {
    /// The default `Scroll` args.
    pub fn new() -> Self {
        Scroll {
            maybe_initial_alignment: None,
        }
    }
}


impl<A> State<A>
    where A: Axis
{

    /// Calculate the new scroll state for the single axis of a `Widget`.
    ///
    /// ```txt
    ///
    ///           >     +---+
    ///           |     |   |
    ///           |   =========================
    ///           |   | | a | scroll root     |
    ///           |   | +---+ aka `kid_area`  |
    ///           |   |            +--------+ |
    ///           |   |            |        | |
    ///           |   =========================
    ///           |                |   b    |
    ///           |                +--------+
    /// scrollable|    +--------+
    ///    range y|    |        |
    ///           |    |        | +------+
    ///           |    |    c   | |      |
    ///           |    +--------+ |  d   |
    ///           |               |      |
    ///           >               +------+
    ///
    ///                ^--------------------^
    ///                     scrollable
    ///                      range x
    ///
    /// ```
    ///
    /// - `kid_area` is the cropped area of the container widget in which kid widgets may be
    /// viewed.
    /// - `a`, `b`, `c` and `d` are widgets that are kids of the "scroll root" widget in their
    /// original, un-scrolled positions.
    /// - `scrollable_range` is the total range occuppied by all children widgets in their
    /// original, un-scrolled positions.
    ///
    /// Everything above and below the set of ==== bars of the parent widget is hidden, i.e:
    ///
    /// ```txt
    ///
    /// =========================
    /// | | a | scroll root     |
    /// | +---+ aka `kid_area`  |
    /// |            +--------+ |
    /// |            |   b    | |
    /// =========================
    ///
    /// ```
    ///
    /// The `scrollable_range` on each axis only becomes scrollable if its length exceeds the
    /// length of the `kid_area` on the same axis. Thus, in the above example, only the *y*
    /// scrollable_range is scrollable.
    ///
    /// The `offset_bounds` are calculated as the amount which the original, un-scrolled,
    /// `scrollable_range` may be offset from its origin.
    ///
    /// ```txt
    ///
    ///   offset +              >
    ///   bounds v              |
    ///   .start >              |   =========================
    ///                         |   |                       |
    ///                         |   |        kid_area       |
    ///                         |   |                       |
    ///                         |   |                       |
    ///          >   scrollable |   =========================
    ///          ^      range y |
    ///          ^              |    
    ///          ^              |    
    ///   offset ^              |    
    ///   bounds ^              |    
    ///     .end ^              |    
    ///          ^              |    
    ///          ^              |    
    ///          +              >    
    ///
    /// ```
    pub fn update(ui: &Ui,
                  idx: super::Id,
                  kid_area: &super::KidArea,
                  maybe_prev_scroll_state: Option<Self>,
                  additional_offset: Scalar) -> Self
    {

        // Retrieve the *current* scroll offset.
        let current_offset = maybe_prev_scroll_state.as_ref()
            .map(|state| state.offset)
            .unwrap_or(0.0);

        // Padding for the range.
        let padding = A::padding_range(kid_area.pad);

        // Get the range for the Axis that concerns this particular scroll `State`.
        let kid_area_range = A::parallel_range(kid_area.rect).pad_ends(padding.start, padding.end);

        // The `kid_area_range` but centred at zero.
        let kid_area_range_origin = Range::from_pos_and_len(0.0, kid_area_range.magnitude());

        // The un-scrolled, scrollable_range relative to the kid_area_range's position.
        let scrollable_range = {
            ui.kids_bounding_box(idx)
                .map(|kids| {
                    A::parallel_range(kids)
                        .shift(-current_offset)
                        .shift(-kid_area_range.middle())
                })
                .unwrap_or_else(|| Range::new(0.0, 0.0))
        };

        // Determine the min and max offset bounds. These bounds are the limits to which the
        // scrollable_range may be shifted in either direction across the range.
        let min_offset = Range::new(scrollable_range.start, kid_area_range_origin.start).magnitude();
        let max_offset = Range::new(scrollable_range.end, kid_area_range_origin.end).magnitude();
        let offset_bounds = Range::new(min_offset, max_offset);

        // The range is only scrollable if it is longer than the padded kid_area_range.
        let is_scrollable = scrollable_range.len() > kid_area_range.len();

        // If the range is scrollable, calculate the new offset by adding the `additional_offset`.
        //
        // The `additional_offset` is given via a `Scroll` event.
        let new_offset_unbounded =
            if is_scrollable { current_offset + additional_offset }
            else             { current_offset };

        // Clamp the new offset to ensure it does not exceed the `offset_bounds`.
        let new_offset = {
            // If there was some previous scroll state, we must also ensure that our new offset does
            // exceed its `offset_bounds` either. We do this in order to avoid causing jitter when
            // scrolling towards either end of the Range.
            let new_offset = maybe_prev_scroll_state.as_ref()
                .map(|prev| prev.offset_bounds.clamp_value(new_offset_unbounded))
                .unwrap_or(new_offset_unbounded);
            offset_bounds.clamp_value(new_offset)
        };

        State {
            offset: new_offset,
            offset_bounds: offset_bounds,
            scrollable_range_len: scrollable_range.len(),
            axis: PhantomData,
            is_scrolling: additional_offset != 0.0,
        }
    }

}


impl Axis for X {

    fn parallel_range(rect: Rect) -> Range {
        rect.x
    }

    fn perpendicular_range(rect: Rect) -> Range {
        rect.y
    }

    fn padding_range(padding: Padding) -> Range {
        padding.x
    }

    fn mouse_scalar(mouse_xy: Point) -> Scalar {
        mouse_xy[0]
    }

    fn offset_direction() -> Scalar {
        1.0
    }

}


impl Axis for Y {

    fn parallel_range(rect: Rect) -> Range {
        rect.y
    }

    fn perpendicular_range(rect: Rect) -> Range {
        rect.x
    }

    fn padding_range(padding: Padding) -> Range {
        padding.y
    }

    fn mouse_scalar(mouse_xy: Point) -> Scalar {
        mouse_xy[1]
    }

    fn offset_direction() -> Scalar {
        -1.0
    }

}
