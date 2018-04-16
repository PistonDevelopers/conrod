//! A wrapper around the `List` widget providing the ability to select one or more items.

use {Color, Positionable, Scalar, Sizeable, Ui, Widget};
use {event, graph, input, widget};
use std;
use input::keyboard::ModifierKey;
use input::state::mouse::Button;

/// A wrapper around the `List` widget that handles single and multiple selection logic.
#[derive(Clone, WidgetCommon_)]
#[allow(missing_copy_implementations)]
pub struct ListSelect<M, D, S> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    num_items: usize,
    mode: M,
    direction: std::marker::PhantomData<D>,
    item_size: S,
    style: widget::list::Style,
    item_instantiation: widget::list::ItemInstantiation,
}

/// A trait that extends the `List` `Direction` trait with behaviour necessary for the `ListSelect`
/// widget.
///
/// Implemented for the `Down`, `Right`, `Up`, `Left` types.
pub trait Direction: widget::list::Direction {
    /// Maps a given `key` to a direction along the list.
    fn key_direction(key: input::Key) -> Option<ListDirection>;
}

/// The direction in which the list flows.
#[derive(Copy, Clone, Debug)]
pub enum ListDirection {
    /// The direction flowing from the start of the list to the end of the list.
    Forward,
    /// The direction flowing from the end of the list to the start of the list.
    Backward,
}

/// Allows the `ListSelect` to be generic over `Single` and `Multiple` selection modes.
///
/// Also allows for defining other custom selection modes.
pub trait Mode {
    /// The data associated with the `Mode`s `Event::Selection`.
    type Selection;

    /// Update the `PendingEvents` in accordance with the given `Click` event.
    fn click_selection<F, D, S>(&self,
                                event::Click,
                                i: usize,
                                num_items: usize,
                                &State,
                                is_selected: F,
                                &mut PendingEvents<Self::Selection, D, S>)
        where F: Fn(usize) -> bool;

    /// Update the `PendingEvents` in accordance with the given `KeyPress` event.
    fn key_selection<F, D, S>(&self,
                              event::KeyPress,
                              i: usize,
                              num_items: usize,
                              &State,
                              is_selected: F,
                              &mut PendingEvents<Self::Selection, D, S>)
        where F: Fn(usize) -> bool,
              D: Direction;
}

widget_ids! {
    struct Ids {
        list,
    }
}

/// Represents the state of the ListSelect.
pub struct State {
    ids: Ids,
    /// Tracking index of last selected entry that has been pressed in order to
    /// perform multi selection when `SHIFT` or `ALT`(Mac) / 'CTRL'(Other OS) is held.
    last_selected_entry: std::cell::Cell<Option<usize>>,
}

/// Buffer used for storing events that have been produced but are yet to be yielded.
pub type PendingEvents<Selection, D, S> = std::collections::VecDeque<Event<Selection, D, S>>;

/// An iterator-like type for yielding `ListSelect` `Event`s.
pub struct Events<M, D, S>
    where M: Mode,
{
    id: widget::Id,
    items: widget::list::Items<D, S>,
    num_items: usize,
    mode: M,
    pending_events: PendingEvents<M::Selection, D, S>,
}

