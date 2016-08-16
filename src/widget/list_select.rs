//! A wrapper around the `List` widget providing the ability to select one or more items.

use {Color, FontSize, Positionable, Scalar, Sizeable, Ui, Widget};
use {event, widget};
use std;

/// A wrapper around the `List` widget that handles single and multiple selection logic.
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct ListSelect {
    common: widget::CommonBuilder,
    item_h: Scalar,
    num_items: u32,
    multiple_selections: bool,
    style: widget::list::Style,
    item_instantiation: widget::list::ItemInstantiation,
}

// widget_style!{
//     /// Styling for the ListSelect, necessary for constructing its renderable Element.
//     style Style {
//         /// Size of text font in list
//         - font_size: FontSize { theme.font_size_medium }
//         /// The background color of the unselected entries.
//         - color: Color { theme.shape_color }
//         /// The background color of the selected entries.
//         - selected_color: Color { theme.label_color }
//         /// Color of the item text, when not selected.
//         - text_color: Color { theme.shape_color }
//         /// Color of the item text, when selected.
//         - selected_text_color: Color { color::BLACK }
//         /// Width of the border surrounding the widget
//         - border: Scalar { theme.border_width }
//         /// The color of the border.
//         - border_color: Color { theme.border_color }
//         /// Font size for the item labels.
//         - label_font_size: FontSize { theme.font_size_medium }
//         /// Auto hide the scrollbar or not
//         - scrollbar_auto_hide: bool { true }
//     }
// }

/// Represents the state of the ListSelect.
#[derive(PartialEq, Clone, Debug)]
pub struct State {
    list_idx: widget::IndexSlot,
    /// Tracking index of last selected entry that has been pressed in order to
    /// perform multi selection when `SHIFT` or `ALT`(Mac) / 'CTRL'(Other OS) is held.
    last_selected_entry: Option<usize>,
}

/// An iterator-like type for yielding `ListSelect` `Event`s.
pub struct Events {
    items: widget::list::Items,
    pending_events: std::collections::VecDeque<Event>,
}

/// The kind of events that the `ListSelect` may `react` to.
/// Provides tuple(s) of index in list and string representation of selection
#[derive(Clone, Debug)]
pub enum Event {
    /// The next `Item` is ready for instantiation.
    Item(widget::list::Item),
    /// A change in selection has occurred.
    Selection(Selection),
    /// A button press occurred while the widget was capturing the mouse.
    Press(event::Press),
    /// A button release occurred while the widget was capturing the mouse.
    Release(event::Release),
    /// A click occurred while the widget was capturing the mouse.
    Click(event::Click),
    /// A double click occurred while the widget was capturing the mouse.
    DoubleClick(event::DoubleClick),
}

/// Represents some change in item selection.
#[derive(Clone, Debug)]
pub enum Selection {
    /// Items which have been added to the selection.
    Add(std::collections::HashSet<usize>),
    /// Items which have been removed from the selection.
    Remove(std::collections::HashSet<usize>),
}

impl Selection {

    /// Update the given slice of `bool`s with this `Selection`.
    ///
    /// Each index in the `Selection` represents and index into the slice.
    pub fn update_bool_slice(&self, slice: &mut [bool]) {
        match *self {
            Selection::Add(ref indices) =>
                for &i in indices {
                    if let Some(b) = slice.get_mut(i) {
                        *b = true;
                    }
                },
            Selection::Remove(ref indices) =>
                for &i in indices {
                    if let Some(b) = slice.get_mut(i) {
                        *b = false;
                    }
                },
        }
    }

    /// Update the given set of selected indices with this `Selection`.
    pub fn update_index_set(&self, set: &mut std::collections::HashSet<usize>) {
        match *self {
            Selection::Add(ref indices) => for &i in indices { set.insert(i) },
            Selection::Remove(ref indices) => for &i in indices { set.remove(i) },
        }
    }

}

impl ListSelect {

    /// Internal constructor
    fn new(num_items: u32, item_h: Scalar, multiple_selection: bool) -> Self {
        ListSelect {
            common: widget::CommonBuilder::new(),
            num_items: num_items,
            item_h: item_h,
            multiple_selections: multiple_selection,
            style: widget::list::Style::new(),
        }
    }

    /// Construct a new ListSelect, allowing one selected item at a time.
    /// Second parameter is a list reflecting which entries within the list are currently selected.
    pub fn single(num_items: u32, item_h: Scalar) -> Self {
        ListSelect::new(num_items, item_h, false)
    }

    /// Construct a new ListSelect, allowing multiple selected items.
    /// Second parameter is a list reflecting which entries within the list are currently selected.
    pub fn multiple(num_items: u32, item_h: Scalar) -> Self {
        ListSelect::new(num_items, item_h, true)
    }

