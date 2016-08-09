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
pub struct ListSelect<'a, T: 'a, F> {
    entries: &'a [T],
    selected: &'a mut [bool],
    common: widget::CommonBuilder,
    maybe_react: Option<F>,
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
    /// If a single enty is selected, return index in list and string representation of item.
    SelectEntry(usize, String),
    /// If several entries are elected, return indices in list and string representations of items.
    SelectEntries(Vec<(usize, String)>),
    /// If entry selected by doubleclick, return index in list and string representation of item.
    DoubleClick(usize, String),
    /// If one or more entries are elected by keyboard, return indices in list and string
    /// representations of items as well as specific key event.
    KeyPress(Vec<(usize, String)>, event::KeyPress),
}

impl<'a, T, F> ListSelect<'a, T, F>
    where F: FnMut(Event),
          T: Display,
{

    /// Internal constructor
    fn new(entries: &'a [T], selected: &'a mut [bool], multi_sel: bool) -> Self {

        // Making sure the two vectors are same length, nicer than a panic
        if entries.len() != selected.len() {
            panic!("ERROR: entries:[T] and selected[bool] in ListSelect does not have same length!");
        }

        ListSelect {
            common: widget::CommonBuilder::new(),
            entries: entries,
            selected: selected,
            maybe_react: None,
            multiple_selections: multi_sel,
            style: Style::new(),
        }
    }

    /// Construct a new ListSelect, allowing one selected item at a time.
    /// Second parameter is a list reflecting which entries within the list are currently selected.
    pub fn single(entries: &'a [T], selected: &'a mut [bool]) -> Self {
        ListSelect::new(entries, selected, false)
    }

    /// Construct a new ListSelect, allowing multiple selected items.
    /// Second parameter is a list reflecting which entries within the list are currently selected.
    pub fn multiple(entries: &'a [T], selected: &'a mut [bool]) -> Self {
        ListSelect::new(entries, selected, true)
    }

    builder_methods!{
        pub font_size { style.font_size = Some(FontSize) }
        pub scrollbar_auto_hide { style.scrollbar_auto_hide = Some(bool) }
        pub react { maybe_react = Some(F) }
        pub selected_color { style.selected_color = Some(Color) }
        pub text_color { style.text_color = Some(Color) }
        pub selected_text_color { style.selected_text_color = Some(Color) }
    }

}

