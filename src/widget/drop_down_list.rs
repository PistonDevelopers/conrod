
use ::{
    Button,
    ButtonStyle,
    Canvas,
    CharacterCache,
    Color,
    Colorable,
    Element,
    FontSize,
    Frameable,
    GlyphCache,
    Labelable,
    NodeIndex,
    Positionable,
    Rect,
    Scalar,
    Sizeable,
    Theme,
};
use widget::{self, Widget};


/// The index of a selected item.
pub type Idx = usize;
/// The number of items in a list.
pub type Len = usize;

/// Displays a given `Vec<String>` as a selectable drop down menu. It's reaction is triggered upon
/// selection of a list item.
pub struct DropDownList<'a, F> {
    common: widget::CommonBuilder,
    strings: &'a mut Vec<String>,
    selected: &'a mut Option<Idx>,
    maybe_react: Option<F>,
    maybe_label: Option<&'a str>,
    style: Style,
    enabled: bool,
}

/// Styling for the DropDownList, necessary for constructing its renderable Element.
#[allow(missing_copy_implementations)]
#[derive(PartialEq, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Style {
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
    buttons: Vec<(NodeIndex, String)>,
    maybe_selected: Option<Idx>,
    maybe_canvas_idx: Option<NodeIndex>,
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
    Closed,
    Open,
}

impl<'a, F> DropDownList<'a, F> {

