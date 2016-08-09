//! A wrapper around the `List` widget providing the ability to select one or more items.

use {
    color,
    event,
    Color,
    Colorable,
    FontSize,
    Labelable,
    NodeIndex,
    Positionable,
    Scalar,
    Borderable,
};
use widget::{self, Widget};
use std;
use std::fmt::Display;

/// Displays a given Vec<T> where T: Display as a selectable List. Its reaction is triggered upon
/// selection of a list item.
pub struct ListSelect<'a, T, F> where F: FnMut(Event), T: Display+'a {
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

impl<'a, T, F> ListSelect<'a, T, F> where F: FnMut(Event), T: Display {

    /// Internal constructor
    fn new(entries: &'a [T], selected: &'a mut [bool], multi_sel: bool) -> Self {

        // Making sure the two vectors are same length, nicer than a panic
        if entries.len()!=selected.len() {
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

    /// Clear selection status for all items
    fn clear_all_selected(&mut self, last_selected_entry: &mut Option<usize>) {
        *last_selected_entry = None;
        for i in 0..self.selected.len() {
            self.selected[i] = false;
        }
    }

    /// Make sure that last selected item refers to an actual selected value in list
    /// If not push first selected item, if any.
    fn validate_last_selected(&mut self, last_selected_entry: &mut Option<usize>) {

        if let Some(ix) = *last_selected_entry {
            if ix<self.selected.len() && !self.selected[ix] {
                *last_selected_entry = None;
            } else {
                *last_selected_entry = None;
            }
        }
        if *last_selected_entry == None {
            for j in 0..self.selected.len() {
                if self.selected[j] {
                    *last_selected_entry = Some(j);
                    break;
                }
            }
        }
    }

    /// Returns a Vec of tuples (index, String) to hand over to React closure
    fn get_selected_as_tuples(&self) -> Vec<(usize, String)> {
        let mut selected = Vec::new();

        for i in 0..self.selected.len() {
            if self.selected[i] {
                selected.push((i as usize, self.entries[i].to_string()));
            }
        }
        selected
    }

    /// given an index, find first and last of enclosing selection.
    /// Used to expand existing block with shift key.
    fn get_selected_range(&self, ix: usize) -> (usize, usize) {
        let mut first = ix;
        while self.selected[first] && first>0 {
            first -= 1;
        }
        if !self.selected[first] {
            first +=1;
        }
        let mut last = ix;
        while self.selected[last] && last<self.selected.len()-1 {
            last+=1;
        }
        if !self.selected[last] {
            last -=1;
        }
        (first, last)
    }

}

impl<'a, T, F> Widget for ListSelect<'a, T, F>
    where F: FnMut(Event), T: Display+'a
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
            list_idx: widget::IndexSlot::new(),
            last_selected_entry:None,
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the ListSelect.
    fn update(mut self, args: widget::UpdateArgs<Self>) {

        let widget::UpdateArgs { idx, mut state, style, mut ui, .. } = args;

        let list_idx = state.list_idx.get(&mut ui);

        {
            state.update(|state| {
                self.validate_last_selected(&mut state.last_selected_entry);
            });
            use position::Sizeable;

            let unsel_rect_color = style.color(&ui.theme);
            let unsel_text_color = style.text_color(&ui.theme);
            let sel_rect_color   = style.selected_color(&ui.theme);
            let sel_text_color   = style.selected_text_color(&ui.theme);

            let font_size = style.font_size(&ui.theme);
            let rect_h = font_size as Scalar * 2.0;

            // For storing NodeIndex for the buttons in the list, for retrieving events
            let mut entry_idx_list = vec![NodeIndex::new(0 as usize); self.entries.len()];

            // If set, a widget event was generated. Set in inner closure
            let mut list_index_event: Option<usize> = None;

            let mut txt_col = unsel_text_color;
            let mut rect_col = unsel_rect_color;

            widget::List::new(self.entries.len() as u32, rect_h)
                .scrollbar_on_top()
                .middle_of(idx)
                .wh_of(idx)
                .item(|item| {
                    let i = item.i;
                    let label = format!("{}", self.entries[i].to_string());

                    // Set button colors, depending if selected or not.
                    if self.selected[i] {
                        txt_col = sel_text_color; rect_col = sel_rect_color;
                    } else {
                        txt_col = unsel_text_color; rect_col = unsel_rect_color;
                    }
                    // Save widget NodeIndex so input states can be retrieved later
                    entry_idx_list[i]=item.widget_idx;

                    let button = widget::Button::new()
                        .label(&label)
                        .label_color(txt_col)
                        .color(rect_col)
                        .label_font_size(font_size)
                        .border(0.0)
                        .react(|| {
                            list_index_event = Some(i);
                        });
                    item.set(button);
                })
                .set(list_idx, &mut ui);

            // If an event (moue click, etc.) happened
            if let Some(i) = list_index_event {
                let multi_selections = self.multiple_selections;

                for widget_event in ui.widget_input(entry_idx_list[i as usize]).events() {
                    use event;
                    use input::{self};

                    match widget_event {

                        // Check if the entry has been `DoubleClick`ed.
                        event::Widget::DoubleClick(click) => {
                            if let input::MouseButton::Left = click.button {
                                // Deselect all others.
                                state.update(|state| {
                                    self.clear_all_selected(&mut state.last_selected_entry);
                                    self.selected[i] = true;
                                    state.last_selected_entry = Some(i);
                                });
                                if let Some(ref mut react) = self.maybe_react {
                                    react(Event::DoubleClick(i, self.entries[i].to_string()));
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
                                Some(idx) if is_shift_down && multi_selections => {

                                    // Default range
                                    let mut start_idx_range = std::cmp::min(idx, i);
                                    let mut end_idx_range = std::cmp::max(idx, i);

                                    // Get continous block selected from last selected idx
                                    // so the block can be extended up or down
                                    let (first, last) = self.get_selected_range(idx);

                                    if i<first {
                                        start_idx_range = i;
                                        end_idx_range = last;
                                    }
                                    if i>last {
                                        start_idx_range = first;
                                        end_idx_range = i;
                                    }

                                    // generate react list inline rather than using get_selected_as_tuples()
                                    let mut temp = Vec::new();
                                    state.update(|state| {

                                        self.clear_all_selected(&mut state.last_selected_entry);
                                        state.last_selected_entry = Some(i);

                                        // Set selected only for the range. Clearing other blocks.
                                        for j in 0..self.selected.len() {
                                            if start_idx_range <= j && j <= end_idx_range {
                                                self.selected[j] = true;
                                                temp.push((j as usize, self.entries[j].to_string()));
                                            } else {
                                                self.selected[j] = false;
                                            }
                                        }
                                    });

                                    if let Some(ref mut react) = self.maybe_react {
                                        react(Event::SelectEntries(temp))
                                    }
                                },

                                // If alt is down, additively select or deselect this item.
                                Some(_) | None if is_alt_down && multi_selections => {
                                    state.update(|state| {
                                        let new_is_selected = !self.selected[i];
                                        self.selected[i] = new_is_selected;
                                        if new_is_selected {
                                            state.last_selected_entry = Some(i);
                                        }
                                    });

                                    let mut list = self.get_selected_as_tuples();
                                    let num_entries_selected = list.len();

                                    if  num_entries_selected > 0 {
                                        if let Some(ref mut react) = self.maybe_react {
                                            if num_entries_selected > 1 {
                                                react(Event::SelectEntries(list));
                                            } else {
                                                // Safe to unwrap here, as list len is 1
                                                let (ix, st) = list.pop().unwrap();
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
                                        self.clear_all_selected(&mut state.last_selected_entry);
                                    });

                                    // Select the current item.
                                    state.update(|state| {
                                        self.selected[i] = true;
                                        state.last_selected_entry = Some(i);
                                    });
                                    if let Some(ref mut react) = self.maybe_react {
                                        react(Event::SelectEntry(i, self.entries[i].to_string()));
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
                                            self.clear_all_selected(&mut state.last_selected_entry);

                                            let i = if i == 0 { 0 } else { i - 1 };
                                            self.selected[i] = true;
                                            state.last_selected_entry = Some(i);

                                            if let Some(ref mut react) = self.maybe_react {
                                                react(Event::SelectEntry(i, self.entries[i].to_string()));
                                            }
                                        }),

                                        // Bump the selection down the list.
                                        input::Key::Down => state.update(|state| {
                                            // Clear old selected entries.
                                            self.clear_all_selected(&mut state.last_selected_entry);

                                            let last_idx = self.entries.len() - 1;
                                            let i = if i < last_idx { i + 1 } else { last_idx };
                                            self.selected[i] = true;
                                            state.last_selected_entry = Some(i);

                                            if let Some(ref mut react) = self.maybe_react {
                                                react(Event::SelectEntry(i, self.entries[i].to_string()));
                                            }
                                        }),

                                        _ => (),
                                    }

                                    // For any other pressed keys, yield an event along
                                    // with all the paths of all selected entries.
                                    let list = self.get_selected_as_tuples();
                                    if let Some(ref mut react) = self.maybe_react {
                                        let key_press = event::KeyPress {
                                            key: key,
                                            modifiers: press.modifiers,
                                        };
                                        react(Event::KeyPress(list, key_press));
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
    }

}

impl<'a, T, F> Colorable for ListSelect<'a, T, F> where F: FnMut(Event), T: Display {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, T, F> Borderable for ListSelect<'a, T, F> where F: FnMut(Event), T: Display {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}
