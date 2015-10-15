//! 
//! Types and functionality related to the scrolling behaviour of widgets.
//!

use {Element, Scalar};
use color::Color;
use mouse::Mouse;
use position::{Dimensions, Point, Range, Rect};
use theme::Theme;
use utils::{map_range, partial_max};


/// A type for building a scrollbar widget.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Scrolling {
    /// Is there horizontal scrolling.
    pub horizontal: bool,
    /// Is there vertical scrolling.
    pub vertical: bool,
    /// Styling for the Scrolling.
    pub style: Style,
}


/// State related to scrolling.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State {
    /// vertical scrollbar.
    pub maybe_vertical: Option<Bar>,
    /// Horizontal scrollbar.
    pub maybe_horizontal: Option<Bar>,
    /// The rectangle representing the Visible area used tot calculate the Bar offsets.
    pub visible: Rect,
    /// The dimensions of the maximum bounding box around both the visible and kids Rects.
    pub total_dim: Dimensions,
    /// The width for vertical scrollbars, the height for horizontal scrollbars.
    pub thickness: Scalar,
    /// The color of the scrollbar.
    pub color: Color,
}


/// Style for the Scrolling.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    /// The width for vertical scrollbars, the height for horizontal scrollbars.
    pub maybe_thickness: Option<Scalar>,
    /// The color of the scrollbar.
    pub maybe_color: Option<Color>,
}


/// The state of a single scrollbar.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Bar {
    /// The current interaction with the Scrollbar.
    pub interaction: Interaction,
    /// The range to which the start of the visible range is bounded.
    pub scrollable: Range,
    /// The distance from the start of the scrollable range and the start of the visible range.
    /// i.e. visible.start - scrollable.start
    pub offset: Scalar,
    /// If the start of the visible range would start before the range of the widget's kids'
    /// bounding box, this amount will represent the difference as a positive scalar value.
    ///
    /// Otherwise, it will remain 0.0.
    ///
    /// We need to keep track of this as we should never let the offset become before the start
    /// overlap when being scrolled.
    pub start_overlap: Scalar,
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
    /// The draggable part of the bar and the mouse's position.
    Handle(Scalar),
    /// The track along which the bar can be dragged.
    Track,
}


impl Scrolling {
    /// Constructs the default Scrolling.
    pub fn new() -> Scrolling {
        Scrolling {
            vertical: false,
            horizontal: false,
            style: Style::new(),
        }
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

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.maybe_scrollbar.as_ref().map(|style| {
            style.maybe_color.unwrap_or(theme.shape_color.plain_contrast())
        })).unwrap_or(theme.shape_color.plain_contrast())
    }

}


impl Interaction {
    /// The stateful version of the given color.
    fn color(&self, color: Color) -> Color {
        match *self {
            Interaction::Normal => color,
            Interaction::Highlighted(_) => color.highlighted(),
            Interaction::Clicked(_) => color.clicked(),
        }
    }
}


impl State {

    /// Construct a new State.
    /// The `visible` rect corresponds to a Widget's `kid_area` aka the viewable container.
    /// The `kids` rect is the area *actually occupied* by the children widgets.
    pub fn new(scrolling: Scrolling, 
               visible: Rect,
               kids: Rect,
               theme: &Theme,
               maybe_prev: Option<&State>) -> State
    {
        State {

            maybe_vertical: if scrolling.vertical {
                let maybe_prev = maybe_prev.as_ref()
                    .and_then(|prev| prev.maybe_vertical.as_ref());
                // For a vertical scrollbar, we want the range to start at the top and end at
                // the bottom. To do this, we will use the invert of our visible and kids y ranges.
                Bar::new_if_scrollable(visible.y.invert(), kids.y.invert(), maybe_prev)
            } else {
                None
            },

            maybe_horizontal: if scrolling.horizontal {
                let maybe_prev = maybe_prev.as_ref()
                    .and_then(|prev| prev.maybe_horizontal.as_ref());
                Bar::new_if_scrollable(visible.x, kids.x, maybe_prev)
            } else {
                None
            },

            total_dim: visible.max(kids).dim(),
            visible: visible,
            thickness: scrolling.style.thickness(theme),
            color: scrolling.style.color(theme),
        }
    }

