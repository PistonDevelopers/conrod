//! Scroll related types and logic.

use {
    Align,
    Color,
    MouseScroll,
    Point,
    Padding,
    Range,
    Rect,
    Scalar,
    Theme,
    Ui,
};
use std::marker::PhantomData;
use ui;


/// Arguments given via a scrollable `Widget`'s builder methods for the scrolling along a single
/// axis.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scroll {
    maybe_initial_alignment: Option<Align>,
    style: Style,
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
    offset_bounds: Range,
    /// The total range which may be "offset" from the "root" range (aka the `kid_area`).
    ///
    /// The `scrollable_range` is determined as the bounding range around both the `kid_area` and
    /// all **un-scrolled** **visible** children widgets.
    scrollable_range_len: Scalar,
    /// The width for vertical scrollbars, the height for horizontal scrollbars.
    pub thickness: Scalar,
    /// The color of the scrollbar.
    pub color: Color,
    /// The current state of interaction between the mouse and the scrollbar.
    pub interaction: Interaction,
    /// The axis type used to instantiate this state.
    axis: PhantomData<A>,
    /// Whether or not the this axis is currently scrolling.
    pub is_scrolling: bool,
}

/// Style for the Scrolling.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    /// The width for vertical scrollbars, the height for horizontal scrollbars.
    pub maybe_thickness: Option<Scalar>,
    /// The color of the scrollbar.
    pub maybe_color: Option<Color>,
}

/// Represents an interaction between the mouse cursor and the scroll bar.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Interaction {
    /// There are currently no interactions.
    Normal,
    /// The mouse is over either the track or the handle of the scroll bar.
    Highlighted(Elem),
    /// The scrollbar handle is currently clicked by the mouse.
    Clicked(Elem),
}

/// The elements that make up the scrollbar.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Elem {
    /// The draggable part of the scrollbar and the mouse's position.
    Handle(Scalar),
    /// The track along which the `Handle` can be dragged.
    Track,
}