    /// Construct a new DropDownList.
    pub fn new(strings: &'a mut Vec<String>, selected: &'a mut Option<Idx>) -> DropDownList<'a, F> {
        DropDownList {
            common: widget::CommonBuilder::new(),
            strings: strings,
            selected: selected,
            maybe_react: None,
            maybe_label: None,
            enabled: true,
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


impl<'a, F> Widget for DropDownList<'a, F> where
    F: FnMut(&mut Option<Idx>, Idx, &str),
{
    type State = State;
    type Style = Style;
    fn common(&self) -> &widget::CommonBuilder { &self.common }
    fn common_mut(&mut self) -> &mut widget::CommonBuilder { &mut self.common }
    fn unique_kind(&self) -> &'static str { "DropDownList" }
    fn init_state(&self) -> State {
        State {
            menu_state: MenuState::Closed,
            buttons: Vec::new(),
            maybe_label: None,
            maybe_selected: None,
            maybe_canvas_idx: None,
        }
    }
    fn style(&self) -> Style { self.style.clone() }

    fn default_width<C: CharacterCache>(&self, theme: &Theme, _: &GlyphCache<C>) -> Scalar {
        const DEFAULT_WIDTH: Scalar = 128.0;
        theme.maybe_drop_down_list.as_ref().map(|default| {
            default.common.maybe_width.unwrap_or(DEFAULT_WIDTH)
        }).unwrap_or(DEFAULT_WIDTH)
    }

    fn default_height(&self, theme: &Theme) -> Scalar {
        const DEFAULT_HEIGHT: Scalar = 32.0;
        theme.maybe_drop_down_list.as_ref().map(|default| {
            default.common.maybe_height.unwrap_or(DEFAULT_HEIGHT)
        }).unwrap_or(DEFAULT_HEIGHT)
    }

    /// Update the state of the DropDownList.
    fn update<C: CharacterCache>(mut self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;

        let (global_mouse, window_dim) = {
            let input = ui.input();
            (input.global_mouse, input.window_dim)
        };
        let frame = style.frame(ui.theme());
        let num_strings = self.strings.len();

        let canvas_idx = state.view().maybe_canvas_idx
            .unwrap_or_else(|| ui.new_unique_node_index());

        // Check that the selected index, if given, is not greater than the number of strings.
        let selected = self.selected.and_then(|idx| if idx < num_strings { Some(idx) }
                                                    else { None });

        // If the number of buttons that we have in our previous state doesn't match the number of
        // strings we've just been given, we need to resize our buttons Vec.
        let num_buttons = state.view().buttons.len();
        let maybe_new_buttons = if num_buttons < num_strings {
            let new_buttons = (num_buttons..num_strings)
                .map(|i| (ui.new_unique_node_index(), self.strings[i].to_owned()));
            let total_new_buttons = state.view().buttons.iter()
                .map(|&(idx, ref string)| (idx, string.clone()))
                .chain(new_buttons);
            Some(total_new_buttons.collect())
        } else {
            None
        };

        // A function to simplify retrieval of the current list of buttons.
        fn get_buttons<'a>(maybe_new: &'a Option<Vec<(NodeIndex, String)>>,
                           state: &'a widget::State<State>) -> &'a [(NodeIndex, String)] {
            maybe_new.as_ref().map(|vec| &vec[..]).unwrap_or_else(|| &state.view().buttons[..])
        }

        // Determine the new menu state by checking whether or not any of our Button's reactions
        // are triggered.
        let new_menu_state = match state.view().menu_state {

            // If closed, we only want the button at the selected index to be drawn.
            MenuState::Closed => {
                let buttons = get_buttons(&maybe_new_buttons, state);

                // Get the button index and the label for the closed menu's button.
                let (button_idx, label) = selected
                    .map(|i| (buttons[i].0, &self.strings[i][..]))
                    .unwrap_or_else(|| (buttons[0].0, self.maybe_label.unwrap_or("")));

                // Use the pre-existing button widget to act as our button.
                let mut was_clicked = false;
                {
                    let mut button = Button::new()
                        .point(rect.xy())
                        .dim(rect.dim())
                        .label(label)
                        .parent(Some(idx))
                        .react(|| was_clicked = true);
                    let is_selected = false;
                    button.style = style.button_style(is_selected);
                    button.set(button_idx, &mut ui);
                }

                // If the closed menu was clicked, we want to open it.
                if was_clicked { MenuState::Open } else { MenuState::Closed }
            },

            // Otherwise if open, we want to set all the buttons that would be currently visible.
            MenuState::Open => {

                let (xy, dim) = rect.xy_dim();
                let max_visible_height = {
                    let bottom_win_y = (-window_dim[1]) / 2.0;
                    const WINDOW_PADDING: Scalar = 20.0;
                    let max = xy[1] + dim[1] / 2.0 - bottom_win_y - WINDOW_PADDING;
                    style.max_visible_height(ui.theme()).map(|max_height| {
                        let height = match max_height {
                            MaxHeight::Items(num) => (dim[1] - frame) * num as Scalar + frame,
                            MaxHeight::Scalar(height) => height,
                        };
                        ::utils::partial_min(height, max)
                    }).unwrap_or(max)
                };
                let canvas_dim = [dim[0], max_visible_height];
                let canvas_shift_y = ::position::align_top_of(dim[1], canvas_dim[1]);
                let canvas_xy = [xy[0], xy[1] + canvas_shift_y];
                let canvas_rect = Rect::from_xy_dim(canvas_xy, canvas_dim);
                Canvas::new()
                    .color(::color::black().alpha(0.0))
                    .frame_color(::color::black().alpha(0.0))
                    .dim([dim[0], max_visible_height])
                    .point(canvas_xy)
                    .parent(Some(idx))
                    .floating(true)
                    .vertical_scrolling(true)
                    .set(canvas_idx, &mut ui);

                let labels = self.strings.iter();
                let button_indices = state.view().buttons.iter().map(|&(idx, _)| idx);
                let xys = (0..num_strings).map(|i| [xy[0], xy[1] - i as f64 * (dim[1] - frame)]);
                let iter = labels.zip(button_indices).zip(xys).enumerate();
                let mut was_clicked = None;
                for (i, ((label, button_node_idx), button_xy)) in iter {
                    let mut button = Button::new()
                        .dim(dim)
                        .label(label)
                        .parent(Some(canvas_idx))
                        .point(button_xy)
                        .react(|| was_clicked = Some(i));
                    button.style = style.button_style(Some(i) == selected);
                    button.set(button_node_idx, &mut ui);
                }

                // If one of the buttons was clicked, we want to close the menu.
                if let Some(i) = was_clicked {

                    // If we were given some react function, we'll call it.
                    if let Some(ref mut react) = self.maybe_react {
                        *self.selected = selected;
                        react(self.selected, i, &self.strings[i])
                    }

                    MenuState::Closed
                // Otherwise if the mouse was released somewhere else we should close the menu.
                } else if global_mouse.left.was_just_pressed
                && !canvas_rect.is_over(global_mouse.xy) {
                    MenuState::Closed
                } else {
                    MenuState::Open
                }
            },

        };

        if let Some(new_buttons) = maybe_new_buttons {
            state.update(|state| state.buttons = new_buttons);
        }

        if state.view().menu_state != new_menu_state {
            state.update(|state| state.menu_state = new_menu_state);
        }

        if state.view().maybe_selected != *self.selected {
            state.update(|state| state.maybe_selected = *self.selected);
        }

        if state.view().maybe_canvas_idx != Some(canvas_idx) {
            state.update(|state| state.maybe_canvas_idx = Some(canvas_idx));
        }

        if state.view().maybe_label.as_ref().map(|label| &label[..]) != self.maybe_label {
            state.update(|state| {
                state.maybe_label = self.maybe_label.as_ref().map(|label| label.to_string());
            });
        }
    }

    /// Construct an Element from the given DropDownList State.
    fn draw<C: CharacterCache>(_args: widget::DrawArgs<Self, C>) -> Element {
        // We don't need to draw anything, as DropDownList is entirely composed of other widgets.
        ::elmesque::element::empty()
    }

}


impl Style {

    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
            maybe_max_visible_height: None,
        }
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.maybe_drop_down_list.as_ref().map(|default| {
            default.style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.maybe_drop_down_list.as_ref().map(|default| {
            default.style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.maybe_drop_down_list.as_ref().map(|default| {
            default.style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label Color for an Element.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.maybe_drop_down_list.as_ref().map(|default| {
            default.style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn label_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.maybe_drop_down_list.as_ref().map(|default| {
            default.style.maybe_label_font_size.unwrap_or(theme.font_size_medium)
        })).unwrap_or(theme.font_size_medium)
    }

    /// Get the maximum visible height of the DropDownList.
    pub fn max_visible_height(&self, theme: &Theme) -> Option<MaxHeight> {
        if let Some(height) = self.maybe_max_visible_height { Some(height) }
        else if let Some(Some(height)) = theme.maybe_drop_down_list.as_ref()
            .map(|default| default.style.maybe_max_visible_height) { Some(height) }
        else { None }
    }

    /// Style for a `Button` given this `Style`'s current state.
    pub fn button_style(&self, is_selected: bool) -> ButtonStyle {
        ButtonStyle {
            maybe_color: self.maybe_color.map(|c| if is_selected { c.highlighted() } else { c }),
            maybe_frame: self.maybe_frame,
            maybe_frame_color: self.maybe_frame_color,
            maybe_label_color: self.maybe_label_color,
            maybe_label_font_size: self.maybe_label_font_size,
        }
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

