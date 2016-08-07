use {
    Button,
    ButtonStyle,
    Color,
    Colorable,
    FontSize,
    Borderable,
    IndexSlot,
    Labelable,
    List,
    NodeIndex,
    Positionable,
    Scalar,
    ScrollbarStyle,
    Sizeable,
};
use super::list::ScrollbarPosition;
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

widget_style!{
    /// Styling for the DropDownList, necessary for constructing its renderable Element.
    style Style {
        /// Color of the widget.
        - color: Color { theme.shape_color }
        /// Width of the widget's border.
        - border: Scalar { theme.border_width }
        /// Color of the widget's border.
        - border_color: Color { theme.border_color }
        /// Color of the item labels.
        - label_color: Color { theme.label_color }
        /// Font size for the item labels.
        - label_font_size: FontSize { theme.font_size_medium }
        /// Maximum height of the Open menu before the scrollbar appears.
        - maybe_max_visible_height: Option<MaxHeight> { None }
        /// The position of the scrollbar in the case that the list is scrollable.
        - scrollbar_position: Option<ScrollbarPosition> { None }
        /// The width of the scrollbar in the case that the list is scrollable.
        - scrollbar_width: Option<Scalar> { None }
    }
}

/// Represents the state of the DropDownList.
#[derive(PartialEq, Clone, Debug)]
pub struct State {
    menu_state: MenuState,
    buttons: Vec<(NodeIndex, String)>,
    maybe_selected: Option<Idx>,
    closed_menu: IndexSlot,
    list_idx: IndexSlot,
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

    /// Specifies that the list should be scrollable and should provide a `Scrollbar` to the right
    /// of the items.
    pub fn scrollbar_next_to(mut self) -> Self {
        self.style.scrollbar_position = Some(Some(ScrollbarPosition::NextTo));
        self
    }

    /// Specifies that the list should be scrollable and should provide a `Scrollbar` that hovers
    /// above the right edge of the items and automatically hides when the user is not scrolling.
    pub fn scrollbar_on_top(mut self) -> Self {
        self.style.scrollbar_position = Some(Some(ScrollbarPosition::OnTop));
        self
    }

    /// Even in the case that the list is scrollable, do not display a `Scrollbar`.
    pub fn no_scrollbar(mut self) -> Self {
        self.style.scrollbar_position = Some(None);
        self
    }

    /// Specify the width of the scrollbar.
    pub fn scrollbar_width(mut self, w: Scalar) -> Self {
        self.style.scrollbar_width = Some(Some(w));
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

    fn init_state(&self) -> State {
        State {
            menu_state: MenuState::Closed,
            buttons: Vec::new(),
            maybe_selected: None,
            list_idx: IndexSlot::new(),
            closed_menu: IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the DropDownList.
    fn update(mut self, args: widget::UpdateArgs<Self>) {
        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;

        let num_strings = self.strings.len();

        // Check that the selected index, if given, is not greater than the number of strings.
        let selected = self.selected.and_then(|idx| if idx < num_strings { Some(idx) }
                                                    else { None });

        // If the number of buttons that we have in our previous state doesn't match the number of
        // strings we've just been given, we need to resize our buttons Vec.
        let num_buttons = state.buttons.len();
        let maybe_new_buttons = if num_buttons < num_strings {
            let new_buttons = (num_buttons..num_strings)
                .map(|i| (ui.new_unique_node_index(), self.strings[i].to_owned()));
            let total_new_buttons = state.buttons.iter()
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
        let new_menu_state = match state.menu_state {
            // If closed, we only want the button at the selected index to be drawn.
            MenuState::Closed => {
                // Get the button index and the label for the closed menu's button.
                let buttons = &state.buttons;
                let (button_idx, label) = selected
                    .map(|i| (buttons[i].0, &self.strings[i][..]))
                    .unwrap_or_else(|| {
                        let closed_menu_idx = state.closed_menu.get(&mut ui);
                        let label = self.maybe_label.unwrap_or("");
                        (closed_menu_idx, label)
                    });

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
                let (_, y, w, h) = rect.x_y_w_h();
                let max_visible_height = {
                    let bottom_win_y = (-ui.window_dim()[1]) / 2.0;
                    const WINDOW_PADDING: Scalar = 20.0;
                    let max = y + h / 2.0 - bottom_win_y - WINDOW_PADDING;
                    style.maybe_max_visible_height(ui.theme()).map(|max_height| {
                        let height = match max_height {
                            MaxHeight::Items(num) => h * num as Scalar,
                            MaxHeight::Scalar(height) => height,
                        };
                        ::utils::partial_min(height, max)
                    }).unwrap_or(max)
                };

                // The list of buttons.
                let mut was_clicked = None;
                let num_strings = self.strings.len();
                let item_h = h;
                let list_h = max_visible_height.min(num_strings as Scalar * item_h);
                let list_idx = state.list_idx.get(&mut ui);
                let scrollbar_color = style.border_color(&ui.theme);
                let scrollbar_position = style.scrollbar_position(&ui.theme);
                let scrollbar_width = style.scrollbar_width(&ui.theme)
                    .unwrap_or_else(|| {
                        ui.theme.widget_style::<ScrollbarStyle>()
                            .and_then(|style| style.style.thickness)
                            .unwrap_or(10.0)
                    });
                List::new(num_strings as u32, item_h)
                    .w_h(w, list_h)
                    .and(|ls| match scrollbar_position {
                        Some(ScrollbarPosition::NextTo) => ls.scrollbar_next_to(),
                        Some(ScrollbarPosition::OnTop) => ls.scrollbar_on_top(),
                        None => ls,
                    })
                    .scrollbar_color(scrollbar_color)
                    .scrollbar_width(scrollbar_width)
                    .mid_top_of(idx)
                    .floating(true)
                    .item(|item| {

                        // Instiate the `Button` for each item.
                        let i = item.i;
                        let label = match self.strings.get(i) {
                            Some(label) => label,
                            None => return,
                        };
                        let mut button = Button::new()
                            .label(label)
                            .react(|| was_clicked = Some(i));
                        button.style = style.button_style(Some(i) == selected);
                        item.set(button);

                    })
                    .set(list_idx, &mut ui);

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

                    // Close the menu if the mouse is pressed and the currently pressed widget is
                    // not any of the drop down list's children.
                    let should_close =
                        ui.global_input.current.mouse.buttons.pressed().next().is_some()
                        && match ui.global_input.current.widget_capturing_mouse {
                            None => true,
                            Some(capturing) => !ui.widget_graph()
                                .does_recursive_depth_edge_exist(idx, capturing),
                        };

                    if should_close {
                        // If a mouse button was pressed somewhere else, close the menu.
                        MenuState::Closed
                    } else {
                        // Otherwise, leave the menu open.
                        MenuState::Open
                    }
                }
            }
        };

        if state.menu_state != new_menu_state {
            state.update(|state| state.menu_state = new_menu_state);
        }

        if state.maybe_selected != *self.selected {
            state.update(|state| state.maybe_selected = *self.selected);
        }
    }

}


impl Style {

    /// Style for a `Button` given this `Style`'s current state.
    pub fn button_style(&self, is_selected: bool) -> ButtonStyle {
        ButtonStyle {
            color: self.color.map(|c| if is_selected { c.highlighted() } else { c }),
            border: self.border,
            border_color: self.border_color,
            label_color: self.label_color,
            label_font_size: self.label_font_size,
        }
    }

}


impl<'a, F> Colorable for DropDownList<'a, F> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, F> Borderable for DropDownList<'a, F> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a, F> Labelable<'a> for DropDownList<'a, F> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
