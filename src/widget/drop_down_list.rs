
use canvas::CanvasId;
use color::{Color, Colorable};
use elmesque::Element;
use frame::Frameable;
use graphics::character::CharacterCache;
use graphics::math::Scalar;
use label::{FontSize, Labelable};
use mouse::Mouse;
use position::{self, Depth, Dimensions, HorizontalAlign, Point, Position, Positionable, VerticalAlign};
use theme::Theme;
use ui::{GlyphCache, UserInput};
use widget::{self, Widget};


/// The index of a selected item.
pub type Idx = usize;
/// The number of items in a list.
pub type Len = usize;

/// The width of the scrollbar when visible.
pub const SCROLLBAR_WIDTH: f64 = 10.0;

/// Displays a given `Vec<String>` as a selectable drop down menu. It's reaction is triggered upon
/// selection of a list item.
pub struct DropDownList<'a, F> {
    strings: &'a mut Vec<String>,
    selected: &'a mut Option<Idx>,
    pos: Position,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    depth: Depth,
    maybe_react: Option<F>,
    maybe_label: Option<&'a str>,
    style: Style,
    enabled: bool,
    maybe_canvas_id: Option<CanvasId>,
}

/// Styling for the DropDownList, necessary for constructing its renderable Element.
#[derive(PartialEq, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Style {
    /// Width of the widget.
    pub maybe_width: Option<Scalar>,
    /// Height of the widget.
    pub maybe_height: Option<Scalar>,
    /// Color of the widget.
    pub maybe_color: Option<Color>,
    /// Width of the widget's frame.
    pub maybe_frame: Option<f64>,
    /// Color of the widget's frame.
    pub maybe_frame_color: Option<Color>,
    /// Color of the item labels.
    pub maybe_label_color: Option<Color>,
    /// Font size for the item labels.
    pub maybe_label_font_size: Option<u32>,
    /// Maximum height of the Open menu before the scrollbar appears.
    pub maybe_max_visible_height: Option<MaxHeight>,
}

/// Represents the state of the DropDownList.
#[derive(PartialEq, Clone, Debug)]
pub struct State {
    menu_state: MenuState,
    maybe_label: Option<String>,
    strings: Vec<String>,
    maybe_selected: Option<Idx>,
}

/// Position of the scroll bar and the height at which it appears.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Scroll {
    /// The current position of the scroll bar as y-axis offset from top.
    y_offset: Scalar,
    /// The maximum offset for the handle given the number of items.
    max_offset: Scalar,
    /// The maximum height for the display before scrollbar appears.
    max_visible_height: Scalar,
}

/// Representations of the max height of the visible area of the DropDownList.
#[derive(PartialEq, Clone, Copy, Debug, RustcEncodable, RustcDecodable)]
pub enum MaxHeight {
    Items(usize),
    Scalar(f64),
}

/// Whether the DropDownList is currently open or closed.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MenuState {
    Closed(Interaction),
    Open(Interaction, Option<Scroll>),
}

/// Describes how the DropDownList is currently being interacted with.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Interaction {
    Normal,
    Highlighted(Elem),
    Clicked(Elem),
}

/// The different elements that make up the Open DropDownList.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Elem {
    ScrollBar(ScrollBar),
    Item(Idx)
}

/// The elements that make up a ScrollBar.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ScrollBar {
    /// The draggable bar and the mouse's position.
    Handle(Point),
    /// The track along which the bar can be dragged.
    Track(Point),
}

impl Interaction {
    fn color(&self, color: Color) -> Color {
        match *self {
            Interaction::Normal => color,
            Interaction::Highlighted(_) => color.highlighted(),
            Interaction::Clicked(_) => color.clicked(),
        }
    }
}