/// Methods for distinguishing behaviour between both scroll axes at compile-time.
pub trait Axis {
    /// The range of the given `Rect` that is parallel with this `Axis`.
    fn parallel_range(Rect) -> Range;
    /// The range of the given `Rect` that is perpendicular with this `Axis`.
    fn perpendicular_range(Rect) -> Range;
    /// Given some rectangular `Padding`, return the `Range` that corresponds with this `Axis`.
    fn padding_range(Padding) -> Range;
    /// The `Rect` for a scroll "track" with the given `thickness` for a container with the given
    /// `Rect`.
    fn track(container: Rect, thickness: Scalar) -> Rect;
    /// The coordinate of the given mouse position that corresponds with this `Axis`.
    fn mouse_scalar(mouse_xy: Point) -> Scalar;
    /// The coordinate of the given `MouseScroll` that corresponds with this `Axis`.
    fn mouse_scroll_axis(MouseScroll) -> Scalar;
    /// The `Rect` for a scroll handle given both `Range`s.
    fn handle_rect(perpendicular_track_range: Range, handle_range: Range) -> Rect;
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
            style: Style::new(),
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
    pub fn update<C>(ui: &mut Ui<C>,
                     idx: super::Index,
                     scroll_args: Scroll,
                     kid_area: &super::KidArea,
                     maybe_prev_scroll_state: Option<Self>) -> Self
    {

        // Retrieve the *current* scroll offset.
        let current_offset = maybe_prev_scroll_state.as_ref()
            .map(|state| state.offset)
            .unwrap_or(0.0);

        // Get the range for the Axis that concerns this particular scroll `State`.
        let kid_area_range = A::parallel_range(kid_area.rect);

        // The `kid_area_range` but centred at zero.
        let kid_area_range_origin = Range::from_pos_and_len(0.0, kid_area_range.magnitude());

        // The mouse state for the widget whose scroll state we're calculating.
        let maybe_mouse = ui::get_mouse_state(ui, idx);

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

        // Determine the min and max offst bounds. These bounds are the limits to which the
        // scrollable_range may be shifted in either direction across the range.
        let offset_bounds = {
            let padding = A::padding_range(kid_area.pad);
            let min_offset = Range::new(scrollable_range.start, kid_area_range_origin.start).magnitude();
            let max_offset = Range::new(scrollable_range.end, kid_area_range_origin.end).magnitude();
            Range::new(min_offset, max_offset).pad_ends(-padding.start, -padding.end)
        };

        // Determine the total `additional_scroll_offset` that we want to add to the
        // `current_offset`. We only need to check for additional offset and interactions if the
        // scrollable_range is actually longer than our kid_area.
        let (additional_offset, new_interaction) = if scrollable_range.len() > kid_area_range.len() {

            let (scroll_bar_drag_offset, new_interaction) = match maybe_prev_scroll_state {
                Some(prev_scroll_state) => {
                    use self::Elem::{Track, Handle};
                    use self::Interaction::{Normal, Highlighted, Clicked};
                    use utils::map_range;

                    let track = track::<A>(kid_area.rect, prev_scroll_state.thickness);
                    let handle = handle::<A>(track, &prev_scroll_state);
                    let handle_range = A::parallel_range(handle);
                    let handle_pos_range_len = || {
                        let track_range = A::parallel_range(track);
                        let handle_pos_at_start = handle_range.align_start_of(track_range).middle();
                        let handle_pos_at_end = handle_range.align_end_of(track_range).middle();
                        let handle_pos_range = Range::new(handle_pos_at_start, handle_pos_at_end);
                        handle_pos_range.len()
                    };
                    let prev_interaction = prev_scroll_state.interaction;
                    let is_scrollable = offset_bounds.len() > 0.0;
                    let new_interaction = match (maybe_mouse, is_scrollable) {
                        (Some(mouse), true) => {
                            use mouse::ButtonPosition::{Down, Up};
                            let mouse_scalar = A::mouse_scalar(mouse.xy);

                            // Check if the mouse is currently over part of the scroll bar.
                            let is_over_elem = if handle.is_over(mouse.xy) {
                                Some(Handle(mouse_scalar))
                            } else if track.is_over(mouse.xy) {
                                Some(Track)
                            } else {
                                None
                            };

                            // Determine the new `Interaction` between the mouse and scrollbar.
                            match (is_over_elem, prev_interaction, mouse.left.position) {
                                (Some(_),    Normal,             Down) => Normal,
                                (Some(elem), _,                  Up)   => Highlighted(elem),
                                (Some(_),    Highlighted(_),     Down) |
                                (_,          Clicked(Handle(_)), Down) => Clicked(Handle(mouse_scalar)),
                                (_,          Clicked(elem),      Down) => Clicked(elem),
                                _                                      => Normal,
                            }
                        },
                        _ => Normal,
                    };

                    // Check whether or not the mouse interactions require (un)capturing of mouse.
                    match (prev_interaction, new_interaction) {
                        (Highlighted(_), Clicked(_)) => { ui::mouse_captured_by(ui, idx); }
                        (Clicked(_), Highlighted(_)) |
                        (Clicked(_), Normal) => { ui::mouse_uncaptured_by(ui, idx); }
                        _ => (),
                    }

                    let scroll_bar_drag_offset = match (prev_interaction, new_interaction) {

                        // When the track is clicked and the handle snaps to the cursor.
                        (Highlighted(Track), Clicked(Handle(mouse_scalar))) => {
                            let handle_pos_range_len = handle_pos_range_len();
                            let offset_range_len = offset_bounds.len();
                            let pos_offset = mouse_scalar - handle_range.middle();
                            let offset = map_range(pos_offset,
                                                   0.0, handle_pos_range_len,
                                                   0.0, offset_range_len);
                            -offset
                        },

                        // When the handle is dragged.
                        (Clicked(Handle(prev_mouse_scalar)), Clicked(Handle(new_mouse_scalar))) => {
                            let handle_pos_range_len = handle_pos_range_len();
                            let offset_range_len = offset_bounds.len();
                            let pos_offset = new_mouse_scalar - prev_mouse_scalar;
                            let offset = map_range(pos_offset,
                                                   0.0, handle_pos_range_len,
                                                   0.0, offset_range_len);
                            -offset
                        },

                        _ => 0.0,
                    }.round();

                    (scroll_bar_drag_offset, new_interaction)
                },
                None => (0.0, Interaction::Normal),
            };

            // Additional offset from mouse scroll events provided by the window.
            let scroll_wheel_offset = {
                maybe_mouse
                    .map(|mouse| A::mouse_scroll_axis(mouse.scroll) * A::offset_direction())
                    .unwrap_or(0.0)
            };

            let additional_offset = scroll_bar_drag_offset + scroll_wheel_offset;

            (additional_offset, new_interaction)

        // Otherwise, our scrollable_range length is shorter than our kid_area, so no need for
        // additional scroll offset or interactions.
        } else {
            (0.0, Interaction::Normal)
        };

        let new_offset = offset_bounds.clamp_value(current_offset + additional_offset);

        State {
            offset: new_offset,
            offset_bounds: offset_bounds,
            scrollable_range_len: scrollable_range.len(),
            thickness: scroll_args.style.thickness(&ui.theme),
            color: scroll_args.style.color(&ui.theme),
            interaction: new_interaction,
            axis: PhantomData,
            is_scrolling: additional_offset != 0.0,
        }
    }