    /// Given some mouse input, update the State and return the resulting State.
    pub fn handle_input(self, mouse: Mouse) -> State {
        State {

            maybe_vertical: self.maybe_vertical.map(|bar| {
                let track = vertical_track(self.visible, self.thickness);
                let handle = vertical_handle(&bar, track, self.total_dim[1]);
                // Invert the visible y axis so that it points downward for vertical scrolling.
                let visible = self.visible.y.invert();
                let mouse_pos_scalar = mouse.xy[1] - track.top();
                bar.handle_input(visible, track, handle, &mouse, mouse_pos_scalar, mouse.scroll.y)
            }),

            maybe_horizontal: self.maybe_horizontal.map(|bar| {
                let track = horizontal_track(self.visible, self.thickness);
                let handle = horizontal_handle(&bar, track, self.total_dim[0]);
                let visible = self.visible.x;
                let mouse_pos_scalar = mouse.xy[0] - track.left();
                bar.handle_input(visible, track, handle, &mouse, mouse_pos_scalar, -mouse.scroll.x)
            }),

            .. self
        }
    }

    /// Is the given xy over either scroll Bars.
    pub fn is_over(&self, target_xy: Point) -> bool {
        if self.maybe_vertical.is_some() {
            if vertical_track(self.visible, self.thickness).is_over(target_xy) {
                return true;
            }
        }
        if self.maybe_horizontal.is_some() {
            if horizontal_track(self.visible, self.thickness).is_over(target_xy) {
                return true;
            }
        }
        false
    }

    /// Converts the Bars' current offset to a positional offset along its visible area.
    pub fn kids_pos_offset(&self) -> Dimensions {
        let maybe_x_offset = self.maybe_horizontal.map(|bar| bar.kids_pos_offset());
        let maybe_y_offset = self.maybe_vertical.map(|bar| bar.kids_pos_offset());
        [maybe_x_offset.unwrap_or(0.0), maybe_y_offset.unwrap_or(0.0)]
    }

    /// Produce a graphical element for the current scroll State.
    pub fn element(&self) -> Element {
        use elmesque::element::{empty, layers};
        use elmesque::form::{collage, rect};

        // Get the color via the current interaction.
        let color = self.color;
        let track_color = color.alpha(0.2);
        let thickness = self.thickness;
        let visible = self.visible;

        // An element for a scroll Bar.
        let bar_element = |bar: Bar, track: Rect, handle: Rect| -> Element {
            // We only want to see the scrollbar if it's highlighted or clicked.
            if let Interaction::Normal = bar.interaction {
                return empty();
            }
            let color = bar.interaction.color(color);
            let track_form = rect(track.w(), track.h()).filled(track_color)
                .shift(track.x(), track.y());
            let handle_form = rect(handle.w(), handle.h()).filled(color)
                .shift(handle.x(), handle.y());
            collage(visible.w() as i32, visible.h() as i32, vec![track_form, handle_form])
        };

        // The element for a vertical scroll Bar.
        let vertical = |bar: Bar| -> Element {
            let track = vertical_track(visible, thickness);
            let handle = vertical_handle(&bar, track, self.total_dim[1]);
            bar_element(bar, track, handle)
        };

        // An element for a horizontal scroll Bar.
        let horizontal = |bar: Bar| -> Element {
            let track = horizontal_track(visible, thickness);
            let handle = horizontal_handle(&bar, track, self.total_dim[0]);
            bar_element(bar, track, handle)
        };

        // Whether we draw horizontal or vertical or both depends on our state.
        match (self.maybe_vertical, self.maybe_horizontal) {
            (Some(v_bar), Some(h_bar)) => layers(vec![horizontal(h_bar), vertical(v_bar)]),
            (Some(bar), None) => vertical(bar),
            (None, Some(bar)) => horizontal(bar),
            (None, None) => empty(),
        }
    }

}


impl Bar {

