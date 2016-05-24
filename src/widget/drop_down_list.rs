
use {
    Backend,
    Button,
    ButtonStyle,
    Color,
    Colorable,
    FontSize,
    Frameable,
    IndexSlot,
    Labelable,
    NodeIndex,
    Positionable,
    Rect,
    Rectangle,
    Scalar,
    Sizeable,
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

/// Unique kind for the widget.
pub const KIND: widget::Kind = "DropDownList";

widget_style!{
    KIND;
    /// Styling for the DropDownList, necessary for constructing its renderable Element.
    style Style {
        /// Color of the widget.
        - color: Color { theme.shape_color }
        /// Width of the widget's frame.
        - frame: Scalar { theme.frame_width }
        /// Color of the widget's frame.
        - frame_color: Color { theme.frame_color }
        /// Color of the item labels.
        - label_color: Color { theme.label_color }
        /// Font size for the item labels.
        - label_font_size: FontSize { theme.font_size_medium }
        /// Maximum height of the Open menu before the scrollbar appears.
        - maybe_max_visible_height: Option<MaxHeight> { None }
    }
}

/// Represents the state of the DropDownList.
#[derive(PartialEq, Clone, Debug)]
pub struct State {
    menu_state: MenuState,
    buttons: Vec<(NodeIndex, String)>,
    maybe_selected: Option<Idx>,
    canvas_idx: IndexSlot,
}

/// Representations of the max height of the visible area of the DropDownList.
#[derive(PartialEq, Clone, Copy, Debug)]
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
    pub fn new(strings: &'a mut Vec<String>, selected: &'a mut Option<Idx>) -> Self {
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

    builder_methods!{
        pub react { maybe_react = Some(F) }
        pub enabled { enabled = bool }
    }

    /// Set the maximum height of the DropDownList (before the scrollbar appears) as a number of
    /// items.
    pub fn max_visible_items(mut self, num: usize) -> Self {
        self.style.maybe_max_visible_height = Some(Some(MaxHeight::Items(num)));
        self
    }

    /// Set the maximum height of the DropDownList (before the scrollbar appears) as a scalar
    /// height.
    pub fn max_visible_height(mut self, height: f64) -> Self {
        self.style.maybe_max_visible_height = Some(Some(MaxHeight::Scalar(height)));
        self
    }

}


impl<'a, F> Widget for DropDownList<'a, F>
    where F: FnMut(&mut Option<Idx>, Idx, &str),
{
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> &'static str {
        KIND
    }

    fn init_state(&self) -> State {
        State {
            menu_state: MenuState::Closed,
            buttons: Vec::new(),
            maybe_selected: None,
            canvas_idx: IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the DropDownList.
    fn update<B: Backend>(mut self, args: widget::UpdateArgs<Self, B>) {
        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;

        let frame = style.frame(ui.theme());
        let num_strings = self.strings.len();

        let canvas_idx = state.view().canvas_idx.get(&mut ui);

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

        if let Some(new_buttons) = maybe_new_buttons {
            state.update(|state| state.buttons = new_buttons);
        }

        // Act on the current menu state and determine what the next one will be.
        // new_menu_state is what we will be getting passed next frame
        let new_menu_state = match state.view().menu_state {
            // If closed, we only want the button at the selected index to be drawn.
            MenuState::Closed => {
                // Get the button index and the label for the closed menu's button.
                let buttons = &state.view().buttons;
                let (button_idx, label) = selected
                    .map(|i| (buttons[i].0, &self.strings[i][..]))
                    .unwrap_or_else(|| (buttons[0].0, self.maybe_label.unwrap_or("")));

                let mut was_clicked = false;
                {
                    // use the pre-existing Button widget
                    let mut button = Button::new()
                    .xy(rect.xy())
                    .wh(rect.dim())
                    .label(label)
                    .parent(idx)
                    .react(|| {was_clicked = true});
                    button.style = style.button_style(false);
                    button.set(button_idx, &mut ui);
                }

                // If the button was clicked, then open, otherwise stay closed
                if was_clicked { MenuState::Open } else { MenuState::Closed }
            },
            MenuState::Open => {
                // Otherwise if open, we want to set all the buttons that would be currently visible.
                let (xy, dim) = rect.xy_dim();
                let max_visible_height = {
                    let bottom_win_y = (-ui.window_dim()[1]) / 2.0;
                    const WINDOW_PADDING: Scalar = 20.0;
                    let max = xy[1] + dim[1] / 2.0 - bottom_win_y - WINDOW_PADDING;
                    style.maybe_max_visible_height(ui.theme()).map(|max_height| {
                        let height = match max_height {
                            MaxHeight::Items(num) => (dim[1] - frame) * num as Scalar + frame,
                            MaxHeight::Scalar(height) => height,
                        };
                        ::utils::partial_min(height, max)
                    }).unwrap_or(max)
                };
                let canvas_dim = [dim[0], max_visible_height];
                let canvas_shift_y = dim[1] / 2.0 - canvas_dim[1] / 2.0;
                let canvas_xy = [xy[0], xy[1] + canvas_shift_y];
                let canvas_rect = Rect::from_xy_dim(canvas_xy, canvas_dim);
                Rectangle::fill([dim[0], max_visible_height])
                    .graphics_for(idx)
                    .color(::color::BLACK.alpha(0.0))
                    .xy(canvas_xy)
                    .parent(idx)
                    .floating(true)
                    .scroll_kids_vertically()
                    .set(canvas_idx, &mut ui);

                let labels = self.strings.iter();
                let button_indices = state.view().buttons.iter().map(|&(idx, _)| idx);
                let xys = (0..num_strings).map(|i| [xy[0], xy[1] - i as f64 * (dim[1] - frame)]);
                let iter = labels.zip(button_indices).zip(xys).enumerate();
                let mut was_clicked = None;
                for (i, ((label, button_node_idx), button_xy)) in iter {
                    let mut button = Button::new()
                        .wh(dim)
                        .label(label)
                        .parent(canvas_idx)
                        .xy(button_xy)
                        .react(|| was_clicked = Some(i));
                    button.style = style.button_style(Some(i) == selected);
                    button.set(button_node_idx, &mut ui);
                }

                // Determine the new menu state
                if let Some(i) = was_clicked {
                    // If one of the buttons was clicked, we want to close the menu.
                    if let Some(ref mut react) = self.maybe_react {
                        // If we were given some react function, we'll call it.
                        *self.selected = selected;
                        react(self.selected, i, &self.strings[i]);
                    }
                    MenuState::Closed
                } else {
                    let mouse_pressed_elsewhere =
                        ui.global_input.current.mouse.buttons.pressed().next().is_some()
                        && !canvas_rect.is_over(ui.global_input.current.mouse.xy);

                    if mouse_pressed_elsewhere {
                        // If a mouse button was pressed somewhere else, close the menu.
                        MenuState::Closed
                    } else {
                        // Otherwise, leave the menu open.
                        MenuState::Open
                    }
                }
            }
        };

        if state.view().menu_state != new_menu_state {
            state.update(|state| state.menu_state = new_menu_state);
        }

        if state.view().maybe_selected != *self.selected {
            state.update(|state| state.maybe_selected = *self.selected);
        }
    }

}


impl Style {

    /// Style for a `Button` given this `Style`'s current state.
    pub fn button_style(&self, is_selected: bool) -> ButtonStyle {
        ButtonStyle {
            color: self.color.map(|c| if is_selected { c.highlighted() } else { c }),
            frame: self.frame,
            frame_color: self.frame_color,
            label_color: self.label_color,
            label_font_size: self.label_font_size,
        }
    }

}


impl<'a, F> Colorable for DropDownList<'a, F> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, F> Frameable for DropDownList<'a, F> {
    builder_methods!{
        frame { style.frame = Some(Scalar) }
        frame_color { style.frame_color = Some(Color) }
    }
}

impl<'a, F> Labelable<'a> for DropDownList<'a, F> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
