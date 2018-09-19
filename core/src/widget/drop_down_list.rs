//! The `DropDownList` and related items.

use {Color, Colorable, FontSize, Borderable, Labelable, Positionable, Sizeable};
use position::{self, Align, Scalar};
use text;
use utils;
use widget::{self, Widget};


/// The index of a selected item.
pub type Idx = usize;
/// The number of items in a list.
pub type Len = usize;

/// Displays a given `Vec<String>` as a selectable drop down menu.
///
/// It's reaction is triggered upon selection of a list item.
#[derive(WidgetCommon_)]
pub struct DropDownList<'a, T: 'a> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    items: &'a [T],
    selected: Option<Idx>,
    maybe_label: Option<&'a str>,
    style: Style,
    enabled: bool,
}

/// Styling for the DropDownList, necessary for constructing its renderable Element.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// Color of the widget.
    #[conrod(default = "theme.shape_color")]
    pub color: Option<Color>,
    /// Width of the widget's border.
    #[conrod(default = "theme.border_width")]
    pub border: Option<Scalar>,
    /// Color of the widget's border.
    #[conrod(default = "theme.border_color")]
    pub border_color: Option<Color>,
    /// Color of the item labels.
    #[conrod(default = "theme.label_color")]
    pub label_color: Option<Color>,
    /// Font size for the item labels.
    #[conrod(default = "theme.font_size_medium")]
    pub label_font_size: Option<FontSize>,
    /// The label's typographic alignment over the *x* axis.
    #[conrod(default = "text::Justify::Center")]
    pub label_justify: Option<text::Justify>,
    /// The label's position relative to its `Button` along the *x* axis.
    #[conrod(default = "position::Relative::Align(Align::Middle)")]
    pub label_x: Option<position::Relative>,
    /// The label's position relative to its `Button` along the *y* axis.
    #[conrod(default = "position::Relative::Align(Align::Middle)")]
    pub label_y: Option<position::Relative>,
    /// Maximum height of the Open menu before the scrollbar appears.
    #[conrod(default = "None")]
    pub maybe_max_visible_height: Option<Option<MaxHeight>>,
    /// The position of the scrollbar in the case that the list is scrollable.
    #[conrod(default = "None")]
    pub scrollbar_position: Option<Option<widget::list::ScrollbarPosition>>,
    /// The width of the scrollbar in the case that the list is scrollable.
    #[conrod(default = "None")]
    pub scrollbar_width: Option<Option<Scalar>>,
    /// The ID of the font used to display the labels.
    #[conrod(default = "theme.font_id")]
    pub label_font_id: Option<Option<text::font::Id>>,
}

widget_ids! {
    struct Ids {
        closed_menu,
        list,
    }
}

/// Represents the state of the DropDownList.
pub struct State {
    menu_state: MenuState,
    ids: Ids,
}

/// Representations of the max height of the visible area of the DropDownList.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MaxHeight {
    /// Specify the max height as a number of items.
    Items(usize),
    /// Specify the max height as an absolute scalar distance.
    Scalar(f64),
}

/// Whether the DropDownList is currently open or closed.
#[derive(PartialEq, Clone, Copy, Debug)]
enum MenuState {
    Closed,
    Open,
}

impl<'a, T> DropDownList<'a, T> {

    /// Construct a new DropDownList.
    pub fn new(items: &'a [T], selected: Option<Idx>) -> Self {
        DropDownList {
            common: widget::CommonBuilder::default(),
            items: items,
            selected: selected,
            maybe_label: None,
            enabled: true,
            style: Style::default(),
        }
    }

    builder_methods!{
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
        self.style.scrollbar_position = Some(Some(widget::list::ScrollbarPosition::NextTo));
        self
    }

    /// Specifies that the list should be scrollable and should provide a `Scrollbar` that hovers
    /// above the right edge of the items and automatically hides when the user is not scrolling.
    pub fn scrollbar_on_top(mut self) -> Self {
        self.style.scrollbar_position = Some(Some(widget::list::ScrollbarPosition::OnTop));
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

    /// Specify the font used for displaying the label.
    pub fn label_font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.label_font_id = Some(Some(font_id));
        self
    }

    /// Align the labels to the left of their `Button`s' surface.
    pub fn left_justify_label(mut self) -> Self {
        self.style.label_justify = Some(text::Justify::Left);
        self
    }

    /// Align the labels to the right of their `Button`s' surface.
    pub fn right_justify_label(mut self) -> Self {
        self.style.label_justify = Some(text::Justify::Right);
        self
    }

    /// Center the labels to the their `Button`s' surface.
    pub fn center_justify_label(mut self) -> Self {
        self.style.label_justify = Some(text::Justify::Center);
        self
    }

    /// Specify the label's position relatively to `Button` along the *x* axis.
    pub fn label_x(mut self, x: position::Relative) -> Self {
        self.style.label_x = Some(x);
        self
    }

