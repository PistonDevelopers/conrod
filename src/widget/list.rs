//! A helper widget, useful for instantiating a sequence of widgets in a vertical list.

use {
    color,
    Color,
    Colorable,
    Positionable,
    Scalar,
    Sizeable,
    Widget,
    Ui,
    UiCell,
};
use graph;
use std;
use widget;

/// A helper widget, useful for instantiating a sequence of widgets in a vertical list.
///
/// The `List` widget simplifies this process by:
///
/// - Generating `widget::Id`s.
/// - Simplifying the positioning and sizing of items.
/// - Optimised widget instantiation by only instantiating visible items. This is very useful for
///   lists containing many items, i.e. a `FileNavigator` over a directory with thousands of files.
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct List {
    /// Common widget building params for the `List`.
    pub common: widget::CommonBuilder,
    /// Unique styling for the `List`.
    pub style: Style,
    /// Whether all or only visible items should be instantiated.
    pub item_instantiation: ItemInstantiation,
    item_h: Scalar,
    num_items: usize,
}

widget_style! {
    /// Unique styling for the `List`.
    style Style {
        /// The width of the scrollbar if it is visible.
        - scrollbar_width: Option<Scalar> { None }
        /// The color of the scrollbar if it is visible.
        - scrollbar_color: Color { theme.border_color }
        /// The location of the `List`'s scrollbar.
        - scrollbar_position: Option<ScrollbarPosition> { None }
    }
}

widget_ids! {
    struct Ids {
        scroll_trigger,
        items[],
        scrollbar,
    }
}

/// Represents the state of the List widget.
pub struct State {
    ids: Ids,
}

/// The data necessary for instantiating a single item within a `List`.
#[derive(Copy, Clone, Debug)]
pub struct Item {
    /// The index of the item within the list.
    pub i: usize,
    /// The id generated for the widget.
    pub widget_id: widget::Id,
    /// The id used for the previous item's widget.
    pub last_id: Option<widget::Id>,
    /// The width of the item.
    pub w: Scalar,
    /// The height of the item.
    pub h: Scalar,
    /// The id of the `scroll_trigger` rectangle, upon which this widget will be placed.
    scroll_trigger_id: widget::Id,
    /// The distance between the top of the first visible item and the top of the `scroll_trigger`
    /// `Rectangle`. This field is used for positioning the item's widget.
    first_item_margin: Scalar,
}

/// The way in which a `List` should instantiate its `Item`s.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ItemInstantiation {
    /// Instantiate an `Item` for every element, regardless of visibility.
    All,
    /// Only instantiate visible `Item`s.
    OnlyVisible,
}

/// If the `List` is scrollable, this describes how th `Scrollbar` should be positioned.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ScrollbarPosition {
    /// To the right of the items (reduces the item width to fit).
    NextTo,
    /// On top of the right edge of the items with auto_hide activated.
    OnTop,
}

/// A wrapper around a `List`'s `Scrollbar` and its `widget::Id`.
pub struct Scrollbar {
    widget: widget::Scrollbar<widget::scroll::Y>,
    id: widget::Id,
}

/// An `Iterator` yielding each `Item` in the list.
pub struct Items {
    item_indices: std::ops::Range<usize>,
    next_item_indices_index: usize,
    list_id: widget::Id,
    last_id: Option<widget::Id>,
    scroll_trigger_id: widget::Id,
    first_item_margin: Scalar,
    item_w: Scalar,
    item_h: Scalar,
}


impl List {

    /// Create a List context to be built upon.
    pub fn new(num_items: usize, item_height: Scalar) -> Self {
        List {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
            item_h: item_height,
            num_items: num_items,
            item_instantiation: ItemInstantiation::OnlyVisible,
        }.crop_kids()
    }

    /// Specifies that the `List` should be scrollable and should provide a `Scrollbar` to the
    /// right of the items.
    pub fn scrollbar_next_to(mut self) -> Self {
        self.style.scrollbar_position = Some(Some(ScrollbarPosition::NextTo));
        self.scroll_kids_vertically()
    }

    /// Specifies that the `List` should be scrollable and should provide a `Scrollbar` that hovers
    /// above the right edge of the items and automatically hides when the user is not scrolling.
    pub fn scrollbar_on_top(mut self) -> Self {
        self.style.scrollbar_position = Some(Some(ScrollbarPosition::OnTop));
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
        self.item_instantiation = ItemInstantiation::All;
        self
    }

    /// Indicates that only `Item`s that are visible should be instantiated. This ensures that we
    /// avoid bloating the widget graph with unnecessary nodes and in turn keep traversal times to
    /// a minimum.
    ///
    /// This is the default `List` behaviour.
    pub fn instantiate_only_visible_items(mut self) -> Self {
        self.item_instantiation = ItemInstantiation::OnlyVisible;
        self
    }

}