/// Is the cursor currently over the widget? If so, which Elem?
fn is_over(mouse_pos: Point,
           frame_w: f64,
           dim: Dimensions,
           menu_state: MenuState,
           len: Len,
           maybe_scroll: Option<(Scalar, Scalar, Scalar)>) -> Option<Elem> {
    use utils::is_over_rect;
    match menu_state {

        MenuState::Closed(_) => match is_over_rect([0.0, 0.0], mouse_pos, dim) {
            false => None,
            true => Some(Elem::Item(0)),
        },

        MenuState::Open(ref interaction, _) => {
            let item_h = dim[1] - frame_w;
            let total_h = item_h * len as f64;

            // If the menu is scrollable.
            if let Some((y_offset, track_h, max_offset)) = maybe_scroll {
                //let centre_y = -(track_h - item_h) / 2.0;
                let middle_y = item_h / 2.0 - track_h / 2.0;
                match *interaction {
                    Interaction::Highlighted(_) | Interaction::Clicked(_) => {
                        let scroll_x = position::align_right_of(dim[0], SCROLLBAR_WIDTH);
                        let handle_h = (track_h / total_h) * track_h;
                        let handle_y = middle_y + position::align_top_of(track_h, handle_h) - y_offset;
                        let handle_dim = [SCROLLBAR_WIDTH, handle_h];
                        let track_dim = [SCROLLBAR_WIDTH, track_h];
                        if is_over_rect([scroll_x, handle_y], mouse_pos, handle_dim) {
                            return Some(Elem::ScrollBar(ScrollBar::Handle(mouse_pos)));
                        } else if is_over_rect([scroll_x, middle_y], mouse_pos, track_dim) {
                            return Some(Elem::ScrollBar(ScrollBar::Track(mouse_pos)));
                        }
                    }
                    _ => (),
                }
                match is_over_rect([0.0, middle_y], mouse_pos, [dim[0], track_h]) {
                    false => None,
                    true => {
                        let scroll_amt = (y_offset / max_offset) * (total_h - track_h);
                        let idx = ((mouse_pos[1] - scroll_amt - item_h / 2.0).abs() / item_h) as Idx;
                        Some(Elem::Item(idx))
                    },
                }
            }

            // If the menu is not scrollable.
            else {
                let centre_y = -(total_h - item_h) / 2.0;
                match is_over_rect([0.0, centre_y], mouse_pos, [dim[0], total_h]) {
                    false => None,
                    true => {
                        let idx = ((mouse_pos[1] - item_h / 2.0).abs() / item_h) as Idx;
                        Some(Elem::Item(idx))
                    },
                }
            }
        },

    }
}