    /// Construct a new Bar for a widget from a given visible range as well as the range occuppied
    /// by the widget's child widgets.
    ///
    /// The given `kids` Range should be relative to the visible range.
    ///
    /// If there is some previous Bar its interaction will be carried through to the Bar.
    pub fn new_if_scrollable(visible: Range, kids: Range, maybe_prev: Option<&Bar>) -> Option<Bar> {

        // The range occuppied by kid widgets when the scroll offset is at 0.0.
        let kids_at_origin = {
            let offset_from_origin = maybe_prev.map(|bar| bar.kids_pos_offset()).unwrap_or(0.0);
            kids.shift(-offset_from_origin)
        };

        // Total combined range of the visible and kids_at_origin ranges.
        let total = visible.max_directed(kids_at_origin);

        // The range that describes the area upon which the start of the visible range can scroll.
        let scrollable = Range::new(total.start .. total.end - visible.magnitude());

        // We only need to calculate offsets if we actually have some scrollable area.
        if scrollable.magnitude().is_normal() && scrollable.direction() == kids.direction() {

            let interaction = maybe_prev.map(|bar| bar.interaction)
                .unwrap_or(Interaction::Normal);

            // The amount that the visible range overlaps the start of the kids range when at its
            // origin position (non-scrolled).
            let start_overlap = {
                let start_diff_at_origin = kids_at_origin.start - visible.start;
                if start_diff_at_origin.signum() == visible.direction() {
                    start_diff_at_origin
                } else {
                    0.0
                }
            };

            // The positional scroll offset.
            let offset = maybe_prev.map(|bar| bar.offset).unwrap_or(0.0);

            Some(Bar {
                interaction: interaction,
                scrollable: scrollable,
                offset: offset,
                start_overlap: start_overlap,
            })
        // Otherwise our offsets are zeroed.
        } else {
            None
        }
    }


    /// Update a scroll `Bar` with the given mouse input.
    pub fn handle_input(self,
                        visible: Range,
                        track: Rect,
                        handle: Rect,
                        mouse: &Mouse,
                        mouse_pos_scalar: Scalar,
                        mouse_scroll_scalar: Scalar) -> Bar
    {
        use self::Elem::{Handle, Track};
        use self::Interaction::{Highlighted, Clicked};

        // Determine whether or not the mouse is over part of the Scrollbar.
        let is_over_elem = is_over_elem(track, handle, mouse, mouse_pos_scalar);

        // Determine the new current `Interaction`.
        let new_interaction = new_interaction(&self, is_over_elem, mouse, mouse_pos_scalar);

        // Calculate the maximum bar offset.
        let bar_offset_max = || {
            let bar_mag = handle.len() * visible.direction();
            track.len() * visible.direction() - bar_mag
        };

        // Calculate a positional offset given some bar offset.
        let pos_offset_from_bar_offset = |bar_offset: Scalar, bar_offset_max: Scalar| {
            map_range(bar_offset, 0.0, bar_offset_max, 0.0, self.scrollable.magnitude())
        };

        // Determine the new offset for the scrollbar.
        let new_offset = match (self.interaction, new_interaction) {

            // When the track is clicked and the handle snaps to the cursor.
            (Highlighted(Track), Clicked(Handle(mouse_pos_scalar))) => {
                // Should try snap the handle so that the mouse is in the middle of it.
                let direction = visible.direction();
                let half_handle_len = handle.len() * direction / 2.0;
                let target_offset = mouse_pos_scalar - half_handle_len;
                let target_pos_offset = pos_offset_from_bar_offset(target_offset, bar_offset_max());
                ::utils::clamp(target_pos_offset, 0.0, self.scrollable.magnitude())
            },

            // When the handle is dragged.
            (Clicked(Handle(prev_mouse_scalar)), Clicked(Handle(mouse_pos_scalar))) => {
                let scroll_amount = mouse_pos_scalar - prev_mouse_scalar;// * visible.direction();
                let pos_scroll_amount = pos_offset_from_bar_offset(scroll_amount, bar_offset_max());
                self.add_to_scroll_offset(pos_scroll_amount)
            },

            // The mouse has been scrolled using a wheel/trackpad/touchpad.
            (_, _) if mouse_scroll_scalar != 0.0 =>
                self.add_to_scroll_offset(mouse_scroll_scalar),

            // Otherwise, we'll assume the offset is unchanged.
            _ => self.offset,
        };

        Bar { interaction: new_interaction, offset: new_offset, ..self }
    }