impl Widget for List {
    type State = State;
    type Style = Style;
    type Event = (Items, Option<Scrollbar>);

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, rect, prev, mut ui, style, .. } = args;
        let List { item_h, num_items, item_instantiation, .. } = self;

        // We need a positive item height in order to do anything useful.
        assert!(item_h > 0.0, "the given item height was {:?} however it must be > 0", item_h);

        // Determine whther or not the list is scrollable.
        let is_scrollable = prev.maybe_y_scroll_state.as_ref()
            .map(|scroll_state| scroll_state.offset_bounds.magnitude().is_sign_negative())
            .unwrap_or(false);

        // The width of the scrollbar.
        let scrollbar_w = style.scrollbar_width(&ui.theme)
            .unwrap_or_else(|| {
                ui.theme.widget_style::<widget::scrollbar::Style>()
                    .and_then(|style| style.style.thickness)
                    .unwrap_or(10.0)
            });

        let scrollbar_position = style.scrollbar_position(&ui.theme);
        let item_w = match (is_scrollable, scrollbar_position) {
            (true, Some(ScrollbarPosition::NextTo)) => rect.w() - scrollbar_w,
            _ => rect.w(),
        };

        let total_item_h = num_items as Scalar * item_h;

        // The widget used to scroll the `List`'s range.
        //
        // By using one long `Rectangle` widget to trigger the scrolling, this allows us to only
        // instantiate the visible items.
        widget::Rectangle::fill([rect.w(), total_item_h])
            .mid_top_of(id)
            .color(color::TRANSPARENT)
            .parent(id)
            .set(state.ids.scroll_trigger, ui);

        // Determine the index range of the items that should be instantiated.
        let (item_idx_range, first_item_margin) = match item_instantiation {
            ItemInstantiation::All => {
                let range = 0..num_items;
                let margin = 0.0;
                (range, margin)
            },
            ItemInstantiation::OnlyVisible => {
                let scroll_trigger_rect = ui.rect_of(state.ids.scroll_trigger).unwrap();
                let hidden_range_length = (scroll_trigger_rect.top() - rect.top()).max(0.0);
                let num_top_hidden_items = hidden_range_length / item_h;
                let num_visible_items = rect.h() / item_h;

                let first_visible_item_idx = num_top_hidden_items.floor() as usize;
                let end_visible_item_idx = std::cmp::min(
                    (num_top_hidden_items + num_visible_items).ceil() as usize,
                    num_items,
                );

                let range = first_visible_item_idx..end_visible_item_idx;
                let margin = first_visible_item_idx as Scalar * item_h;
                (range, margin)
            },
        };

        // Ensure there are at least as many indices as there are visible items.
        if state.ids.items.len() < item_idx_range.len() {
            let id_gen = &mut ui.widget_id_generator();
            state.update(|state| state.ids.items.resize(item_idx_range.len(), id_gen));
        }

        let items = Items {
            list_id: id,
            item_indices: item_idx_range,
            next_item_indices_index: 0,
            last_id: None,
            scroll_trigger_id: state.ids.scroll_trigger,
            first_item_margin: first_item_margin,
            item_w: item_w,
            item_h: item_h,
        };

        // Instantiate the `Scrollbar` only if necessary.
        let auto_hide = match (is_scrollable, scrollbar_position) {
            (false, _) | (true, None) => return (items, None),
            (_, Some(ScrollbarPosition::NextTo)) => false,
            (_, Some(ScrollbarPosition::OnTop)) => true,
        };
        let scrollbar_color = style.scrollbar_color(&ui.theme);
        let scrollbar = widget::Scrollbar::y_axis(id)
            .and_if(prev.maybe_floating.is_some(), |s| s.floating(true))
            .color(scrollbar_color)
            .thickness(scrollbar_w)
            .auto_hide(auto_hide);
        let scrollbar = Scrollbar {
            widget: scrollbar,
            id: state.ids.scrollbar,
        };

        (items, Some(scrollbar))
    }
}


impl Items {

    /// Yield the next `Item` in the list.
    pub fn next(&mut self, ui: &Ui) -> Option<Item> {
        let Items {
            ref mut item_indices,
            ref mut next_item_indices_index,
            ref mut last_id,
            list_id,
            scroll_trigger_id,
            first_item_margin,
            item_w,
            item_h,
        } = *self;

        // Retrieve the `node_index` that was generated for the next `Item`.
        let node_index = match ui.widget_graph().widget(list_id)
            .and_then(|container| container.unique_widget_state::<List>())
            .and_then(|&graph::UniqueWidgetState { ref state, .. }| {
                state.ids.items.get(*next_item_indices_index).map(|&id| id)
            })
        {
            Some(node_index) => {
                *next_item_indices_index += 1;
                Some(node_index)
            },
            None => return None,
        };

        match (item_indices.next(), node_index) {
            (Some(i), Some(node_index)) => {
                let item = Item {
                    i: i,
                    last_id: *last_id,
                    widget_id: node_index,
                    scroll_trigger_id: scroll_trigger_id,
                    w: item_w,
                    h: item_h,
                    first_item_margin: first_item_margin,
                };
                *last_id = Some(node_index);
                Some(item)
            },
            _ => None,
        }
    }

}


impl Item {

    /// Sets the given widget as the widget to use for the item.
    ///
    /// Sets the:
    /// - position of the widget.
    /// - dimensions of the widget.
    /// - parent of the widget.
    /// - and finally sets the widget within the `Ui`.
    pub fn set<W>(self, widget: W, ui: &mut UiCell) -> W::Event
        where W: Widget,
    {
        let Item { widget_id, last_id, w, h, scroll_trigger_id, first_item_margin, .. } = self;

        widget
            .w_h(w, h)
            .and(|w| match last_id {
                None => w.mid_top_with_margin_on(scroll_trigger_id, first_item_margin)
                    .align_left_of(scroll_trigger_id),
                Some(id) => w.down_from(id, 0.0),
            })
            .parent(scroll_trigger_id)
            .set(widget_id, ui)
    }

}


impl Scrollbar {
    /// Set the `Scrollbar` within the given `Ui`.
    pub fn set(self, ui: &mut UiCell) {
        let Scrollbar { widget, id } = self;
        widget.set(id, ui);
    }
}
