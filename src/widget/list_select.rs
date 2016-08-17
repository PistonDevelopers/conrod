//! A wrapper around the `List` widget providing the ability to select one or more items.

use {Color, Positionable, Scalar, Sizeable, Ui, Widget};
use {event, graph, widget};
use std;

/// A wrapper around the `List` widget that handles single and multiple selection logic.
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct ListSelect {
    common: widget::CommonBuilder,
    item_h: Scalar,
    num_items: usize,
    multiple_selections: bool,
    style: widget::list::Style,
    item_instantiation: widget::list::ItemInstantiation,
}

/// Represents the state of the ListSelect.
#[derive(PartialEq, Clone, Debug)]
pub struct State {
    list_idx: widget::IndexSlot,
    /// Tracking index of last selected entry that has been pressed in order to
    /// perform multi selection when `SHIFT` or `ALT`(Mac) / 'CTRL'(Other OS) is held.
    last_selected_entry: std::cell::Cell<Option<usize>>,
}

/// An iterator-like type for yielding `ListSelect` `Event`s.
pub struct Events {
    idx: widget::Index,
    items: widget::list::Items,
    num_items: usize,
    multiple_selections: bool,
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
            Selection::Add(ref indices) =>
                for &i in indices {
                    set.insert(i);
                },
            Selection::Remove(ref indices) =>
                for &i in indices {
                    set.remove(&i);
                },
        }
    }

}

impl ListSelect {

    /// Internal constructor
    fn new(num_items: usize, item_h: Scalar, multiple_selection: bool) -> Self {
        ListSelect {
            common: widget::CommonBuilder::new(),
            style: widget::list::Style::new(),
            item_h: item_h,
            num_items: num_items,
            multiple_selections: multiple_selection,
            item_instantiation: widget::list::ItemInstantiation::OnlyVisible,
        }
    }

    /// Construct a new ListSelect, allowing one selected item at a time.
    /// Second parameter is a list reflecting which entries within the list are currently selected.
    pub fn single(num_items: usize, item_h: Scalar) -> Self {
        ListSelect::new(num_items, item_h, false)
    }

    /// Construct a new ListSelect, allowing multiple selected items.
    /// Second parameter is a list reflecting which entries within the list are currently selected.
    pub fn multiple(num_items: usize, item_h: Scalar) -> Self {
        ListSelect::new(num_items, item_h, true)
    }

    /// Specifies that the `List` should be scrollable and should provide a `Scrollbar` to the
    /// right of the items.
    pub fn scrollbar_next_to(mut self) -> Self {
        self.style.scrollbar_position = Some(Some(widget::list::ScrollbarPosition::NextTo));
        self
    }

    /// Specifies that the `List` should be scrollable and should provide a `Scrollbar` that hovers
    /// above the right edge of the items and automatically hides when the user is not scrolling.
    pub fn scrollbar_on_top(mut self) -> Self {
        self.style.scrollbar_position = Some(Some(widget::list::ScrollbarPosition::OnTop));
        self
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
            last_selected_entry: std::cell::Cell::new(None),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the ListSelect.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { idx, mut state, style, mut ui, .. } = args;
        let ListSelect { num_items, item_h, item_instantiation, multiple_selections, .. } = self;

        // Make sure that `last_selected_entry` refers to an actual selected value in the list.
        // If not push first selected item, if any.
        if let Some(i) = state.last_selected_entry.get() {
            if i >= num_items {
                state.update(|state| state.last_selected_entry.set(None));
            }
        }

        let list_idx = state.list_idx.get(&mut ui);
        let scrollbar_position = style.scrollbar_position(&ui.theme);

        let mut list = widget::List::new(num_items, item_h)
            .and_if(scrollbar_position.is_some(), |ls| ls.scroll_kids_vertically());
        list.item_instantiation = item_instantiation;
        list.style = style.clone();
        let (items, scrollbar) = list.middle_of(idx).wh_of(idx).set(list_idx, &mut ui);

        let events = Events {
            idx: idx,
            items: items,
            num_items: num_items,
            multiple_selections: multiple_selections,
            pending_events: std::collections::VecDeque::new(),
        };

        (events, scrollbar)
    }
}