    /// A function for shifting some current offset by some amount while ensuring it remains within the
    /// Bar's Range.
    fn add_to_scroll_offset(&self, amount: Scalar) -> Scalar {
        use utils::clamp;
        let target_offset = self.offset + amount;
        let min_offset = self.start_overlap;
        let max_offset = self.scrollable.magnitude();

        // If the offset is before the start, only let it be dragged towards the end.
        let clamp_current_to_max = || clamp(target_offset, self.offset, max_offset);
        // If the offset is past the end, only let it be dragged towards the start.
        let clamp_min_to_current = || clamp(target_offset, min_offset, self.offset);
        // Otherwise, clamp it between 0.0 and the max.
        let clamp_min_to_max = || clamp(target_offset, min_offset, max_offset);

        // For a positive range, check the start and end of the range normally.
        if max_offset >= 0.0 {
            if      self.offset < min_offset { clamp_current_to_max() }
            else if self.offset > max_offset { clamp_min_to_current() }
            else                             { clamp_min_to_max() }

        // Otherwise, check the inverse.
        } else {
            if      self.offset > min_offset { clamp_current_to_max() }
            else if self.offset < max_offset { clamp_min_to_current() }
            else                             { clamp_min_to_max() }
        }
    }


    /// Converts the Bar's current offset to the positional offset required to shift the children
    /// widgets in accordance with the scrolling.
    pub fn kids_pos_offset(&self) -> Scalar {
        let Bar { offset, scrollable, .. } = *self;
        if scrollable.len() == 0.0 { 0.0 } else { -offset }
    }

    /// Converts the given bar offset to a positional offset.
    ///
    /// TODO: Needs testing.
    pub fn pos_offset_from_bar_offset(&self, bar_offset: Scalar, bar_len: Scalar) -> Scalar {
        map_range(bar_offset, 0.0, bar_len, 0.0, self.scrollable.magnitude())
    }

}


/// The area for a vertical scrollbar track as its dimensions and position.
fn vertical_track(container: Rect, thickness: Scalar) -> Rect {
    let w = thickness;
    let x = container.x() + container.w() / 2.0 - w / 2.0;
    Rect {
        x: Range::from_pos_and_len(x, w),
        y: container.y,
    }
}

/// The area for a vertical scrollbar handle as its dimensions and position.
fn vertical_handle(bar: &Bar, track: Rect, total_len: Scalar) -> Rect {
    let offset = partial_max(bar.start_overlap - bar.offset, 0.0);
    let max_offset = bar.scrollable.len() - (bar.offset.abs() - offset);
    let track_h = track.h();
    let h = map_range(track_h, 0.0, total_len, 0.0, track_h);
    let y = map_range(offset, 0.0, max_offset, track.top(), track.bottom() + h) - h / 2.0;
    Rect {
        x: track.x,
        y: Range::from_pos_and_len(y, h),
    }
}

/// The area for a horizontal scrollbar track as its dimensions and position.
fn horizontal_track(container: Rect, thickness: Scalar) -> Rect {
    let h = thickness;
    let y = container.y() - container.h() / 2.0 + h / 2.0;
    Rect {
        x: container.x,
        y: Range::from_pos_and_len(y, h),
    }
}

/// The area for a horizontal scrollbar handle as its dimensions and position.
fn horizontal_handle(bar: &Bar, track: Rect, total_len: Scalar) -> Rect {
    let offset = partial_max(bar.offset - bar.start_overlap, 0.0);
    let max_offset = bar.scrollable.len() - (bar.offset.abs() - offset);
    let track_w = track.w();
    let w = map_range(track_w, 0.0, total_len, 0.0, track_w);
    let x = map_range(offset, 0.0, max_offset, track.left(), track.right() - w) + w / 2.0;
    Rect {
        x: Range::from_pos_and_len(x, w),
        y: track.y,
    }
}