/// The kind of events that the `ListSelect` may `react` to.
/// Provides tuple(s) of index in list and string representation of selection
#[derive(Clone, Debug)]
pub enum Event<Selection, Direction, Size> {
    /// The next `Item` is ready for instantiation.
    Item(widget::list::Item<Direction, Size>),
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

/// A single item selection `Mode` for the `ListSelect`.
#[derive(Copy, Clone)]
pub struct Single;

/// A selection `Mode` for the `ListSelect` that allows selecting more than one item at a time.
#[derive(Copy, Clone)]
pub struct Multiple;

/// Represents some change in item selection for a `ListSelect` in `Multiple` mode.
#[derive(Clone, Debug)]
pub enum Selection<H: std::hash::BuildHasher = std::collections::hash_map::RandomState> {
    /// Items which have been added to the selection.
    Add(std::collections::HashSet<usize, H>),
    /// Items which have been removed from the selection.
    Remove(std::collections::HashSet<usize, H>),
}


impl<H: std::hash::BuildHasher> Selection<H> {

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
    pub fn update_index_set<T>(&self, set: &mut std::collections::HashSet<usize, T>)
        where T: std::hash::BuildHasher
    {
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


impl ListSelect<Single, widget::list::Down, widget::list::Dynamic> {
    /// Construct a new ListSelect, allowing one selected item at a time.
    pub fn single(num_items: usize) -> Self {
        Self::new(num_items, Single)
    }
}

impl ListSelect<Multiple, widget::list::Down, widget::list::Dynamic> {
    /// Construct a new ListSelect, allowing multiple selected items.
    pub fn multiple(num_items: usize) -> Self {
        Self::new(num_items, Multiple)
    }
}

impl<M, D, S> ListSelect<M, D, S>
    where M: Mode,
          D: Direction,
          S: widget::list::ItemSize,
{

    /// Flows items from top to bottom.
    pub fn flow_down(self) -> ListSelect<M, widget::list::Down, S> {
        let ListSelect { common, num_items, mode, item_size, style, item_instantiation, .. } = self;
        ListSelect {
            common: common,
            num_items: num_items,
            mode: mode,
            direction: std::marker::PhantomData,
            item_size: item_size,
            style: style,
            item_instantiation: item_instantiation,
        }
    }

    /// Flows items from left to right.
    pub fn flow_right(self) -> ListSelect<M, widget::list::Right, S> {
        let ListSelect { common, num_items, mode, item_size, style, item_instantiation, .. } = self;
        ListSelect {
            common: common,
            num_items: num_items,
            mode: mode,
            direction: std::marker::PhantomData,
            item_size: item_size,
            style: style,
            item_instantiation: item_instantiation,
        }
    }

    /// Flows items from right to left.
    pub fn flow_left(self) -> ListSelect<M, widget::list::Left, S> {
        let ListSelect { common, num_items, mode, item_size, style, item_instantiation, .. } = self;
        ListSelect {
            common: common,
            num_items: num_items,
            mode: mode,
            direction: std::marker::PhantomData,
            item_size: item_size,
            style: style,
            item_instantiation: item_instantiation,
        }
    }

    /// Flows items from bottom to top.
    pub fn flow_up(self) -> ListSelect<M, widget::list::Up, S> {
        let ListSelect { common, num_items, mode, item_size, style, item_instantiation, .. } = self;
        ListSelect {
            common: common,
            num_items: num_items,
            mode: mode,
            direction: std::marker::PhantomData,
            item_size: item_size,
            style: style,
            item_instantiation: item_instantiation,
        }
    }

    /// Specify a fixed item size, where size is a `Scalar` in the direction that the `List` is
    /// flowing. When a `List` is constructed with this method, all items will have a fixed, equal
    /// length.
    pub fn item_size(self, length: Scalar) -> ListSelect<M, D, widget::list::Fixed> {
        let ListSelect { common, num_items, mode, direction, style, .. } = self;
        ListSelect {
            common: common,
            num_items: num_items,
            mode: mode,
            direction: direction,
            item_size: widget::list::Fixed { length: length },
            style: style,
            item_instantiation: widget::list::ItemInstantiation::OnlyVisible,
        }
    }
}

impl<M> ListSelect<M, widget::list::Down, widget::list::Dynamic> {
    /// Begin building a new `ListSelect` with the given mode.
    ///
    /// This method is only useful when using a custom `Mode`, otherwise `ListSelect::single` or
    /// `ListSelect::multiple` will probably be more suitable.
    pub fn new(num_items: usize, mode: M) -> Self
        where M: Mode,
    {
        ListSelect {
            common: widget::CommonBuilder::default(),
            style: widget::list::Style::default(),
            num_items: num_items,
            item_size: widget::list::Dynamic {},
            mode: mode,
            direction: std::marker::PhantomData,
            item_instantiation: widget::list::ItemInstantiation::All,
        }
    }
}

impl<M, D, S> ListSelect<M, D, S> {

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
    pub fn scrollbar_thickness(mut self, w: Scalar) -> Self {
        self.style.scrollbar_thickness = Some(Some(w));
        self
    }

    /// The color of the `Scrollbar`.
    pub fn scrollbar_color(mut self, color: Color) -> Self {
        self.style.scrollbar_color = Some(color);
        self
    }

}

impl<M, D> ListSelect<M, D, widget::list::Fixed> {

    /// Indicates that an `Item` should be instatiated for every element in the list, regardless of
    /// whether or not the `Item` would be visible.
    ///
    /// This is the default (and only) behaviour for `List`s with dynamic item sizes. This is
    /// because a `List` cannot know the total length of its combined items in advanced when each
    /// item is dynamically sized and their size is not given until they are set.
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
    /// This is the default behaviour for `ListSelect`s with fixed item sizes.
    pub fn instantiate_only_visible_items(mut self) -> Self {
        self.item_instantiation = widget::list::ItemInstantiation::OnlyVisible;
        self
    }

}

impl<M, D, S> Widget for ListSelect<M, D, S>
    where M: Mode,
          D: Direction,
          S: widget::list::ItemSize,
{
    type State = State;
    type Style = widget::list::Style;
    type Event = (Events<M, D, S>, Option<widget::list::Scrollbar<D::Axis>>);

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
            last_selected_entry: std::cell::Cell::new(None),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the ListSelect.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, style, ui, .. } = args;
        let ListSelect { num_items, item_size, item_instantiation, mode, .. } = self;

        // Make sure that `last_selected_entry` refers to an actual selected value in the list.
        // If not push first selected item, if any.
        if let Some(i) = state.last_selected_entry.get() {
            if i >= num_items {
                state.update(|state| state.last_selected_entry.set(None));
            }
        }


        let mut list = widget::List::<D, _>::from_item_size(num_items, item_size);

        let scrollbar_position = style.scrollbar_position(&ui.theme);
        list = match scrollbar_position {
            Some(widget::list::ScrollbarPosition::OnTop) => list.scrollbar_on_top(),
            Some(widget::list::ScrollbarPosition::NextTo) => list.scrollbar_next_to(),
            None => list,
        };

        list.item_instantiation = item_instantiation;
        list.style = style.clone();
        let (items, scrollbar) = list.middle_of(id).wh_of(id).set(state.ids.list, ui);

        let events = Events {
            id: id,
            items: items,
            num_items: num_items,
            mode: mode,
            pending_events: PendingEvents::new(),
        };

        (events, scrollbar)
    }
}

impl<M, D, S> Events<M, D, S>
    where M: Mode,
          D: Direction,
          S: widget::list::ItemSize,
{

    /// Yield the next `Event`.
    pub fn next<F>(&mut self, ui: &Ui, is_selected: F) -> Option<Event<M::Selection, D, S>>
        where F: Fn(usize) -> bool,
    {
        let Events {
            id,
            num_items,
            ref mode,
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
                .widget(id)
                .and_then(|container| container.unique_widget_state::<ListSelect<M, D, S>>())
                .map(|&graph::UniqueWidgetState { ref state, .. }| state)
                .expect("couldn't find `ListSelect` state in the widget graph")
        };

        // Ensure's the last selected entry is still selected.
        //
        // Sets the `last_selected_entry` to `None` if it is no longer selected.
        let ensure_last_selected_validity = |state: &State| {
            if let Some(i) = state.last_selected_entry.get() {
                if !is_selected(i) {
                    state.last_selected_entry.set(None);
                }
            }
        };

        let i = item.i;

        // Check for any events that may have occurred to this widget.
        for widget_event in ui.widget_input(item.widget_id).events() {
            match widget_event {

                // Produce a `DoubleClick` event.
                event::Widget::DoubleClick(click) => {
                    if let input::MouseButton::Left = click.button {
                        pending_events.push_back(Event::DoubleClick(click));
                    }
                },

                // Check if the entry has been `Click`ed.
                event::Widget::Click(click) => {
                    pending_events.push_back(Event::Click(click));

                    let state = state();
                    ensure_last_selected_validity(state);
                    mode.click_selection(click, i, num_items, state,
                                         &is_selected, pending_events);
                },

                // Check for whether or not the item should be selected.
                event::Widget::Press(press) => {
                    pending_events.push_back(Event::Press(press));

                    if let Some(key_press) = press.key() {
                        let state = state();
                        ensure_last_selected_validity(state);
                        mode.key_selection(key_press, i, num_items, state,
                                           &is_selected, pending_events);
                    }
                },
                
                event::Widget::Tap(_) => {
                    let dummy_click=event::Click{
                        button:Button::Left,
                        xy:[200.0,123.0],
                        modifiers:ModifierKey::NO_MODIFIER
                    };
                    pending_events.push_back(Event::Click(dummy_click.clone()));
                    let state = state();
                    ensure_last_selected_validity(state);
                    mode.click_selection(dummy_click, i, num_items, state,
                                         &is_selected, pending_events);
                },
                // Produce a `Release` event.
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

impl Mode for Single {
    type Selection = usize;

    fn click_selection<F, D, S>(&self,
                                _: event::Click,
                                i: usize,
                                _num_items: usize,
                                state: &State,
                                _is_selected: F,
                                pending: &mut PendingEvents<Self::Selection, D, S>)
        where F: Fn(usize) -> bool,
    {
        state.last_selected_entry.set(Some(i));
        let event = Event::Selection(i);
        pending.push_back(event);
    }

    fn key_selection<F, D, S>(&self,
                              press: event::KeyPress,
                              _i: usize,
                              num_items: usize,
                              state: &State,
                              _is_selected: F,
                              pending: &mut PendingEvents<Self::Selection, D, S>)
        where F: Fn(usize) -> bool,
              D: Direction,
    {
        let i = match state.last_selected_entry.get() {
            Some(i) => i,
            None => return,
        };

        let selection = match D::key_direction(press.key) {
            Some(ListDirection::Backward) => if i == 0 { 0 } else { i - 1 },
            Some(ListDirection::Forward) => std::cmp::min(i + 1, num_items - 1),
            None => return,
        };

        state.last_selected_entry.set(Some(selection));
        let event = Event::Selection(selection);
        pending.push_back(event);
    }

}

impl Mode for Multiple {
    type Selection = Selection;

    fn click_selection<F, D, S>(&self,
                                click: event::Click,
                                i: usize,
                                num_items: usize,
                                state: &State,
                                is_selected: F,
                                pending: &mut PendingEvents<Self::Selection, D, S>)
        where F: Fn(usize) -> bool,
    {
        let shift = click.modifiers.contains(input::keyboard::ModifierKey::SHIFT);
        let alt = click.modifiers.contains(input::keyboard::ModifierKey::ALT)
               || click.modifiers.contains(input::keyboard::ModifierKey::CTRL);

        let event = match state.last_selected_entry.get() {

            Some(idx) if shift => {
                let start = std::cmp::min(idx, i);
                let end = std::cmp::max(idx, i);

                state.last_selected_entry.set(Some(i));
                let selection = (start..end + 1).collect();
                Event::Selection(Selection::Add(selection))
            },

            Some(_) | None if alt => {
                let selection = std::iter::once(i).collect();
                if !is_selected(i) {
                    state.last_selected_entry.set(Some(i));
                    Event::Selection(Selection::Add(selection))
                } else {
                    Event::Selection(Selection::Remove(selection))
                }
            },

            _ => {
                let old_selection = (0..num_items).filter(|&i| is_selected(i)).collect();
                let event = Event::Selection(Selection::Remove(old_selection));
                pending.push_back(event);
                let selection = std::iter::once(i).collect();
                state.last_selected_entry.set(Some(i));
                Event::Selection(Selection::Add(selection))
            },

        };

        pending.push_back(event);
    }

    fn key_selection<F, D, S>(&self,
                              press: event::KeyPress,
                              _i: usize,
                              num_items: usize,
                              state: &State,
                              is_selected: F,
                              pending: &mut PendingEvents<Self::Selection, D, S>)
        where F: Fn(usize) -> bool,
              D: Direction,
    {
        let i = match state.last_selected_entry.get() {
            Some(i) => i,
            None => return,
        };

        let alt = press.modifiers.contains(input::keyboard::ModifierKey::ALT);

        let end = match D::key_direction(press.key) {
            Some(ListDirection::Backward) =>
                if i == 0 || alt { 0 } else { i - 1 },
            Some(ListDirection::Forward) => {
                let last_idx = num_items - 1;
                if i >= last_idx || alt { last_idx } else { i + 1 }
            },
            None => return,
        };

        state.last_selected_entry.set(Some(end));

        let selection = if press.modifiers.contains(input::keyboard::ModifierKey::SHIFT) {
            let start = std::cmp::min(i, end);
            let end = std::cmp::max(i, end) + 1;
            (start..end).collect()
        } else {
            let old_selection = (0..num_items).filter(|&i| is_selected(i)).collect();
            let event = Event::Selection(Selection::Remove(old_selection));
            pending.push_back(event);
            std::iter::once(end).collect()
        };

        let event = Event::Selection(Selection::Add(selection));
        pending.push_back(event);
    }

}

impl Direction for widget::list::Down {
    fn key_direction(key: input::Key) -> Option<ListDirection> {
        match key {
            input::Key::Down => Some(ListDirection::Forward),
            input::Key::Up => Some(ListDirection::Backward),
            _ => None,
        }
    }
}

impl Direction for widget::list::Up {
    fn key_direction(key: input::Key) -> Option<ListDirection> {
        match key {
            input::Key::Up => Some(ListDirection::Forward),
            input::Key::Down => Some(ListDirection::Backward),
            _ => None,
        }
    }
}

impl Direction for widget::list::Right {
    fn key_direction(key: input::Key) -> Option<ListDirection> {
        match key {
            input::Key::Right => Some(ListDirection::Forward),
            input::Key::Left => Some(ListDirection::Backward),
            _ => None,
        }
    }
}

impl Direction for widget::list::Left {
    fn key_direction(key: input::Key) -> Option<ListDirection> {
        match key {
            input::Key::Left => Some(ListDirection::Forward),
            input::Key::Right => Some(ListDirection::Backward),
            _ => None,
        }
    }
}
