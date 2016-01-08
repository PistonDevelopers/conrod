//! Scroll related types and logic.

use {
    Align,
    Color,
    Mouse,
    Point,
    Range,
    Rect,
    Scalar,
    Theme,
    Ui,
};
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
pub struct State {
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
}

/// Style for the Scrolling.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    /// The width for vertical scrollbars, the height for horizontal scrollbars.
    pub maybe_thickness: Option<Scalar>,
    /// The color of the scrollbar.
    pub maybe_color: Option<Color>,
}


impl Scroll {
    /// The default `Scroll` args.
    pub fn new() -> Self {
        Scroll {
            maybe_initial_alignment: None,
            style: Style::new(),
        }
    }
}


impl State {

    /// Calculate the new scroll state for the single axis of a `Widget`.
    pub fn update<C>(ui: &Ui<C>,
                     idx: super::Index,
                     scroll_args: Scroll,
                     kid_area_range: Range,
                     maybe_prev_scroll_state: Option<State>) -> Self
    {

        // Retrieve the *current* scroll offset.
        let current_offset = maybe_prev_scroll_state
            .map(|state| state.offset)
            .unwrap_or(0.0);

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
                .map(|kids| kids.y.shift(-current_offset).shift(kid_area_range.middle()))
                .unwrap_or_else(|| kid_area_range);
            kid_area_range.max(kids_bounding_range)
        };

        // Determine the min and max scroll bounds.
        let offset_bounds = {
            let max_offset = scrollable_range.end - kid_area_range.end;
            let min_offset = kid_area_range.start - scrollable_range.start;
            Range { start: min_offset, end: max_offset }
        };

        // Determine the total `additional_scroll_offset` that we want to add to the
        // `current_offset`.
        let additional_offset = {

            let scroll_bar_offset = {
                0.0
            };

            let scroll_wheel_offset = {
                ui::get_mouse_state(ui, idx)
                    .map(|mouse| -mouse.scroll.y)
                    .unwrap_or(0.0)
            };

            scroll_bar_offset + scroll_wheel_offset
        };

        let new_offset = offset_bounds.clamp_value(current_offset + additional_offset);

        State {
            offset: new_offset,
            offset_bounds: offset_bounds,
            scrollable_range_len: scrollable_range.len(),
            thickness: scroll_args.style.thickness(&ui.theme),
            color: scroll_args.style.color(&ui.theme),
        }
    }

    /// Whether or not the given `xy` point is over the scroll track.
    pub fn is_over(&self, xy: Point, kid_area_rect: Rect) -> bool {
        track(kid_area_rect, self.thickness).is_over(xy)
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


pub fn track(container: Rect, thickness: Scalar) -> Rect {
    let w = thickness;
    let h = container.h();
    let y = container.y();
    Rect::from_xy_dim([0.0, y], [w, h]).align_right_of(container)
}

pub fn handle(track: Rect, state: &State) -> Rect {
    let track_len = track.h();
    let len = track_len * (track_len / state.scrollable_range_len);
    let handle_range = Range::from_pos_and_len(0.0, len);
    let pos = {
        let pos_min = handle_range.align_start_of(track.y).middle();
        let pos_max = handle_range.align_end_of(track.y).middle();
        let pos_bounds = Range::new(pos_min, pos_max);
        state.offset_bounds.map_value_to(state.offset, &pos_bounds)
    };
    Rect {
        x: track.x,
        y: Range::from_pos_and_len(pos, len),
    }
}