/// Whether or not the mouse is currently over the Bar, and if so, which Elem.
fn is_over_elem(track: Rect, handle: Rect, mouse: &Mouse, mouse_scalar: Scalar) -> Option<Elem> {
    if handle.is_over(mouse.xy) {
        Some(Elem::Handle(mouse_scalar))
    } else if track.is_over(mouse.xy) {
        Some(Elem::Track)
    } else {
        None
    }
}

/// Determine the new current `Interaction` for a Bar.
/// The given mouse_scalar is the position of the mouse to be recorded by the Handle.
/// For vertical handle this is mouse.y, for horizontal this is mouse.x.
fn new_interaction(bar: &Bar,
                   is_over_elem: Option<Elem>,
                   mouse: &Mouse,
                   mouse_scalar: Scalar) -> Interaction
{
    use self::Interaction::{Normal, Highlighted, Clicked};

    // If there's no need for a scroll bar, leave the interaction as `Normal`.
    if bar.scrollable.len() == 0.0 {
        Normal
    } else {
        use self::Elem::Handle;
        use mouse::ButtonPosition::{Down, Up};
        match (is_over_elem, bar.interaction, mouse.left.position) {
            (Some(_),    Normal,             Down) => Normal,
            (Some(elem), _,                  Up)   => Highlighted(elem),
            (Some(_),    Highlighted(_),     Down) |
            (_,          Clicked(Handle(_)), Down) => Clicked(Handle(mouse_scalar)),
            (_,          Clicked(elem),      Down) => Clicked(elem),
            _                                      => Normal,
        }
    }
}



/// Whether or not the scrollbar should capture the mouse given previous and new states.
pub fn capture_mouse(prev: &State, new: &State) -> bool {
    match (prev.maybe_vertical, new.maybe_vertical) {
        (Some(ref prev_bar), Some(ref new_bar)) =>
            match (prev_bar.interaction, new_bar.interaction) {
                (Interaction::Highlighted(_), Interaction::Clicked(_)) => return true,
                _ => (),
            },
        _ => (),
    }
    match (prev.maybe_horizontal, new.maybe_horizontal) {
        (Some(ref prev_bar), Some(ref new_bar)) =>
            match (prev_bar.interaction, new_bar.interaction) {
                (Interaction::Highlighted(_), Interaction::Clicked(_)) => return true,
                _ => (),
            },
        _ => (),
    }
    false
}


/// Whether or not the scrollbar should uncapture the mouse given previous and new states.
pub fn uncapture_mouse(prev: &State, new: &State) -> bool {
    match (prev.maybe_vertical, new.maybe_vertical) {
        (Some(ref prev_bar), Some(ref new_bar)) =>
            match (prev_bar.interaction, new_bar.interaction) {
                (Interaction::Clicked(_), Interaction::Highlighted(_)) |
                (Interaction::Clicked(_), Interaction::Normal)         => return true,
                _ => (),
            },
        _ => (),
    }
    match (prev.maybe_horizontal, new.maybe_horizontal) {
        (Some(ref prev_bar), Some(ref new_bar)) =>
            match (prev_bar.interaction, new_bar.interaction) {
                (Interaction::Clicked(_), Interaction::Highlighted(_)) |
                (Interaction::Clicked(_), Interaction::Normal)         => return true,
                _ => (),
            },
        _ => (),
    }
    false
}


#[test]
fn test_bar_new_no_scroll() {
    // Create a `Bar` that shouldn't scroll.
    let visible = Range::new(-5.0..5.0);
    let kids = Range::new(-3.0..3.0);
    let maybe_bar = Bar::new_if_scrollable(visible, kids, None);
    assert_eq!(maybe_bar, None);
}

#[test]
fn test_bar_new_no_scroll_rev_range() {
    // Now with a reversed range.
    let visible = Range::new(5.0..-5.0);
    let kids = Range::new(3.0..-3.0);
    let maybe_bar = Bar::new_if_scrollable(visible, kids, None);
    assert_eq!(maybe_bar, None);
}