    /// Specify the label's position relatively to `Button` along the *y* axis.
    pub fn label_y(mut self, y: position::Relative) -> Self {
        self.style.label_y = Some(y);
        self
    }

}


impl<'a, T> Widget for DropDownList<'a, T>
    where T: AsRef<str>,
{
    type State = State;
    type Style = Style;
    type Event = Option<Idx>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            menu_state: MenuState::Closed,
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the DropDownList.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, rect, style, ui, .. } = args;

        let num_items = self.items.len();

        // Check that the selected index, if given, is not greater than the number of items.
        let selected = self.selected.and_then(|idx| if idx < num_items { Some(idx) }
                                                    else { None });

        // Track whether or not a list item was clicked.
        let mut clicked_item = None;

        // Act on the current menu state and determine what the next one will be.
        // new_menu_state is what we will be getting passed next frame
        let new_menu_state = match state.menu_state {

            // If closed, we only want the button at the selected index to be drawn.
            MenuState::Closed => {
                // Get the button index and the label for the closed menu's button.
                let label = selected
                    .map(|i| self.items[i].as_ref())
                    .unwrap_or_else(|| self.maybe_label.unwrap_or(""));

                let was_clicked = {
                    // use the pre-existing Button widget
                    let mut button = widget::Button::new()
                        .xy(rect.xy())
                        .wh(rect.dim())
                        .label(label)
                        .parent(id);
                    button.style = style.button_style(false);
                    button.set(state.ids.closed_menu, ui).was_clicked()
                };

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
                        utils::partial_min(height, max)
                    }).unwrap_or(max)
                };

                // The list of buttons.
                let num_items = self.items.len();
                let item_h = h;
                let list_h = max_visible_height.min(num_items as Scalar * item_h);
                let scrollbar_color = style.border_color(&ui.theme);
                let scrollbar_position = style.scrollbar_position(&ui.theme);
                let scrollbar_width = style.scrollbar_width(&ui.theme)
                    .unwrap_or_else(|| {
                        ui.theme.widget_style::<widget::scrollbar::Style>()
                            .and_then(|style| style.style.thickness)
                            .unwrap_or(10.0)
                    });

                let (mut events, scrollbar) = widget::ListSelect::single(num_items)
                    .flow_down()
                    .item_size(item_h)
                    .w_h(w, list_h)
                    .and(|ls| match scrollbar_position {
                        Some(widget::list::ScrollbarPosition::NextTo) => ls.scrollbar_next_to(),
                        Some(widget::list::ScrollbarPosition::OnTop) => ls.scrollbar_on_top(),
                        None => ls,
                    })
                    .scrollbar_color(scrollbar_color)
                    .scrollbar_thickness(scrollbar_width)
                    .mid_top_of(id)
                    .floating(true)
                    .set(state.ids.list, ui);

                while let Some(event) = events.next(ui, |i| Some(i) == selected) {
                    use widget::list_select::Event;
                    match event {

                        // Instantiate a `Button` for each item.
                        Event::Item(item) => {
                            let i = item.i;
                            let label = self.items[i].as_ref();
                            let mut button = widget::Button::new().label(label);
                            button.style = style.button_style(Some(i) == selected);
                            item.set(button, ui);
                        },

                        // The selection changed.
                        Event::Selection(ix) => clicked_item = Some(ix),

                        _ => (),
                    }
                }

                // Instantiate the `Scrollbar` if there is one.
                if let Some(scrollbar) = scrollbar {
                    scrollbar.set(ui);
                }

                // Close the menu if the mouse is pressed and the currently pressed widget is
                // not any of the drop down list's children.
                let should_close = clicked_item.is_some() ||
                    clicked_item.is_none()
                    && ui.global_input().current.mouse.buttons.pressed().next().is_some()
                    && match ui.global_input().current.widget_capturing_mouse {
                        None => true,
                        Some(capturing) => !ui.widget_graph()
                            .does_recursive_depth_edge_exist(id, capturing),
                    };

                // If a mouse button was pressed somewhere else, close the menu.
                //
                // Otherwise, leave the menu open.
                if should_close { MenuState::Closed } else { MenuState::Open }
            }
        };

        if state.menu_state != new_menu_state {
            state.update(|state| state.menu_state = new_menu_state);
        }

        clicked_item
    }

}


impl Style {

    /// Style for a `Button` given this `Style`'s current state.
    pub fn button_style(&self, is_selected: bool) -> widget::button::Style {
        widget::button::Style {
            color: self.color.map(|c| if is_selected { c.highlighted() } else { c }),
            border: self.border,
            border_color: self.border_color,
            label_color: self.label_color,
            label_font_size: self.label_font_size,
            label_justify: self.label_justify,
            label_x: self.label_x,
            label_y: self.label_y,
            label_font_id: self.label_font_id,
        }
    }

}


impl<'a, T> Colorable for DropDownList<'a, T> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, T> Borderable for DropDownList<'a, T> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a, T> Labelable<'a> for DropDownList<'a, T> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
