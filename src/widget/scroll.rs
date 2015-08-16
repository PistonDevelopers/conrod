
use {Element, Scalar};
use color::Color;
use mouse::Mouse;
use position::{Dimensions, Point};
use theme::Theme;
use utils::is_over_rect;
use widget;


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
    /// The width for vertical scrollbars, the height for horizontal scrollbars.
    pub width: Scalar,
    /// The color of the scrollbar.
    pub color: Color,
}


/// Style for the Scrolling.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    /// The width for vertical scrollbars, the height for horizontal scrollbars.
    pub maybe_width: Option<Scalar>,
    /// The color of the scrollbar.
    pub maybe_color: Option<Color>,
}


/// The state of a single scrollbar.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Bar {
    /// The current interaction with the Scrollbar.
    pub interaction: Interaction,
    /// The current scroll position as an offset from the top left.
    pub offset: Scalar,
    /// The maximum possible offset for the handle.
    pub max_offset: Scalar,
    /// The total length of the area occupied by child widgets.
    pub total_length: Scalar,
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
            maybe_width: None,
            maybe_color: None,
        }
    }

    /// Get the width of the scrollbar or a default from the theme.
    pub fn width(&self, theme: &Theme) -> Scalar {
        const DEFAULT_WIDTH: Scalar = 10.0;
        self.maybe_width.or(theme.maybe_scrollbar.as_ref().map(|style| {
            style.maybe_width.unwrap_or(DEFAULT_WIDTH)
        })).unwrap_or(DEFAULT_WIDTH)
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.maybe_scrollbar.as_ref().map(|style| {
            style.maybe_color.unwrap_or(theme.shape_color.complement())
        })).unwrap_or(theme.shape_color.complement())
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