    /// Specifies that the `List` should be scrollable and should provide a `Scrollbar` to the
    /// right of the items.
    pub fn scrollbar_next_to(mut self) -> Self {
        self.style.scrollbar_position = Some(Some(widget::list::ScrollbarPosition::NextTo));
        self.scroll_kids_vertically()
    }

    /// Specifies that the `List` should be scrollable and should provide a `Scrollbar` that hovers
    /// above the right edge of the items and automatically hides when the user is not scrolling.
    pub fn scrollbar_on_top(mut self) -> Self {
        self.style.scrollbar_position = Some(Some(widget::list::ScrollbarPosition::OnTop));
        self.scroll_kids_vertically()
    }

    /// The width of the `Scrollbar`.
    pub fn scrollbar_width(mut self, w: Scalar) -> Self {
        self.style.scrollbar_width = Some(Some(w));
        self
    }

    /// The color of the `Scrollbar`.
    pub fn scrollbar_color(mut self, color: Color) -> Self {
        self.style.scrollbar_color = Some(color);
        self
    }

    /// Indicates that an `Item` should be instatiated for every element in the list, regardless of
    /// whether or not the `Item` would be visible.
    ///
    /// Note: This may cause significantly heavier CPU load for lists containing many items (100+).
    /// We only recommend using this when absolutely necessary as large lists may cause unnecessary
    /// bloating within the widget graph, and in turn result in greater traversal times.
    pub fn instantiate_all_items(mut self) -> Self {
        self.item_instantiation = widget::list::ItemInstantiation::All;
        self
    }

    /// Indicates that only `Item`s that are visible should be instantiated. This ensures that we
    /// avoid bloating the widget graph with unnecessary nodes and in turn keep traversal times to
    /// a minimum.
    ///
    /// This is the default `List` behaviour.
    pub fn instantiate_only_visible_items(mut self) -> Self {
        self.item_instantiation = widget::list::ItemInstantiation::OnlyVisible;
        self
    }

}

impl Widget for ListSelect {
    type State = State;
    type Style = widget::list::Style;
    type Event = (Events, Option<widget::list::Scrollbar>);

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self) -> Self::State {
        State {
            list_idx: widget::IndexSlot::new(),
            last_selected_entry:None,
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the ListSelect.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { idx, mut state, style, mut ui, .. } = args;
        let ListSelect { num_items, item_h, multiple_selections, .. } = self;

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

        let list_idx = state.list_idx.get(&mut ui);
        let mut list = widget::List::new(num_items, item_h);
        list.style = style;
        let (items, scrollbar) = list.middle_of(idx).wh_of(idx).set(list_idx, &mut ui);

        let events = Events {
            items: items,
            pending_events: std::collections::VecDeque::new(),
        };

        (events, scrollbar)
    }
}

impl Events {

    /// Yield the next `Event`.
    pub fn next<F>(&mut self, ui: &Ui, is_selected: F) -> Option<Event>
        where F: FnMut(usize) -> bool,
    {
        let Events { list_select_idx, ref mut items, ref mut pending_events } = *self;

        if let Some(event) = pending_events.pop_front() {
            return Some(event);
        }

        let item = match items.next(ui) {
            Some(item) => item,
            None => return None,
        };

        // Borrow the `ListSelect::State` from the `Ui`'s widget graph.
        fn state(list_select_idx: widget::Index, ui: &Ui) -> &State {
            ui.widget_graph().widget(list_select_idx)
        }

        let i = item.i;

        // Check for any events that may have occurred to this widget.
        for widget_event in ui.widget_input(item.widget_idx).events() {
            use {event, input};

            match widget_event {

                // Check if the entry has been `DoubleClick`ed.
                event::Widget::DoubleClick(click) => {
                    if let input::MouseButton::Left = click.button {
                        pending_events.push_back(Event::DoubleClick(click));
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

                    pending_events.push_back(Event::Click(click));
                    pending_events.push_back(selection_event);
                },

                // Check for whether or not the item should be selected.
                event::Widget::Press(press) => {
                    pending_events.push_back(Event::Press(press));
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

                                pending_events.push_back(event);
                            }
                        },

                        _ => (),
                    }
                },

                event::Widget::Release(release) => {
                    let event = Event::Release(release);
                    pending_events.push_back(event);
                },

                _ => (),
            }
        }

        let item_event = Event::Item(item);

        // If we can avoid causing `pending_events` to allocate, do so.
        match pending_events.pop_front() {
            Some(event) => {
                pending_events.push_back(item_event);
                Some(event)
            },
            None => Some(item_event),
        }
    }

}