impl<'a, T, F> Widget for ListSelect<'a, T, F>
    where F: FnMut(Event),
          T: Display + 'a,
{
    type State = State;
    type Style = Style;
    type Event = ();

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
    fn update(mut self, args: widget::UpdateArgs<Self>) -> Self::Event {
        use Sizeable;

        let widget::UpdateArgs { idx, mut state, style, mut ui, .. } = args;
        let ListSelect { entries, selected, multiple_selections, mut maybe_react, .. } = self;

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

        // If set, a widget event was generated. Set in inner closure
        let mut clicked_item = None;

        let num_items = self.entries.len() as u32;
        let (mut list_items, list_scrollbar) = widget::List::new(num_items, rect_h)
            .scrollbar_on_top()
            .middle_of(idx)
            .wh_of(idx)
            .set(list_idx, &mut ui);

        while let Some(item) = list_items.next(&ui) {
            let i = item.i;
            let label = format!("{}", self.entries[i].to_string());

            // Button colors, depending if selected or not.
            let (rect_color, text_color) =  match selected[i] {
                true => (sel_rect_color, sel_text_color),
                false => (unsel_rect_color, unsel_text_color),
            };

            let item_idx = item.widget_idx;
            let button = widget::Button::new()
                .label(&label)
                .label_color(text_color)
                .color(rect_color)
                .label_font_size(font_size)
                .border(0.0);
            if item.set(button, &mut ui).was_clicked() {
                clicked_item = Some((i, item_idx));
            }
        }

        if let Some(scrollbar) = list_scrollbar {
            scrollbar.set(&mut ui);
        }

        // Clear selection status for all items.
        fn clear_all_selected(last_selected_entry: &mut Option<usize>, selected: &mut [bool]) {
            *last_selected_entry = None;
            for i in 0..selected.len() {
                selected[i] = false;
            }
        }

        // Returns a Vec of tuples (index, String) to hand over to React closure
        let selected_entries = |entries: &[T], selected: &[bool]| -> Vec<(usize, String)> {
            let mut selected_entries = Vec::new();

            for i in 0..selected.len() {
                if selected[i] {
                    selected_entries.push((i as usize, entries[i].to_string()));
                }
            }

            selected_entries
        };

        // Given an index, find the first and last indices of the enclosing selection.
        // Used to expand existing selection with shift key.
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

        // If no list items were clicked, we're done.
        let (i, item_idx) = match clicked_item {
            Some(item) => item,
            None => return,
        };

        // Otherwise, check for changes in selection.
        for widget_event in ui.widget_input(item_idx).events() {
            use {event, input};

            match widget_event {

                // Check if the entry has been `DoubleClick`ed.
                event::Widget::DoubleClick(click) => {
                    if let input::MouseButton::Left = click.button {
                        // Deselect all others.
                        state.update(|state| {
                            clear_all_selected(&mut state.last_selected_entry, selected);
                            selected[i] = true;
                            state.last_selected_entry = Some(i);
                        });
                        if let Some(ref mut react) = maybe_react {
                            react(Event::DoubleClick(i, entries[i].to_string()));
                        }
                    }
                },

                // Check if the entry has been `Click`ed.
                event::Widget::Click(click) => {
                    let is_shift_down = click.modifiers.contains(input::keyboard::SHIFT);

                    // On Mac ALT is use for adding to selection, on Windows it's CTRL
                    let is_alt_down = click.modifiers.contains(input::keyboard::ALT) ||
                                      click.modifiers.contains(input::keyboard::CTRL);

                    match state.last_selected_entry {

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

                            // generate react list inline rather than using get_selected_as_tuples()
                            let mut temp = Vec::new();
                            state.update(|state| {

                                clear_all_selected(&mut state.last_selected_entry, selected);
                                state.last_selected_entry = Some(i);

                                // Set selected only for the range. Clearing other blocks.
                                for j in 0..selected.len() {
                                    if start_idx_range <= j && j <= end_idx_range {
                                        selected[j] = true;
                                        temp.push((j as usize, entries[j].to_string()));
                                    } else {
                                        selected[j] = false;
                                    }
                                }
                            });

                            if let Some(ref mut react) = maybe_react {
                                react(Event::SelectEntries(temp))
                            }
                        },

                        // If alt is down, additively select or deselect this item.
                        Some(_) | None if is_alt_down && multiple_selections => {
                            state.update(|state| {
                                let new_is_selected = !selected[i];
                                selected[i] = new_is_selected;
                                if new_is_selected {
                                    state.last_selected_entry = Some(i);
                                }
                            });

                            let mut selected_entries = selected_entries(entries, selected);
                            let num_entries_selected = selected_entries.len();

                            if  num_entries_selected > 0 {
                                if let Some(ref mut react) = maybe_react {
                                    if num_entries_selected > 1 {
                                        react(Event::SelectEntries(selected_entries));
                                    } else {
                                        // Safe to unwrap here, as list len is 1
                                        let (ix, st) = selected_entries.pop().unwrap();
                                        react(Event::SelectEntry(ix, st));
                                    }
                                }
                            }
                        },

                        // Otherwise, no shift/ctrl/alt, select just this one
                        // Clear all others
                        _ => {
                            // Deselect all others.
                            state.update(|state| {
                                clear_all_selected(&mut state.last_selected_entry, selected);
                            });

                            // Select the current item.
                            selected[i] = true;
                            state.update(|state| state.last_selected_entry = Some(i));
                            if let Some(ref mut react) = maybe_react {
                                react(Event::SelectEntry(i, entries[i].to_string()));
                            }
                        },
                    }
                },

                // Check for whether or not the item should be selected.
                event::Widget::Press(press) => match press.button {

                    // Keyboard check whether the selection has been bumped up or down.
                    event::Button::Keyboard(key) => {
                        if let Some(i) = state.last_selected_entry {
                            match key {

                                // Bump the selection up the list.
                                input::Key::Up => state.update(|state| {
                                    // Clear old selected entries.
                                    clear_all_selected(&mut state.last_selected_entry, selected);

                                    let i = if i == 0 { 0 } else { i - 1 };
                                    selected[i] = true;
                                    state.last_selected_entry = Some(i);

                                    if let Some(ref mut react) = maybe_react {
                                        react(Event::SelectEntry(i, entries[i].to_string()));
                                    }
                                }),

                                // Bump the selection down the list.
                                input::Key::Down => state.update(|state| {
                                    // Clear old selected entries.
                                    clear_all_selected(&mut state.last_selected_entry, selected);

                                    let last_idx = entries.len() - 1;
                                    let i = if i < last_idx { i + 1 } else { last_idx };
                                    selected[i] = true;
                                    state.last_selected_entry = Some(i);

                                    if let Some(ref mut react) = maybe_react {
                                        react(Event::SelectEntry(i, entries[i].to_string()));
                                    }
                                }),

                                _ => (),
                            }

                            // For any other pressed keys, yield an event along
                            // with all the paths of all selected entries.
                            let selected_entries = selected_entries(entries, selected);
                            if let Some(ref mut react) = maybe_react {
                                let key_press = event::KeyPress {
                                    key: key,
                                    modifiers: press.modifiers,
                                };
                                react(Event::KeyPress(selected_entries, key_press));
                            }
                        }
                    },

                    _ => (),
                },

                _ => (),
            }
        }

    }

}

impl<'a, T, F> Colorable for ListSelect<'a, T, F> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, T, F> Borderable for ListSelect<'a, T, F> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}