/// The new state of the given scrollbar if it has changed.
pub fn update(kid_area: &widget::KidArea,
              state: &State,
              maybe_mouse: Option<Mouse>) -> Option<State>
{
    use self::Elem::{Handle, Track};
    use self::Interaction::{Normal, Highlighted, Clicked};
    use utils::clamp;

    let width = state.width;

    // Determine the new current `Interaction` for a Scrollbar.
    // The given mouse_scalar is the position of the mouse to be recorded by the Handle.
    // For vertical handle this is mouse.y, for horizontal this is mouse.x.
    let get_new_interaction = |bar: &Bar,
                               is_over_elem: Option<Elem>,
                               mouse_scalar: Scalar| -> Interaction
    {
        if let Some(mouse) = maybe_mouse {
            use mouse::ButtonState::{Down, Up};
            match (is_over_elem, bar.interaction, mouse.left) {
                (Some(_),    Normal,             Down) => Normal,
                (Some(elem), _,                  Up)   => Highlighted(elem),
                (Some(_),    Highlighted(_),     Down) |
                (_,          Clicked(Handle(_)), Down) => Clicked(Handle(mouse_scalar)),
                (_,          Clicked(elem),      Down) => Clicked(elem),
                _                                      => Normal,
            }
        } else {
            Normal
        }
    };

    // Simplify construction of a new Bar.
    let maybe_new_bar = |bar: &Bar, new_interaction: Interaction, new_offset: Scalar| {
        // Has the `Bar` changed at all.
        let has_changed = bar.interaction != new_interaction || bar.offset != new_offset;
        // Construct a new `Bar` if it has changed.
        if has_changed {
            Some(Bar {
                interaction: new_interaction,
                offset: new_offset,
                max_offset: bar.max_offset,
                total_length: bar.total_length,
            })
        } else {
            None
        }
    };

    // Gives the updated vertical `Bar` if it has changed.
    let vertical = |bar: &Bar| -> Option<Bar> {

        let (track_dim, track_xy) = vertical_track_area(kid_area, width);
        let (handle_dim, handle_xy) =
            vertical_handle_area(track_dim, track_xy, width, bar.offset, bar.max_offset);

        // Determine whether or not the mouse is over part of the Scrollbar.
        let is_over_elem = maybe_mouse.and_then(|mouse| {
            if is_over_rect(track_xy, mouse.xy, track_dim) {
                if is_over_rect(handle_xy, mouse.xy, handle_dim) {
                    Some(Handle(mouse.xy[1]))
                } else {
                    Some(Track)
                }
            } else {
                None
            }
        });

        // Determine the new current `Interaction`.
        let new_interaction = match maybe_mouse {
            Some(mouse) => get_new_interaction(bar, is_over_elem, mouse.xy[1]),
            None => Normal,
        };

        // Determine the new offset for the scrollbar.
        let new_offset = match (bar.interaction, new_interaction, maybe_mouse) {

            // When the track is clicked and the handle snaps to the cursor.
            (Highlighted(Track), Clicked(Handle(mouse_y)), _) => {
                // Should try snap the handle so that the mouse is in the middle of it.
                let track_top = track_xy[1] + track_dim[1] / 2.0;
                let target_offset = -((mouse_y - track_top) + handle_dim[1] / 2.0);
                clamp(target_offset, 0.0, bar.max_offset)
            },

            // When the handle is dragged.
            (Clicked(Handle(prev_mouse_y)), Clicked(Handle(mouse_y)), _) => {
                let diff = prev_mouse_y - mouse_y;
                clamp(bar.offset + diff, 0.0, bar.max_offset)
            },

            // The mouse has been scrolled using a wheel/trackpad/touchpad.
            (_, _, Some(mouse)) if mouse.scroll.y != 0.0 => {
                clamp(bar.offset - mouse.scroll.y, 0.0, bar.max_offset)
            },

            // Otherwise, we'll assume the offset is unchanged.
            _ => bar.offset,

        };

        // Check to see if the bar has changed and return a new bar if it has.
        maybe_new_bar(bar, new_interaction, new_offset)
    };

    // Gives the updated horizontal `Bar` if it has changed.
    let horizontal = |bar: &Bar| -> Option<Bar> {

        let (track_dim, track_xy) = horizontal_track_area(kid_area, width);
        let (handle_dim, handle_xy) =
            horizontal_handle_area(track_dim, track_xy, width, bar.offset, bar.max_offset);

        // Determine whether or not the mouse is over part of the Scrollbar.
        let is_over_elem = maybe_mouse.and_then(|mouse| {
            if is_over_rect(track_xy, mouse.xy, track_dim) {
                if is_over_rect(handle_xy, mouse.xy, handle_dim) {
                    Some(Handle(mouse.xy[0]))
                } else {
                    Some(Track)
                }
            } else {
                None
            }
        });

        // Determine the new current `Interaction`.
        let new_interaction = match maybe_mouse {
            Some(mouse) => get_new_interaction(bar, is_over_elem, mouse.xy[0]),
            None => Normal,
        };

        // Determine the new offset for the scrollbar.
        let new_offset = match (bar.interaction, new_interaction, maybe_mouse) {

            // When the track is clicked and the handle snaps to the cursor.
            (Highlighted(Track), Clicked(Handle(mouse_x)), _) => {
                // Should try snap the handle so that the mouse is in the middle of it.
                let track_left = track_xy[0] - track_dim[1] / 2.0;
                let target_offset = (mouse_x - track_left) + handle_dim[0] / 2.0;
                clamp(target_offset, 0.0, bar.max_offset)
            },

            // When the handle is dragged.
            (Clicked(Handle(prev_mouse_x)), Clicked(Handle(mouse_x)), _) => {
                let diff = prev_mouse_x - mouse_x;
                clamp(bar.offset + diff, 0.0, bar.max_offset)
            },

            // The mouse has been scrolled using a wheel/trackpad/touchpad.
            (_, _, Some(mouse)) if mouse.scroll.x != 0.0 => {
                clamp(bar.offset + mouse.scroll.x, 0.0, bar.max_offset)
            },

            // Otherwise, we'll assume the offset is unchanged.
            _ => bar.offset,

        };

        // Check to see if the bar has changed and return a new bar if it has.
        maybe_new_bar(bar, new_interaction, new_offset)
    };

    // Produce a new scroll state if there has been any changes in either bar.
    match (&state.maybe_vertical, &state.maybe_horizontal) {

        // We have both vertical and horizontal bars.
        (&Some(ref v_bar), &Some(ref h_bar)) => match (vertical(v_bar), horizontal(h_bar)) {
            (None, None) => None,
            (Some(new_v_bar), None) => Some(State { maybe_vertical: Some(new_v_bar), ..*state }),
            (None, Some(new_h_bar)) => Some(State { maybe_horizontal: Some(new_h_bar), ..*state }),
            (Some(new_v_bar), Some(new_h_bar)) =>
                Some(State {
                    maybe_vertical: Some(new_v_bar),
                    maybe_horizontal: Some(new_h_bar),
                    ..*state
                }),
        },

        // We only have a vertical scrollbar.
        (&Some(ref v_bar), &None) => vertical(v_bar).map(|new_v_bar| {
            State { maybe_vertical: Some(new_v_bar), ..*state }
        }),

        // We only have a horizontal scrollbar.
        (&None, &Some(ref h_bar)) => horizontal(h_bar).map(|new_h_bar| {
            State { maybe_horizontal: Some(new_h_bar), ..*state }
        }),

        // We don't have any scrollbars.
        (&None, &None) => None,
    }
}


