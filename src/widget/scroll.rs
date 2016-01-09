//! Scroll related types and logic.

use {
    Align,
    Color,
    Mouse,
    MouseScroll,
    Point,
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
    fn parallel_range(Rect) -> Range;
    fn perpendicular_range(Rect) -> Range;
    fn track(container: Rect, thickness: Scalar) -> Rect;
    fn mouse_scalar(mouse_xy: Point) -> Scalar;
    fn mouse_scroll_axis(MouseScroll) -> Scalar;
    fn handle_rect(perpendicular_track_range: Range, handle_range: Range) -> Rect;
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
    pub fn update<C>(ui: &mut Ui<C>,
                     idx: super::Index,
                     scroll_args: Scroll,
                     kid_area_rect: Rect,
                     maybe_prev_scroll_state: Option<Self>) -> Self
    {

        // Retrieve the *current* scroll offset.
        let current_offset = maybe_prev_scroll_state.as_ref()
            .map(|state| state.offset)
            .unwrap_or(0.0);

        // Get the range for the Axis that concerns this particular scroll `State`.
        let kid_area_range = A::parallel_range(kid_area_rect);

        // The mouse state for the widget whose scroll state we're calculating.
        let maybe_mouse = ui::get_mouse_state(ui, idx);

        // Retrieve the entire scrollable_range. This is the total range which may be "offset" from
        // the "root" range (aka the `kid_area`). The scrollable_range is determined as the
        // bounding range around both the kid_area and all **un-scrolled** **visible** children
        // widgets. Note that this means when reading all children widget's `Range`s to construct
        // the bounding `Range`, we must first subtract the `current_scroll_offset` in order to get
        // the *original* aka *un-scrolled* position. Thus, the calculated `scrollable_range`
        // should be entirely unaffected by the `scroll_offset`. Notice also that we add the
        // position of the `kid_area` to the `kids_bounding_range`. This allows us to create the
        // `scrollable_range` relative to the `kid_area_range`.
        let scrollable_range = {
            let kids_bounding_range = ui.kids_bounding_box(idx)
                .map(|kids| {
                    A::parallel_range(kids)
                        .shift(-current_offset)
                        .shift(kid_area_range.middle())
                })
                .unwrap_or_else(|| kid_area_range);
            kid_area_range.max(kids_bounding_range)
        };

        // Determine the min and max offst bounds. These bounds are the limits to which the
        // scrollable_range may be shifted in either direction across the range.
        let offset_bounds = {
            let min_offset = kid_area_range.start - scrollable_range.start;
            let max_offset = scrollable_range.end - kid_area_range.end;
            let max_offset = min_offset - (max_offset - min_offset) * A::offset_direction();
            Range { start: min_offset, end: max_offset }
        };

        // Determine the total `additional_scroll_offset` that we want to add to the
        // `current_offset`.
        let (additional_offset, new_interaction) = {

            let (scroll_bar_drag_offset, new_interaction) = match maybe_prev_scroll_state {
                Some(prev_scroll_state) => {
                    use self::Elem::{Track, Handle};
                    use self::Interaction::{Normal, Highlighted, Clicked};
                    use utils::map_range;

                    let track = track::<A>(kid_area_rect, prev_scroll_state.thickness);
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
                    };

                    (scroll_bar_drag_offset, new_interaction)
                },
                None => (0.0, Interaction::Normal),
            };

            let scroll_wheel_offset = {
                maybe_mouse
                    .map(|mouse| A::mouse_scroll_axis(mouse.scroll) * A::offset_direction())
                    .unwrap_or(0.0)
            };

            let additional_offset = scroll_bar_drag_offset + scroll_wheel_offset;

            (additional_offset, new_interaction)
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

pub fn track<A: Axis>(container: Rect, thickness: Scalar) -> Rect {
    A::track(container, thickness)
}


impl Axis for X {

    fn parallel_range(rect: Rect) -> Range {
        rect.x
    }

    fn perpendicular_range(rect: Rect) -> Range {
        rect.y
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