/// Determine and return the new State by comparing the mouse state
/// and position to the previous State.
fn get_new_menu_state(is_over_elem: Option<Elem>,
                      menu_state: MenuState,
                      height: Scalar,
                      mouse: Mouse,
                      maybe_scroll: Option<(Scalar, Scalar, Scalar)>) -> MenuState {
    use self::Interaction::{Normal, Clicked, Highlighted};
    use self::Elem::{Item, ScrollBar};
    use self::ScrollBar::{Handle, Track};
    use self::MenuState::{Closed, Open};
    use mouse::ButtonState::{Down, Up};

    let new_scroll = || maybe_scroll.map(|(y_offset, max_height, max_offset)| Scroll {
        y_offset: y_offset,
        max_offset: max_offset,
        max_visible_height: max_height,
    });

    let shift_scroll = |prev_mouse_xy: Point| {
        maybe_scroll.map(|(y_offset, max_height, max_offset)| {
            let y_offset = y_offset + (prev_mouse_xy[1] - mouse.xy[1]);
            let y_offset = ::utils::clamp(y_offset, 0.0, max_offset);
            Scroll {
                y_offset: y_offset,
                max_offset: max_offset,
                max_visible_height: max_height,
            }
        })
    };

    let scroll_with_mouse = |maybe_scroll: Option<Scroll>| {
        maybe_scroll.map(|scroll| {
            let y_offset = scroll.y_offset;
            let new_y_offset = ::utils::clamp(y_offset - mouse.scroll.y, 0.0, scroll.max_offset);
            Scroll { y_offset: new_y_offset, ..scroll }
        })
    };

    match menu_state {

        MenuState::Closed(draw_state) => match is_over_elem {
            Some(_) => match (draw_state, mouse.left) {
                (Normal,         Down) => Closed(Normal),
                (Normal,         Up)   |
                (Highlighted(_), Up)   => Closed(Highlighted(Item(0))),
                (Highlighted(_), Down) => Closed(Clicked(Item(0))),
                (Clicked(_),     Down) => Closed(Clicked(Item(0))),
                (Clicked(_),     Up)   => Open(Normal, new_scroll()),
            },
            None => MenuState::Closed(Normal),
        },

        MenuState::Open(draw_state, _prev_maybe_scroll) => {
            match is_over_elem {
                Some(elem) => match (draw_state, mouse.left, elem) {
                    (Normal,         Down, _) => Open(Normal, new_scroll()),
                    (Normal,         Up,   _) |
                    (Highlighted(_), Up,   _) =>
                        Open(Highlighted(elem), scroll_with_mouse(new_scroll())),

                    // Scroll bar has just been clicked.
                    (Highlighted(_), Down, ScrollBar(bar)) => {
                        match bar {
                            Handle(_) =>
                                Open(Clicked(ScrollBar(Handle(mouse.xy))), new_scroll()),
                            Track(_) => {
                                let maybe_scroll = maybe_scroll.map(|(_y_offset, max_height, max_offset)| {
                                    let top = height / 2.0;
                                    let handle_h = max_height - max_offset;
                                    let picked_y = (mouse.xy[1] - top).abs() - handle_h / 2.0;
                                    let y_offset = ::utils::clamp(picked_y, 0.0, max_offset);
                                    Scroll {
                                        y_offset: y_offset,
                                        max_visible_height: max_height,
                                        max_offset: max_offset,
                                    }
                                });
                                Open(Clicked(ScrollBar(Handle(mouse.xy))), maybe_scroll)
                            },
                        }
                    },

                    // An item has just been clicked.
                    (Highlighted(_), Down, _) => Open(Clicked(elem), new_scroll()),

                    // The scroll bar handle has just been dragged.
                    (Clicked(ScrollBar(Handle(xy))), Down, _) =>
                        Open(Clicked(ScrollBar(Handle(mouse.xy))), shift_scroll(xy)),

                    // An item remains clicked.
                    (Clicked(elem), Down, _)           => Open(Clicked(elem), new_scroll()),

                    // The scrollbar, previously clicked, was released above an item.
                    (Clicked(ScrollBar(_)), Up, Elem::Item(idx)) =>
                        Open(Highlighted(Elem::Item(idx)), new_scroll()),

                    // An item was selected.
                    (Clicked(_),    Up, Elem::Item(_)) => Closed(Normal),

                    // The scrollbar was released but remains highlighted.
                    // TODO
                    (Clicked(_),    Up, scroll)        =>
                        Open(Highlighted(scroll), new_scroll()),
                },

                None => match (draw_state, mouse.left) {
                    (Highlighted(_), Up)  => Open(Normal, new_scroll()),
                    (Clicked(ScrollBar(Handle(xy))), Down) =>
                        Open(Clicked(ScrollBar(Handle(mouse.xy))), shift_scroll(xy)),
                    (Clicked(_), Up)      => Open(Normal, new_scroll()),
                    (Clicked(elem), Down) => Open(Clicked(elem), new_scroll()),
                    (Normal, Up)          => Open(Normal, new_scroll()),
                    (Normal, Down)        => Closed(Normal),
                    _                     => Closed(Normal),
                },
            }
        }

    }
}

impl<'a, F> DropDownList<'a, F> {

    /// Construct a new DropDownList.
    pub fn new(strings: &'a mut Vec<String>, selected: &'a mut Option<Idx>) -> DropDownList<'a, F> {
        DropDownList {
            strings: strings,
            selected: selected,
            pos: Position::default(),
            maybe_h_align: None,
            maybe_v_align: None,
            depth: 0.0,
            maybe_react: None,
            maybe_label: None,
            enabled: true,
            maybe_canvas_id: None,
            style: Style::new(),
        }
    }

    /// Set the DropDownList's reaction. It will be triggered upon selection of a list item.
    pub fn react(mut self, reaction: F) -> DropDownList<'a, F> {
        self.maybe_react = Some(reaction);
        self
    }

    /// If true, will allow user inputs.  If false, will disallow user inputs.
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
    }

    /// Set which Canvas to attach the Widget to. Note that you can also attach a widget to a
    /// Canvas by using the canvas placement `Positionable` methods.
    pub fn canvas(mut self, id: CanvasId) -> Self {
        self.maybe_canvas_id = Some(id);
        self
    }

    /// Set the maximum height of the DropDownList (before the scrollbar appears) as a number of
    /// items.
    pub fn max_visible_items(mut self, num: usize) -> Self {
        self.style.maybe_max_visible_height = Some(MaxHeight::Items(num));
        self
    }

    /// Set the maximum height of the DropDownList (before the scrollbar appears) as a scalar
    /// height.
    pub fn max_visible_height(mut self, height: f64) -> Self {
        self.style.maybe_max_visible_height = Some(MaxHeight::Scalar(height));
        self
    }

}