    /// Whether or not the given `xy` point is over the scroll track.
    pub fn is_over(&self, xy: Point, kid_area_rect: Rect) -> bool {
        A::track(kid_area_rect, self.thickness).is_over(xy)
    }

}


impl Style {

    /// Construct a new default Style.
    pub fn new() -> Style {
        Style {
            maybe_thickness: None,
            maybe_color: None,
        }
    }

    /// Get the thickness of the scrollbar or a default from the theme.
    pub fn thickness(&self, theme: &Theme) -> Scalar {
        const DEFAULT_THICKNESS: Scalar = 10.0;
        self.maybe_thickness.or(theme.maybe_scrollbar.as_ref().map(|style| {
            style.maybe_thickness.unwrap_or(DEFAULT_THICKNESS)
        })).unwrap_or(DEFAULT_THICKNESS)
    }

    /// Get the **Color** for the scrollbar.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.maybe_scrollbar.as_ref().map(|style| {
            style.maybe_color.unwrap_or(theme.shape_color.plain_contrast())
        })).unwrap_or(theme.shape_color.plain_contrast())
    }

}


/// Calculates the `Rect` for a scroll "track" with the given `thickness` over the given axis for
/// the given `container`.
pub fn track<A: Axis>(container: Rect, thickness: Scalar) -> Rect {
    A::track(container, thickness)
}

/// Calculates the `Rect` for a scroll "handle" sitting on the given `track` with an offset and
/// length that represents the given `Axis`' `state`.
pub fn handle<A: Axis>(track: Rect, state: &State<A>) -> Rect {
    let track_range = A::parallel_range(track);
    let track_len = track_range.len();
    let len = track_len * (track_len / state.scrollable_range_len);
    let handle_range = Range::from_pos_and_len(0.0, len);
    let pos = {
        let pos_min = handle_range.align_start_of(track_range).middle();
        let pos_max = handle_range.align_end_of(track_range).middle();
        let pos_bounds = Range::new(pos_min, pos_max);
        state.offset_bounds.map_value_to(state.offset, &pos_bounds)
    };
    let perpendicular_track_range = A::perpendicular_range(track);
    let range = Range::from_pos_and_len(pos, len);
    A::handle_rect(perpendicular_track_range, range)
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

    fn track(container: Rect, thickness: Scalar) -> Rect {
        let h = thickness;
        let w = container.w();
        let x = container.x();
        Rect::from_xy_dim([x, 0.0], [w, h]).align_bottom_of(container)
    }

    fn mouse_scalar(mouse_xy: Point) -> Scalar {
        mouse_xy[0]
    }

    fn mouse_scroll_axis(scroll: MouseScroll) -> Scalar {
        scroll.x
    }

    fn handle_rect(perpendicular_track_range: Range, handle_range: Range) -> Rect {
        Rect {
            x: handle_range,
            y: perpendicular_track_range,
        }
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

    fn track(container: Rect, thickness: Scalar) -> Rect {
        let w = thickness;
        let h = container.h();
        let y = container.y();
        Rect::from_xy_dim([0.0, y], [w, h]).align_right_of(container)
    }

    fn mouse_scalar(mouse_xy: Point) -> Scalar {
        mouse_xy[1]
    }

    fn mouse_scroll_axis(scroll: MouseScroll) -> Scalar {
        scroll.y
    }

    fn handle_rect(perpendicular_track_range: Range, handle_range: Range) -> Rect {
        Rect {
            x: perpendicular_track_range,
            y: handle_range,
        }
    }

    fn offset_direction() -> Scalar {
        -1.0
    }

}
