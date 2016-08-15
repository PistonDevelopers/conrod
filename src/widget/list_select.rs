//! A wrapper around the `List` widget providing the ability to select one or more items.

use {
    color,
    event,
    Color,
    Colorable,
    FontSize,
    Labelable,
    Positionable,
    Scalar,
    Borderable,
};
use widget::{self, Widget};
use std;
use std::fmt::Display;

/// Displays a given Vec<T> where T: Display as a selectable List. Its reaction is triggered upon
/// selection of a list item.
pub struct ListSelect<'a, T: 'a> {
    entries: &'a [T],
    selected: &'a [bool],
    common: widget::CommonBuilder,
    multiple_selections: bool,
    style: Style,
}

widget_style!{
    /// Styling for the ListSelect, necessary for constructing its renderable Element.
    style Style {
        /// Size of text font in list
        - font_size: FontSize { theme.font_size_medium }
        /// The background color of the unselected entries.
        - color: Color { theme.shape_color }
        /// The background color of the selected entries.
        - selected_color: Color { theme.label_color }
        /// Color of the item text, when not selected.
        - text_color: Color { theme.shape_color }
        /// Color of the item text, when selected.
        - selected_text_color: Color { color::BLACK }
        /// Width of the border surrounding the widget
        - border: Scalar { theme.border_width }
        /// The color of the border.
        - border_color: Color { theme.border_color }
        /// Font size for the item labels.
        - label_font_size: FontSize { theme.font_size_medium }
        /// Auto hide the scrollbar or not
        - scrollbar_auto_hide: bool { true }
    }
}

/// Represents the state of the ListSelect.
#[derive(PartialEq, Clone, Debug)]
pub struct State {
    list_idx: widget::IndexSlot,
    /// Tracking index of last selected entry that has been pressed in order to
    /// perform multi selection when `SHIFT` or `ALT`(Mac) / 'CTRL'(Other OS) is held.
    last_selected_entry: Option<usize>,
}

/// The kind of events that the `ListSelect` may `react` to.
/// Provides tuple(s) of index in list and string representation of selection
#[derive(Clone, Debug)]
pub enum Event {
    /// A change in selection has occurred.
    ///
    /// TODO: This might need to be changed to Vec to keep track of the order in which items were
    /// selected. Not sure if this will be important to the user yet or not, but will likely wait
    /// for a use-case to arise.
    Selection(std::collections::HashSet<usize>),
    /// A button press occurred while the widget was capturing the mouse.
    Press(event::Press),
    /// A button release occurred while the widget was capturing the mouse.
    Release(event::Release),
    /// A click occurred while the widget was capturing the mouse.
    Click(event::Click),
    /// A double click occurred while the widget was capturing the mouse.
    DoubleClick(event::DoubleClick),
}

impl<'a, T> ListSelect<'a, T>
    where T: Display,
{

    /// Internal constructor
    fn new(entries: &'a [T], selected: &'a [bool], multi_sel: bool) -> Self {

        // Making sure the two vectors are same length, nicer than a panic
        if entries.len() != selected.len() {
            panic!("ERROR: entries:[T] and selected[bool] in ListSelect does not have same length!");
        }

        ListSelect {
            common: widget::CommonBuilder::new(),
            entries: entries,
            selected: selected,
            multiple_selections: multi_sel,
            style: Style::new(),
        }
    }

    /// Construct a new ListSelect, allowing one selected item at a time.
    /// Second parameter is a list reflecting which entries within the list are currently selected.
    pub fn single(entries: &'a [T], selected: &'a [bool]) -> Self {
        ListSelect::new(entries, selected, false)
    }

    /// Construct a new ListSelect, allowing multiple selected items.
    /// Second parameter is a list reflecting which entries within the list are currently selected.
    pub fn multiple(entries: &'a [T], selected: &'a [bool]) -> Self {
        ListSelect::new(entries, selected, true)
    }

    builder_methods!{
        pub font_size { style.font_size = Some(FontSize) }
        pub scrollbar_auto_hide { style.scrollbar_auto_hide = Some(bool) }
        pub selected_color { style.selected_color = Some(Color) }
        pub text_color { style.text_color = Some(Color) }
        pub selected_text_color { style.selected_text_color = Some(Color) }
    }

}