impl<'a, F> Widget for DropDownList<'a, F>
    where
        F: FnMut(&mut Option<Idx>, Idx, String),
{
    type State = State;
    type Style = Style;
    fn unique_kind(&self) -> &'static str { "DropDownList" }
    fn init_state(&self) -> State {
        State {
            menu_state: MenuState::Closed(Interaction::Normal),
            strings: Vec::new(),
            maybe_label: None,
            maybe_selected: None,
        }
    }
    fn style(&self) -> Style { self.style.clone() }
    fn canvas_id(&self) -> Option<CanvasId> { self.maybe_canvas_id }

    /// Capture the mouse if the menu was opened.
    fn capture_mouse(prev: &State, new: &State) -> bool {
        match (prev.menu_state, new.menu_state) {
            (MenuState::Closed(_), MenuState::Open(_, _)) => true,
            _ => false,
        }
    }

    /// Uncapture the mouse if the menu was closed.
    fn uncapture_mouse(prev: &State, new: &State) -> bool {
        match (prev.menu_state, new.menu_state) {
            (MenuState::Open(_, _), MenuState::Closed(_)) => true,
            _ => false,
        }
    }

    /// Update the state of the DropDownList.
    fn update<'b, C>(mut self,
                     prev_state: &widget::State<State>,
                     xy: Point,
                     dim: Dimensions,
                     input: UserInput<'b>,
                     style: &Style,
                     theme: &Theme,
                     _glyph_cache: &GlyphCache<C>) -> Option<State>
        where
            C: CharacterCache,
    {
        let widget::State { ref state, .. } = *prev_state;
        let maybe_mouse = input.maybe_mouse.map(|mouse| mouse.relative_to(xy));
        let frame = style.frame(theme);
        let num_strings = self.strings.len();

        // Check for a new interaction with the DropDownList.
        let (new_menu_state, is_over_elem) = match (self.enabled, maybe_mouse) {
            (false, _) | (true, None) => (MenuState::Closed(Interaction::Normal), None),
            (true, Some(mouse)) => {

                // Determine the maximum visible height before the scrollbar should appear.
                let bottom_win_y = (-input.window_dim[1]) / 2.0;
                const WINDOW_PADDING: Scalar = 20.0;
                let max_max_visible_height = xy[1] + dim[1] / 2.0 - bottom_win_y - WINDOW_PADDING;
                let max_visible_height = match style.max_visible_height(theme) {
                    Some(max_height) => {
                        let height = match max_height {
                            MaxHeight::Items(num) => (dim[1] - frame) * num as Scalar + frame,
                            MaxHeight::Scalar(height) => height,
                        };
                        ::utils::partial_min(height, max_max_visible_height)
                    },
                    None => max_max_visible_height,
                };

                // Determine whether or not there should be a scrollbar. If we do need one, and we
                // also had one last update, then carry the y_offset across from the last update.
                let item_h = dim[1] - frame;
                let total_h = item_h * (num_strings as Scalar) + frame;
                let maybe_scrollbar = {
                    if total_h <= max_visible_height { None } else {
                        let y_offset = match state.menu_state {
                            MenuState::Open(_, Some(ref scroll)) => scroll.y_offset,
                            _ => 0.0,
                        };
                        let max_offset = max_visible_height - (max_visible_height / total_h) * max_visible_height;
                        Some((y_offset, max_visible_height, max_offset))
                    }
                };
                let is_over_elem = is_over(mouse.xy, frame, dim, state.menu_state,
                                           num_strings, maybe_scrollbar);
                let new_menu_state = get_new_menu_state(is_over_elem, state.menu_state, dim[1],
                                                        mouse, maybe_scrollbar);
                (new_menu_state, is_over_elem)
            },
        };

        // Check that the selected index, if given, is not greater than the number of strings.
        let selected = self.selected.and_then(|idx| if idx < num_strings { Some(idx) }
                                                    else { None });

        // Call the `react` closure if mouse was released on one of the DropDownList items.
        if let Some(ref mut react) = self.maybe_react {
            if let (MenuState::Open(o_d_state, _), MenuState::Closed(c_d_state)) =
                (state.menu_state, new_menu_state) {
                if let (Some(Elem::Item(idx_a)), Interaction::Clicked(Elem::Item(idx_b)), Interaction::Normal) =
                    (is_over_elem, o_d_state, c_d_state) {
                    if idx_a == idx_b {
                        *self.selected = selected;
                        react(self.selected, idx_a, self.strings[idx_a].clone())
                    }
                }
            }
        }

        // Function for constructing a new DropDownList State.
        let new_state = || {
            State {
                menu_state: new_menu_state,
                maybe_label: self.maybe_label.as_ref().map(|label| label.to_string()),
                strings: self.strings.clone(),
                maybe_selected: *self.selected,
            }
        };

        // Check whether or not the state has changed since the previous update.
        let state_has_changed = state.menu_state != new_menu_state
            || &state.strings[..] != &(*self.strings)[..]
            || state.maybe_selected != *self.selected
            || state.maybe_label.as_ref().map(|string| &string[..]) != self.maybe_label;

        // Construct the new state if there was a change.
        if state_has_changed { Some(new_state()) } else { None }
    }


    /// Construct an Element from the given DropDownList State.
    fn draw<C>(new_state: &widget::State<State>,
               style: &Style,
               theme: &Theme,
               _glyph_cache: &GlyphCache<C>) -> Element
        where
            C: CharacterCache,
    {
        use elmesque::form::{collage, rect, text};
        use elmesque::text::Text;

        let widget::State { ref state, dim, xy, .. } = *new_state;

        // Retrieve the styling for the Element.
        let color = style.color(theme);
        let frame = style.frame(theme);
        let frame_color = style.frame_color(theme);
        let label_color = style.label_color(theme);
        let font_size = style.label_font_size(theme);
        let pad_dim = ::vecmath::vec2_sub(dim, [frame * 2.0; 2]);

        // Construct the DropDownList's Element.
        match state.menu_state {

            MenuState::Closed(draw_state) => {
                let string = match state.maybe_selected {
                    Some(idx) => state.strings[idx].clone(),
                    None => match state.maybe_label {
                        Some(ref label) => label.clone(),
                        None => state.strings[0].clone(),
                    },
                };
                let frame_form = rect(dim[0], dim[1]).filled(frame_color);
                let inner_form = rect(pad_dim[0], pad_dim[1]).filled(draw_state.color(color));
                let text_form = text(Text::from_string(string)
                                         .color(label_color)
                                         .height(font_size as f64));

                // Chain and shift the Forms into position.
                let form_chain = Some(frame_form).into_iter()
                    .chain(Some(inner_form))
                    .chain(Some(text_form))
                    .map(|form| form.shift(xy[0].floor(), xy[1].floor()));

                // Collect the Form's into a renderable Element.
                collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
            },

            MenuState::Open(interaction, maybe_scroll) => {

                // Determine the visible index range and the height of the first and last items.
                let item_h = dim[1] - frame;
                let total_h = item_h * state.strings.len() as f64 + frame;
                let (start_idx, end_idx, scroll_amt) = if let Some(scroll) = maybe_scroll {
                    let Scroll { max_visible_height, y_offset, max_offset } = scroll;
                    let num_items_that_fit = max_visible_height / item_h;
                    let num_items_scrolled_past = y_offset / item_h;
                    let start_idx = num_items_scrolled_past.floor() as usize;
                    let last_visible_item = num_items_scrolled_past + num_items_that_fit;
                    let end_idx = last_visible_item.floor() as usize;
                    let scroll_amt = (y_offset / max_offset) * (total_h - max_visible_height);
                    (start_idx, end_idx, scroll_amt)
                } else {
                    (0, state.strings.len() - 1, 0.0)
                };

                // Chain and shift the Forms into position.
                let visible_range = &state.strings[start_idx..end_idx + 1];
                let item_form_chain = visible_range.iter().enumerate().flat_map(|(i, string)| {
                    let i = i + start_idx;
                    let color = match state.maybe_selected {
                        None => match interaction {
                            Interaction::Highlighted(Elem::Item(idx)) =>
                                if i == idx { color.highlighted() } else { color },
                            Interaction::Clicked(Elem::Item(idx)) =>
                                if i == idx { color.clicked() } else { color },
                            _ => color,
                        },
                        Some(sel_idx) => {
                            if sel_idx == i { color.clicked() }
                            else {
                                match interaction {
                                    Interaction::Highlighted(Elem::Item(idx)) =>
                                        if i == idx { color.highlighted() } else { color },
                                    Interaction::Clicked(Elem::Item(idx)) =>
                                        if i == idx { color.clicked() } else { color },
                                    _ => color,
                                }
                            }
                        },
                    };
                    let frame_form = rect(dim[0], dim[1]).filled(frame_color);
                    let inner_form = rect(pad_dim[0], pad_dim[1]).filled(color);
                    let text_form = text(Text::from_string(string.clone())
                                             .color(label_color)
                                             .height(font_size as f64));
                    let shift_amt = -(i as f64 * dim[1] - i as f64 * frame).floor() + scroll_amt;
                    let forms = Some(frame_form).into_iter()
                        .chain(Some(inner_form))
                        .map(move |form| form.shift_y(shift_amt))
                        .chain(Some(text_form.shift_y(shift_amt.floor())));
                    forms
                });

                let maybe_scroll_forms = match interaction {
                    Interaction::Highlighted(_) | Interaction::Clicked(Elem::ScrollBar(_)) => {
                        Some(maybe_scroll.iter().flat_map(|scroll| {
                            let Scroll { y_offset, max_visible_height, .. } = *scroll;
                            let scroll_x = position::align_right_of(dim[0], SCROLLBAR_WIDTH);
                            let track_dim = [SCROLLBAR_WIDTH, max_visible_height];
                            let middle_y = item_h / 2.0 - max_visible_height / 2.0;
                            let track_xy = [scroll_x, middle_y];

                            let handle_h = (max_visible_height / total_h) * max_visible_height;
                            let handle_dim = [SCROLLBAR_WIDTH, handle_h];
                            let handle_top = position::align_top_of(track_dim[1], handle_h);
                            let handle_y = track_xy[1] + handle_top - y_offset;
                            let handle_xy = [scroll_x, handle_y];

                            let track_color = match interaction {
                                Interaction::Clicked(Elem::ScrollBar(ScrollBar::Track(_))) =>
                                    color.plain_contrast().plain_contrast().alpha(0.3).clicked(),
                                Interaction::Highlighted(Elem::ScrollBar(ScrollBar::Track(_))) =>
                                    color.plain_contrast().plain_contrast().alpha(0.3).highlighted(),
                                _ => color.plain_contrast().plain_contrast().alpha(0.3),
                            };
                            let handle_color = match interaction {
                                Interaction::Clicked(Elem::ScrollBar(ScrollBar::Handle(_))) =>
                                    color.plain_contrast().alpha(0.9).clicked(),
                                Interaction::Highlighted(Elem::ScrollBar(ScrollBar::Handle(_))) =>
                                    color.plain_contrast().alpha(0.75).highlighted(),
                                _ => color.plain_contrast().alpha(0.6),
                            };

                            let track_form = rect(track_dim[0], track_dim[1])
                                .filled(track_color)
                                .shift(track_xy[0], track_xy[1]);
                            let handle_form = rect(handle_dim[0], handle_dim[1])
                                .filled(handle_color)
                                .shift(handle_xy[0], handle_xy[1]);
                            Some(track_form).into_iter().chain(Some(handle_form))
                        }))
                    },
                    _ => None,
                }.into_iter().flat_map(|forms| forms);

                let form_chain = item_form_chain.chain(maybe_scroll_forms)
                    .map(|form| form.shift(xy[0].floor(), xy[1].floor()));

                // Collect the Form's into a renderable Element.
                let element = collage(dim[0] as i32, dim[1] as i32, form_chain.collect());

                match maybe_scroll {
                    Some(scroll) => {
                        let x = xy[0];
                        let y = xy[1] + dim[1] / 2.0 - scroll.max_visible_height / 2.0;
                        element.crop(x, y, dim[0], scroll.max_visible_height)
                    },
                    None => element,
                }
            },

        }

    }

}