impl Events {

    /// Yield the next `Event`.
    pub fn next<F>(&mut self, ui: &Ui, is_selected: F) -> Option<Event>
        where F: Fn(usize) -> bool,
    {
        let Events {
            idx,
            multiple_selections,
            num_items,
            ref mut items,
            ref mut pending_events,
        } = *self;

        if let Some(event) = pending_events.pop_front() {
            return Some(event);
        }

        let item = match items.next(ui) {
            Some(item) => item,
            None => return None,
        };

        // Borrow the `ListSelect::State` from the `Ui`'s widget graph.
        let state = || {
            ui.widget_graph()
                .widget(idx)
                .and_then(|container| container.unique_widget_state::<ListSelect>())
                .map(|&graph::UniqueWidgetState { ref state, .. }| state)
                .expect("couldn't find `ListSelect` state in the widget graph")
        };

        // Produce the current selection as a set of indices.
        let current_selection = || {
            (0..num_items).filter(|&i| is_selected(i)).collect()
        };

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
                    pending_events.push_back(Event::Click(click));
                    let is_shift_down = click.modifiers.contains(input::keyboard::SHIFT);

                    // On Mac ALT is use for adding to selection, on Windows it's CTRL
                    let is_alt_down = click.modifiers.contains(input::keyboard::ALT) ||
                                      click.modifiers.contains(input::keyboard::CTRL);

                    let state = state();

                    let selection_event = match state.last_selected_entry.get() {

                        // If there is already a currently selected item and shift is
                        // held, extend the selection to this one.
                        Some(idx) if is_shift_down && multiple_selections => {

                            let start_idx_range = std::cmp::min(idx, i);
                            let end_idx_range = std::cmp::max(idx, i);

                            state.last_selected_entry.set(Some(i));
                            let selection = (start_idx_range..end_idx_range + 1).collect();
                            Event::Selection(Selection::Add(selection))
                        },

                        // If alt is down, additively select or deselect this item.
                        Some(_) | None if is_alt_down && multiple_selections => {
                            let selection = std::iter::once(i).collect();
                            if !is_selected(i) {
                                state.last_selected_entry.set(Some(i));
                                Event::Selection(Selection::Add(selection))
                            } else {
                                Event::Selection(Selection::Remove(selection))
                            }
                        },

                        // Otherwise, no shift/ctrl/alt, select just this one
                        // Clear all others
                        _ => {
                            let old_selection = current_selection();
                            let event = Event::Selection(Selection::Remove(old_selection));
                            pending_events.push_back(event);
                            let selection = std::iter::once(i).collect();
                            state.last_selected_entry.set(Some(i));
                            Event::Selection(Selection::Add(selection))
                        },
                    };

                    pending_events.push_back(selection_event);
                },

                // Check for whether or not the item should be selected.
                event::Widget::Press(press) => {
                    pending_events.push_back(Event::Press(press));
                    let state = state();
                    match press.button {

                        // Keyboard check whether the selection has been bumped up or down.
                        event::Button::Keyboard(key) => {
                            if let Some(i) = state.last_selected_entry.get() {
                                let alt = press.modifiers.contains(input::keyboard::ALT);

                                let end = match key {
                                    input::Key::Up =>
                                        if i == 0 || alt { 0 } else { i - 1 },
                                    input::Key::Down => {
                                        let last_idx = num_items - 1;
                                        if i >= last_idx || alt { last_idx } else { i + 1 }
                                    },
                                    _ => continue,
                                };

                                state.last_selected_entry.set(Some(end));

                                let selection = if press.modifiers.contains(input::keyboard::SHIFT) {
                                    let start = std::cmp::min(i, end);
                                    let end = std::cmp::max(i, end) + 1;
                                    (start..end).collect()
                                } else {
                                    let old_selection = current_selection();
                                    let event = Event::Selection(Selection::Remove(old_selection));
                                    pending_events.push_back(event);
                                    std::iter::once(end).collect()
                                };

                                let event = Event::Selection(Selection::Add(selection));
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