impl<'a, T> Widget for ListSelect<'a, T>
    where T: Display + 'a,
{
    type State = State;
    type Style = Style;
    type Event = Vec<Event>;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self) -> State {
        State {
            list_idx: widget::IndexSlot::new(),
            last_selected_entry:None,
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the ListSelect.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        use Sizeable;

        let widget::UpdateArgs { idx, mut state, style, mut ui, .. } = args;
        let ListSelect { entries, selected, multiple_selections, .. } = self;

        // Make sure that `last_selected_entry` refers to an actual selected value in the list.
        // If not push first selected item, if any.
        if let Some(ix) = state.last_selected_entry {
            if ix >= selected.len() || !selected[ix] {
                state.update(|state| state.last_selected_entry = None);
            }
        }
        if state.last_selected_entry.is_none() {
            for j in 0..selected.len() {
                if selected[j] {
                    state.update(|state| state.last_selected_entry = Some(j));
                    break;
                }
            }
        }

        let unsel_rect_color = style.color(&ui.theme);
        let unsel_text_color = style.text_color(&ui.theme);
        let sel_rect_color = style.selected_color(&ui.theme);
        let sel_text_color = style.selected_text_color(&ui.theme);

        let font_size = style.font_size(&ui.theme);
        let rect_h = font_size as Scalar * 2.0;

        // Given an index, find the first and last indices of the enclosing selection.
        //
        // This is used when expanding an existing selection with the shift key.
        fn selected_range(selected: &[bool], ix: usize) -> (usize, usize) {
            let mut first = ix;
            while selected[first] && first > 0 {
                first -= 1;
            }
            if !selected[first] {
                first += 1;
            }
            let mut last = ix;
            while selected[last] && last < selected.len() - 1 {
                last += 1;
            }
            if !selected[last] {
                last -= 1;
            }
            (first, last)
        }

        // Collect all relevant events that have occurred.
        let mut events = Vec::new();

        let list_idx = state.list_idx.get(&mut ui);
        let num_items = self.entries.len() as u32;
        let (mut list_items, list_scrollbar) = widget::List::new(num_items, rect_h)
            .scrollbar_on_top()
            .middle_of(idx)
            .wh_of(idx)
            .set(list_idx, &mut ui);

        while let Some(item) = list_items.next(&ui) {
            let i = item.i;

            // Check for any events that may have occurred to this widget.
            for widget_event in ui.widget_input(item.widget_idx).events() {
                use {event, input};

                match widget_event {

                    // Check if the entry has been `DoubleClick`ed.
                    event::Widget::DoubleClick(click) => {
                        if let input::MouseButton::Left = click.button {
                            events.push(Event::DoubleClick(click));
                        }
                    },

                    // Check if the entry has been `Click`ed.
                    event::Widget::Click(click) => {
                        let is_shift_down = click.modifiers.contains(input::keyboard::SHIFT);

                        // On Mac ALT is use for adding to selection, on Windows it's CTRL
                        let is_alt_down = click.modifiers.contains(input::keyboard::ALT) ||
                                          click.modifiers.contains(input::keyboard::CTRL);

                        let selection_event = match state.last_selected_entry {

                            // If there is already a currently selected item and shift is
                            // held, extend the selection to this one.
                            Some(idx) if is_shift_down && multiple_selections => {

                                // Default range
                                let mut start_idx_range = std::cmp::min(idx, i);
                                let mut end_idx_range = std::cmp::max(idx, i);

                                // Get continous block selected from last selected idx
                                // so the block can be extended up or down
                                let (first, last) = selected_range(selected, idx);

                                if i < first {
                                    start_idx_range = i;
                                    end_idx_range = last;
                                }
                                if i > last {
                                    start_idx_range = first;
                                    end_idx_range = i;
                                }

                                state.update(|state| state.last_selected_entry = Some(i));
                                let selection = (start_idx_range..end_idx_range + 1).collect();
                                Event::Selection(selection)
                            },

                            // If alt is down, additively select or deselect this item.
                            Some(_) | None if is_alt_down && multiple_selections => {
                                let mut selection: std::collections::HashSet<_> = selected.iter()
                                    .enumerate()
                                    .filter_map(|(i, &s)| if s { Some(i) } else { None })
                                    .collect();
                                if !selected[i] {
                                    selection.insert(i);
                                    state.update(|state| state.last_selected_entry = Some(i));
                                } else {
                                    selection.remove(&i);
                                };
                                Event::Selection(selection)
                            },

                            // Otherwise, no shift/ctrl/alt, select just this one
                            // Clear all others
                            _ => {
                                let selection = std::iter::once(i).collect();
                                state.update(|state| state.last_selected_entry = Some(i));
                                Event::Selection(selection)
                            },
                        };

                        events.push(Event::Click(click));
                        events.push(selection_event);
                    },

                    // Check for whether or not the item should be selected.
                    event::Widget::Press(press) => {
                        events.push(Event::Press(press));
                        match press.button {

                            // Keyboard check whether the selection has been bumped up or down.
                            event::Button::Keyboard(key) => {
                                if let Some(i) = state.last_selected_entry {
                                    let event = match key {

                                        // Bump the selection up the list.
                                        input::Key::Up => {
                                            let i = if i == 0 { 0 } else { i - 1 };
                                            state.update(|state| state.last_selected_entry = Some(i));
                                            let selection = std::iter::once(i).collect();
                                            Event::Selection(selection)
                                        },

                                        // Bump the selection down the list.
                                        input::Key::Down => {
                                            let last_idx = entries.len() - 1;
                                            let i = if i < last_idx { i + 1 } else { last_idx };
                                            state.update(|state| state.last_selected_entry = Some(i));
                                            let selection = std::iter::once(i).collect();
                                            Event::Selection(selection)
                                        },

                                        _ => continue,
                                    };

                                    events.push(event);
                                }
                            },

                            _ => (),
                        }
                    },

                    event::Widget::Release(release) => {
                        let event = Event::Release(release);
                        events.push(event);
                    },

                    _ => (),
                }
            }

            let label = format!("{}", self.entries[i].to_string());

            // Button colors, depending if selected or not.
            let (rect_color, text_color) =  match selected[i] {
                true => (sel_rect_color, sel_text_color),
                false => (unsel_rect_color, unsel_text_color),
            };

            let button = widget::Button::new()
                .label(&label)
                .label_color(text_color)
                .color(rect_color)
                .label_font_size(font_size)
                .border(0.0);
            item.set(button, &mut ui);
        }

        if let Some(scrollbar) = list_scrollbar {
            scrollbar.set(&mut ui);
        }

        events
    }

}

impl<'a, T> Colorable for ListSelect<'a, T> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, T> Borderable for ListSelect<'a, T> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}