impl Style {

    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_width: None,
            maybe_height: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
            maybe_max_visible_height: None,
        }
    }

    /// Get the width of the Widget.
    pub fn width(&self, theme: &Theme) -> Scalar {
        const DEFAULT_WIDTH: Scalar = 128.0;
        self.maybe_width.or(theme.maybe_drop_down_list.as_ref().map(|style| {
            style.maybe_width.unwrap_or(DEFAULT_WIDTH)
        })).unwrap_or(DEFAULT_WIDTH)
    }

    /// Get the height of the Widget.
    pub fn height(&self, theme: &Theme) -> Scalar {
        const DEFAULT_HEIGHT: Scalar = 32.0;
        self.maybe_height.or(theme.maybe_drop_down_list.as_ref().map(|style| {
            style.maybe_height.unwrap_or(DEFAULT_HEIGHT)
        })).unwrap_or(DEFAULT_HEIGHT)
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.maybe_drop_down_list.as_ref().map(|style| {
            style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.maybe_drop_down_list.as_ref().map(|style| {
            style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.maybe_drop_down_list.as_ref().map(|style| {
            style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label Color for an Element.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.maybe_drop_down_list.as_ref().map(|style| {
            style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn label_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.maybe_drop_down_list.as_ref().map(|style| {
            style.maybe_label_font_size.unwrap_or(theme.font_size_medium)
        })).unwrap_or(theme.font_size_medium)
    }

    /// Get the maximum visible height of the DropDownList.
    pub fn max_visible_height(&self, theme: &Theme) -> Option<MaxHeight> {
        if let Some(height) = self.maybe_max_visible_height { Some(height) }
        else if let Some(Some(height)) = theme.maybe_drop_down_list.as_ref()
            .map(|style| style.maybe_max_visible_height) { Some(height) }
        else { None }
    }

}


impl<'a, F> Colorable for DropDownList<'a, F> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

impl<'a, F> Frameable for DropDownList<'a, F> {
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, F> Labelable<'a> for DropDownList<'a, F> {
    fn label(mut self, text: &'a str) -> Self {
        self.maybe_label = Some(text);
        self
    }

    fn label_color(mut self, color: Color) -> Self {
        self.style.maybe_label_color = Some(color);
        self
    }

    fn label_font_size(mut self, size: FontSize) -> Self {
        self.style.maybe_label_font_size = Some(size);
        self
    }
}

impl<'a, F> Positionable for DropDownList<'a, F> {
    fn position(mut self, pos: Position) -> Self {
        self.pos = pos;
        self
    }
    fn get_position(&self) -> Position { self.pos }
    #[inline]
    fn horizontal_align(self, h_align: HorizontalAlign) -> Self {
        DropDownList { maybe_h_align: Some(h_align), ..self }
    }
    #[inline]
    fn vertical_align(self, v_align: VerticalAlign) -> Self {
        DropDownList { maybe_v_align: Some(v_align), ..self }
    }
    fn get_horizontal_align(&self, theme: &Theme) -> HorizontalAlign {
        self.maybe_h_align.unwrap_or(theme.align.horizontal)
    }
    fn get_vertical_align(&self, theme: &Theme) -> VerticalAlign {
        self.maybe_v_align.unwrap_or(theme.align.vertical)
    }
    fn depth(mut self, depth: Depth) -> Self {
        self.depth = depth;
        self
    }
    fn get_depth(&self) -> Depth { self.depth }
}

impl<'a, F> ::position::Sizeable for DropDownList<'a, F> {
    #[inline]
    fn width(mut self, w: Scalar) -> Self {
        self.style.maybe_width = Some(w);
        self
    }
    #[inline]
    fn height(mut self, h: Scalar) -> Self {
        self.style.maybe_height = Some(h);
        self
    }
    fn get_width<C: CharacterCache>(&self, theme: &Theme, _: &GlyphCache<C>) -> Scalar {
        self.style.width(theme)
    }
    fn get_height(&self, theme: &Theme) -> Scalar {
        self.style.height(theme)
    }
}