/// Construct a renderable Element from the state for the given widget's kid area.
pub fn element(kid_area: &widget::KidArea, state: State) -> Element {
    use elmesque::element::{empty, layers};
    use elmesque::form::{collage, rect};

    // Get the color via the current interaction.
    let color = state.color;
    let track_color = color.alpha(0.2);
    let width = state.width;

    // The element for a vertical slider.
    let vertical = |bar: Bar| -> Element {
        let color = bar.interaction.color(color);
        let (track_dim, track_xy) = vertical_track_area(kid_area, width);
        let (handle_dim, handle_xy) =
            vertical_handle_area(track_dim, track_xy, width, bar.offset, bar.max_offset);
        let track_form = rect(track_dim[0], track_dim[1]).filled(track_color)
            .shift(track_xy[0], track_xy[1]);
        let handle_form = rect(handle_dim[0], handle_dim[1]).filled(color)
            .shift(handle_xy[0], handle_xy[1]);
        collage(kid_area.dim[0] as i32, kid_area.dim[1] as i32, vec![track_form, handle_form])
    };

    // An element for a horizontal slider.
    let horizontal = |bar: Bar| -> Element {
        let color = bar.interaction.color(color);
        let (track_dim, track_xy) = horizontal_track_area(kid_area, width);
        let (handle_dim, handle_xy) =
            horizontal_handle_area(track_dim, track_xy, width, bar.offset, bar.max_offset);
        let track_form = rect(track_dim[0], track_dim[1]).filled(track_color)
            .shift(track_xy[0], track_xy[1]);
        let handle_form = rect(handle_dim[0], handle_dim[1]).filled(color)
            .shift(handle_xy[0], handle_xy[1]);
        collage(kid_area.dim[0] as i32, kid_area.dim[1] as i32, vec![track_form, handle_form])
    };

    // Whether we draw horizontal or vertical or both depends on our state.
    match (state.maybe_vertical, state.maybe_horizontal) {
        (Some(v_bar), Some(h_bar)) => layers(vec![horizontal(h_bar), vertical(v_bar)]),
        (Some(bar), None) => vertical(bar),
        (None, Some(bar)) => horizontal(bar),
        (None, None) => empty(),
    }
}


/// The area for a vertical scrollbar track as its dimensions and position.
fn vertical_track_area(container: &widget::KidArea, width: Scalar) -> (Dimensions, Point) {
    let w = width;
    let h = container.dim[1];
    let x = container.xy[0] + container.dim[0] / 2.0 - w / 2.0;
    let y = container.xy[1];
    ([w, h], [x, y])
}

/// The area for a vertical scrollbar handle as its dimensions and position.
fn vertical_handle_area(track_dim: Dimensions,
                        track_xy: Point,
                        width: Scalar,
                        offset: Scalar,
                        max_offset: Scalar) -> (Dimensions, Point)
{
    let w = width;
    let h = track_dim[1] - max_offset;
    let x = track_xy[0];
    let top_of_track = track_xy[1] + track_dim[1] / 2.0;
    let y = top_of_track - offset - (h / 2.0);
    ([w, h], [x, y])
}

/// The area for a horizontal scrollbar track as its dimensions and position.
fn horizontal_track_area(container: &widget::KidArea, width: Scalar) -> (Dimensions, Point) {
    let w = container.dim[0];
    let h = width;
    let x = container.xy[0];
    let y = container.xy[1] - container.dim[1] / 2.0 + h / 2.0;
    ([w, h], [x, y])
}

/// The area for a horizontal scrollbar handle as its dimensions and position.
fn horizontal_handle_area(track_dim: Dimensions,
                          track_xy: Point,
                          width: Scalar,
                          offset: Scalar,
                          max_offset: Scalar) -> (Dimensions, Point)
{
    let w = track_dim[1] - max_offset;
    let h = width;
    let left_of_track = track_xy[0] - track_dim[0] / 2.0;
    let x = left_of_track + offset + (w / 2.0);
    let y = track_xy[1];
    ([w, h], [x, y])
}


/// Is the given xy over the area of a scrollbar with the given state.
pub fn is_over(state: &State, kid_area: &widget::KidArea, target_xy: Point) -> bool {
    if state.maybe_vertical.is_some() {
        let (track_dim, track_xy) = vertical_track_area(kid_area, state.width);
        if is_over_rect(track_xy, target_xy, track_dim) {
            return true;
        }
    } else if state.maybe_horizontal.is_some() {
        let (track_dim, track_xy) = horizontal_track_area(kid_area, state.width);
        if is_over_rect(track_xy, target_xy, track_dim) {
            return true;
        }
    }
    false
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

