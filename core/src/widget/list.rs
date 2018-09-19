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
use position::{Range, Rect};
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
#[derive(Clone, WidgetCommon_)]
#[allow(missing_copy_implementations)]
pub struct List<D, S> {
    /// Common widget building params for the `List`.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// Unique styling for the `List`.
    pub style: Style,
    /// Whether all or only visible items should be instantiated.
    pub item_instantiation: ItemInstantiation,
    num_items: usize,
    direction: std::marker::PhantomData<D>,
    item_size: S,
}

/// Items flow from bottom to top.
#[derive(Clone, Copy, Debug)]
pub enum Up {}
/// Items flow from top to bottom.
#[derive(Clone, Copy, Debug)]
pub enum Down {}
/// Items flow from right to left.
#[derive(Clone, Copy, Debug)]
pub enum Left {}
/// Items flow from left to right.
#[derive(Clone, Copy, Debug)]
pub enum Right {}

/// A type that implements `ItemSize` for `List`s whose `Item`s are a fixed size and known prior to
/// setting the widgets for each item.
#[derive(Clone, Copy, Debug)]
pub struct Fixed {
    /// The length of each item in the direction that the list flows.
    pub length: Scalar,
}

/// A type that implements `ItemSize` for `List`s whose `Item`s' length are unknown until setting
/// the widget for each item.
#[derive(Clone, Copy, Debug)]
pub struct Dynamic {}

/// The direction in which the list is laid out.
pub trait Direction {
    /// The direction along which the `Scrollbar` is laid out.
    type Axis: widget::scrollbar::Axis;

    /// For some given `Rect`, returns the parallel and perpendicular ranges respectively.
    fn ranges(Rect) -> (Range, Range);

    /// Begin building the scrollbar for the `List`.
    fn scrollbar(widget::Id) -> widget::Scrollbar<Self::Axis>;

    /// Borrow the scroll state associated with this `Direction`'s axis.
    fn common_scroll(common: &widget::CommonBuilder) -> Option<&widget::scroll::Scroll>;

    /// Positions the given widget.
    fn position_item<W>(item_widget: W,
                        last_id: Option<widget::Id>,
                        scroll_trigger_id: widget::Id,
                        first_item_margin: Scalar) -> W
        where W: Widget;

    /// Position the `Rectangle` used for scrolling `List`s with fixed `Item` sizes.
    fn position_scroll_trigger<W>(scroll_trigger: W, list: widget::Id) -> W
        where W: Widget;

    /// Calls the suitable `scroll_kids_<axis>` method on the `List`.
    fn scroll_list_kids<S>(list: List<Self, S>) -> List<Self, S>
        where Self: Sized,
              S: ItemSize;

    /// Size the widget given its breadth.
    fn size_breadth<W>(widget: W, breadth: Scalar) -> W
        where W: Widget;

    /// Size the widget given its length.
    fn size_length<W>(widget: W, length: Scalar) -> W
        where W: Widget;
}

/// The way in which the `List`'s items are sized. E.g. `Fired` or `Dynamic`.
pub trait ItemSize: Sized + Clone + Copy {

    /// Update the `List` widget.
    fn update_list<D>(List<D, Self>, widget::UpdateArgs<List<D, Self>>)
        -> <List<D, Self> as Widget>::Event
        where D: Direction;

    /// Set the size for the given item `widget` and return it.
    fn size_item<W, D>(&self, widget: W, breadth: Scalar) -> W
        where W: Widget,
              D: Direction;
}

/// Unique styling for the `List`.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// The width of the scrollbar if it is visible.
    #[conrod(default = "None")]
    pub scrollbar_thickness: Option<Option<Scalar>>,
    /// The color of the scrollbar if it is visible.
    #[conrod(default = "theme.border_color")]
    pub scrollbar_color: Option<Color>,
    /// The location of the `List`'s scrollbar.
    #[conrod(default = "None")]
    pub scrollbar_position: Option<Option<ScrollbarPosition>>,
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
pub struct Item<D, S> {
    /// The index of the item within the list.
    pub i: usize,
    /// The id generated for the widget.
    pub widget_id: widget::Id,
    /// The id used for the previous item's widget.
    pub last_id: Option<widget::Id>,
    breadth: Scalar,
    size: S,
    /// The id of the `scroll_trigger` rectangle, upon which this widget will be placed.
    scroll_trigger_id: widget::Id,
    /// The distance between the top of the first visible item and the top of the `scroll_trigger`
    /// `Rectangle`. This field is used for positioning the item's widget.
    first_item_margin: Scalar,
    /// The direction in which the items are laid out.
    direction: std::marker::PhantomData<D>,
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
pub struct Scrollbar<A> {
    widget: widget::Scrollbar<A>,
    id: widget::Id,
}

/// An `Iterator` yielding each `Item` in the list.
pub struct Items<D, S> {
    item_indices: std::ops::Range<usize>,
    next_item_indices_index: usize,
    list_id: widget::Id,
    last_id: Option<widget::Id>,
    scroll_trigger_id: widget::Id,
    first_item_margin: Scalar,
    item_breadth: Scalar,
    item_size: S,
    direction: std::marker::PhantomData<D>,
}


impl<D> List<D, Dynamic>
    where D: Direction,
{
    /// Begin building a new `List`.
    pub fn new(num_items: usize) -> Self {
        List::from_item_size(num_items, Dynamic {})
    }
}

impl List<Left, Dynamic> {
    /// Begin building a new `List` flowing from right to left.
    pub fn flow_left(num_items: usize) -> Self {
        List::new(num_items)
    }
}

impl List<Right, Dynamic> {
    /// Begin building a new `List` flowing from left to right.
    pub fn flow_right(num_items: usize) -> Self {
        List::new(num_items)
    }
}

impl List<Up, Dynamic> {
    /// Begin building a new `List` flowing from bottom to top.
    pub fn flow_up(num_items: usize) -> Self {
        List::new(num_items)
    }
}

impl List<Down, Dynamic> {
    /// Begin building a new `List` flowing from top to bottom.
    pub fn flow_down(num_items: usize) -> Self {
        List::new(num_items)
    }
}

impl<D, S> List<D, S>
    where D: Direction,
          S: ItemSize,
{
    /// Begin building a new `List` given some direction and item size.
    pub fn from_item_size(num_items: usize, item_size: S) -> Self {
        List {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            num_items: num_items,
            item_instantiation: ItemInstantiation::All,
            item_size: item_size,
            direction: std::marker::PhantomData,
        }.crop_kids()
    }

    /// Specify a fixed item size, where size is a `Scalar` in the direction that the `List` is
    /// flowing. When a `List` is constructed with this method, all items will have a fixed, equal
    /// length.
    pub fn item_size(self, length: Scalar) -> List<D, Fixed> {
        let List { common, style, num_items, .. } = self;
        List {
            common: common,
            style: style,
            num_items: num_items,
            item_instantiation: ItemInstantiation::OnlyVisible,
            item_size: Fixed { length: length },
            direction: std::marker::PhantomData,
        }
    }
}

impl<D> List<D, Fixed>
    where D: Direction,
{
    /// Indicates that an `Item` should be instantiated for every element in the list, regardless of
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
        self.item_instantiation = ItemInstantiation::All;
        self
    }

    /// Indicates that only `Item`s that are visible should be instantiated. This ensures that we
    /// avoid bloating the widget graph with unnecessary nodes and in turn keep traversal times to
    /// a minimum.
    ///
    /// This is the default behaviour for `List`s with fixed item sizes.
    pub fn instantiate_only_visible_items(mut self) -> Self {
        self.item_instantiation = ItemInstantiation::OnlyVisible;
        self
    }
}

impl<D, S> List<D, S>
    where D: Direction,
          S: ItemSize,
{
    /// Specifies that the `List` should be scrollable and should provide a `Scrollbar` to the
    /// right of the items.
    pub fn scrollbar_next_to(mut self) -> Self {
        self.style.scrollbar_position = Some(Some(ScrollbarPosition::NextTo));
        D::scroll_list_kids(self)
    }

    /// Specifies that the `List` should be scrollable and should provide a `Scrollbar` that hovers
    /// above the right edge of the items and automatically hides when the user is not scrolling.
    pub fn scrollbar_on_top(mut self) -> Self {
        self.style.scrollbar_position = Some(Some(ScrollbarPosition::OnTop));
        D::scroll_list_kids(self)
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

impl<D, S> Widget for List<D, S>
    where D: Direction,
          S: ItemSize,
{
    type State = State;
    type Style = Style;
    type Event = (Items<D, S>, Option<Scrollbar<D::Axis>>);

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        S::update_list(self, args)
    }
}


impl<D, S> Items<D, S>
    where D: Direction,
          S: ItemSize,
{

    /// Yield the next `Item` in the list.
    pub fn next(&mut self, ui: &Ui) -> Option<Item<D, S>> {
        let Items {
            ref mut item_indices,
            ref mut next_item_indices_index,
            ref mut last_id,
            list_id,
            scroll_trigger_id,
            first_item_margin,
            item_breadth,
            item_size,
            ..
        } = *self;

        // Retrieve the `node_index` that was generated for the next `Item`.
        let node_index = match ui.widget_graph().widget(list_id)
            .and_then(|container| container.unique_widget_state::<List<D, S>>())
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
                    breadth: item_breadth,
                    size: item_size,
                    first_item_margin: first_item_margin,
                    direction: std::marker::PhantomData,
                };
                *last_id = Some(node_index);
                Some(item)
            },
            _ => None,
        }
    }

}


impl<D, S> Item<D, S>
    where D: Direction,
          S: ItemSize,
{

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
        let Item {
            widget_id, last_id, breadth, size, scroll_trigger_id, first_item_margin, ..
        } = self;

        widget
            .and(|w| size.size_item::<W, D>(w, breadth))
            .and(|w| D::position_item(w, last_id, scroll_trigger_id, first_item_margin))
            .parent(scroll_trigger_id)
            .set(widget_id, ui)
    }

}

impl<S> Item<Down, S> {
    /// The width of the `Item`.
    pub fn width(&self) -> Scalar {
        self.breadth
    }
}

impl Item<Down, Fixed> {
    /// The height of the `Item`.
    pub fn height(&self) -> Scalar {
        self.size.length
    }
}

impl<S> Item<Up, S> {
    /// The width of the `Item`.
    pub fn width(&self) -> Scalar {
        self.breadth
    }
}

impl Item<Up, Fixed> {
    /// The height of the `Item`.
    pub fn height(&self) -> Scalar {
        self.size.length
    }
}

impl<S> Item<Right, S> {
    /// The height of the `Item`.
    pub fn height(&self) -> Scalar {
        self.breadth
    }
}

impl Item<Right, Fixed> {
    /// The width of the `Item`.
    pub fn width(&self) -> Scalar {
        self.size.length
    }
}

impl<S> Item<Left, S> {
    /// The height of the `Item`.
    pub fn height(&self) -> Scalar {
        self.breadth
    }
}

impl Item<Left, Fixed> {
    /// The width of the `Item`.
    pub fn width(&self) -> Scalar {
        self.size.length
    }
}


impl<A> Scrollbar<A>
    where A: widget::scrollbar::Axis,
{
    /// Set the `Scrollbar` within the given `Ui`.
    pub fn set(self, ui: &mut UiCell) {
        let Scrollbar { widget, id } = self;
        widget.set(id, ui);
    }
}


impl ItemSize for Fixed {

    fn update_list<D>(list: List<D, Self>, args: widget::UpdateArgs<List<D, Self>>)
        -> <List<D, Self> as Widget>::Event
        where D: Direction,
    {
        let widget::UpdateArgs { id, state, rect, prev, ui, style, .. } = args;
        let List { common, item_size, num_items, item_instantiation, .. } = list;

        // The following code is generic over the direction in which the `List` is laid out.
        //
        // To avoid using terms like `width` and `height`, the more general terms `length` and
        // `breadth` are used. For a `List` flowing downwards, `length` would refer to the item's
        // `height`, while `breadth` would refer to the item's `width`.

        // We need a positive item length in order to do anything useful.
        assert!(item_size.length > 0.0,
                "the given item height was {:?} however it must be > 0",
                item_size.length);

        let total_item_length = num_items as Scalar * item_size.length;
        let (list_range, list_perpendicular_range) = D::ranges(rect);
        let list_length = list_range.len();
        let list_breadth = list_perpendicular_range.len();

        // Determine whether or not the list is currently scrollable.
        let is_scrollable = D::common_scroll(&common).is_some() && total_item_length > list_length;

        // The width of the scrollbar.
        let scrollbar_thickness = style.scrollbar_thickness(&ui.theme)
            .unwrap_or_else(|| {
                ui.theme.widget_style::<widget::scrollbar::Style>()
                    .and_then(|style| style.style.thickness)
                    .unwrap_or(10.0)
            });

        let scrollbar_position = style.scrollbar_position(&ui.theme);
        let item_breadth = match (is_scrollable, scrollbar_position) {
            (true, Some(ScrollbarPosition::NextTo)) => list_breadth - scrollbar_thickness,
            _ => list_breadth,
        };

        // The widget used to scroll the `List`'s range.
        //
        // By using one long `Rectangle` widget to trigger the scrolling, this allows us to only
        // instantiate the visible items.
        {
            let scroll_trigger = widget::Rectangle::fill([0.0, 0.0]);
            let scroll_trigger = D::position_scroll_trigger(scroll_trigger, id);
            let scroll_trigger = D::size_breadth(scroll_trigger, list_breadth);
            let scroll_trigger = D::size_length(scroll_trigger, total_item_length);
            scroll_trigger.color(color::TRANSPARENT).parent(id).set(state.ids.scroll_trigger, ui);
        }

        // Determine the index range of the items that should be instantiated.
        let (item_idx_range, first_item_margin) = match item_instantiation {
            ItemInstantiation::All => {
                let range = 0..num_items;
                let margin = 0.0;
                (range, margin)
            },
            ItemInstantiation::OnlyVisible => {
                let scroll_trigger_rect = ui.rect_of(state.ids.scroll_trigger).unwrap();
                let (scroll_trigger_range, _) = D::ranges(scroll_trigger_rect);

                let hidden_range_length = (scroll_trigger_range.start - list_range.start).abs();
                let num_start_hidden_items = hidden_range_length / item_size.length;
                let num_visible_items = list_length / item_size.length;
                let first_visible_item_idx = num_start_hidden_items.floor() as usize;
                let end_visible_item_idx = std::cmp::min(
                    (num_start_hidden_items + num_visible_items).ceil() as usize,
                    num_items,
                );

                let range = first_visible_item_idx..end_visible_item_idx;
                let margin = first_visible_item_idx as Scalar * item_size.length;
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
            item_breadth: item_breadth,
            direction: std::marker::PhantomData,
            first_item_margin: first_item_margin,
            item_size: item_size,
        };

        // Instantiate the `Scrollbar` only if necessary.
        let auto_hide = match (is_scrollable, scrollbar_position) {
            (false, _) | (true, None) => return (items, None),
            (_, Some(ScrollbarPosition::NextTo)) => false,
            (_, Some(ScrollbarPosition::OnTop)) => true,
        };
        let scrollbar_color = style.scrollbar_color(&ui.theme);
        let scrollbar = D::scrollbar(id)
            .and_if(prev.maybe_floating.is_some(), |s| s.floating(true))
            .color(scrollbar_color)
            .thickness(scrollbar_thickness)
            .auto_hide(auto_hide);
        let scrollbar = Scrollbar {
            widget: scrollbar,
            id: state.ids.scrollbar,
        };

        (items, Some(scrollbar))
    }

    fn size_item<W, D>(&self, widget: W, breadth: Scalar) -> W
        where W: Widget,
              D: Direction,
    {
        let widget = D::size_breadth(widget, breadth);
        D::size_length(widget, self.length)
    }

}

impl ItemSize for Dynamic {

    fn update_list<D>(list: List<D, Self>, args: widget::UpdateArgs<List<D, Self>>)
        -> <List<D, Self> as Widget>::Event
        where D: Direction,
    {
        let widget::UpdateArgs { id, state, rect, prev, ui, style, .. } = args;
        let List { item_size, num_items, .. } = list;

        let (_, list_perpendicular_range) = D::ranges(rect);
        let list_breadth = list_perpendicular_range.len();

        // Always instantiate all items for a list with dynamically sized items.
        let item_idx_range = 0..num_items;
        let first_item_margin = 0.0;

        // Ensure there are at least as many indices as there are visible items.
        if state.ids.items.len() < item_idx_range.len() {
            let id_gen = &mut ui.widget_id_generator();
            state.update(|state| state.ids.items.resize(item_idx_range.len(), id_gen));
        }

        // The width of the scrollbar.
        let scrollbar_thickness = style.scrollbar_thickness(&ui.theme)
            .unwrap_or_else(|| {
                ui.theme.widget_style::<widget::scrollbar::Style>()
                    .and_then(|style| style.style.thickness)
                    .unwrap_or(10.0)
            });

        let scrollbar_position = style.scrollbar_position(&ui.theme);
        let item_breadth = match scrollbar_position {
            Some(ScrollbarPosition::NextTo) => list_breadth - scrollbar_thickness,
            _ => list_breadth,
        };

        let items = Items {
            list_id: id,
            item_indices: item_idx_range,
            next_item_indices_index: 0,
            last_id: None,
            scroll_trigger_id: id,
            item_breadth: item_breadth,
            direction: std::marker::PhantomData,
            first_item_margin: first_item_margin,
            item_size: item_size,
        };

        // Instantiate the `Scrollbar` only if necessary.
        let auto_hide = match scrollbar_position {
            None => return (items, None),
            Some(ScrollbarPosition::NextTo) => false,
            Some(ScrollbarPosition::OnTop) => true,
        };
        let scrollbar_color = style.scrollbar_color(&ui.theme);
        let scrollbar = D::scrollbar(id)
            .and_if(prev.maybe_floating.is_some(), |s| s.floating(true))
            .color(scrollbar_color)
            .thickness(scrollbar_thickness)
            .auto_hide(auto_hide);
        let scrollbar = Scrollbar {
            widget: scrollbar,
            id: state.ids.scrollbar,
        };

        (items, Some(scrollbar))
    }

    fn size_item<W, D>(&self, widget: W, breadth: Scalar) -> W
        where W: Widget,
              D: Direction,
    {
        D::size_breadth(widget, breadth)
    }

}


impl Direction for Down {
    type Axis = widget::scroll::Y;

    fn ranges(Rect { x, y }: Rect) -> (Range, Range) {
        (y.invert(), x)
    }

    fn scrollbar(id: widget::Id) -> widget::Scrollbar<Self::Axis> {
        widget::Scrollbar::y_axis(id)
    }

    fn common_scroll(common: &widget::CommonBuilder) -> Option<&widget::scroll::Scroll> {
        common.maybe_y_scroll.as_ref()
    }

    fn scroll_list_kids<S>(list: List<Self, S>) -> List<Self, S>
        where Self: Sized,
              S: ItemSize,
    {
        list.scroll_kids_vertically()
    }

    fn position_item<W>(widget: W,
                        last_id: Option<widget::Id>,
                        scroll_trigger_id: widget::Id,
                        first_item_margin: Scalar) -> W
        where W: Widget,
    {
        match last_id {
            None => widget.mid_top_with_margin_on(scroll_trigger_id, first_item_margin)
                .align_left_of(scroll_trigger_id),
            Some(id) => widget.down_from(id, 0.0),
        }
    }

    fn position_scroll_trigger<W>(scroll_trigger: W, list: widget::Id) -> W
        where W: Widget
    {
        scroll_trigger.mid_top_of(list)
    }

    fn size_breadth<W>(widget: W, breadth: Scalar) -> W
        where W: Widget
    {
        widget.w(breadth)
    }

    fn size_length<W>(widget: W, length: Scalar) -> W
        where W: Widget
    {
        widget.h(length)
    }

}

impl Direction for Up {
    type Axis = widget::scroll::Y;

    fn ranges(Rect { x, y }: Rect) -> (Range, Range) {
        (y, x)
    }

    fn scrollbar(id: widget::Id) -> widget::Scrollbar<Self::Axis> {
        widget::Scrollbar::y_axis(id)
    }

    fn common_scroll(common: &widget::CommonBuilder) -> Option<&widget::scroll::Scroll> {
        common.maybe_y_scroll.as_ref()
    }

    fn scroll_list_kids<S>(list: List<Self, S>) -> List<Self, S>
        where Self: Sized,
              S: ItemSize,
    {
        list.scroll_kids_vertically()
    }

    fn position_item<W>(widget: W,
                        last_id: Option<widget::Id>,
                        scroll_trigger_id: widget::Id,
                        first_item_margin: Scalar) -> W
        where W: Widget,
    {
        match last_id {
            None => widget.mid_bottom_with_margin_on(scroll_trigger_id, first_item_margin)
                .align_left_of(scroll_trigger_id),
            Some(id) => widget.up_from(id, 0.0),
        }
    }

    fn position_scroll_trigger<W>(scroll_trigger: W, list: widget::Id) -> W
        where W: Widget
    {
        scroll_trigger.mid_bottom_of(list)
    }

    fn size_breadth<W>(widget: W, breadth: Scalar) -> W
        where W: Widget
    {
        widget.w(breadth)
    }

    fn size_length<W>(widget: W, length: Scalar) -> W
        where W: Widget
    {
        widget.h(length)
    }

}

impl Direction for Left {
    type Axis = widget::scroll::X;

    fn ranges(Rect { x, y }: Rect) -> (Range, Range) {
        (x.invert(), y)
    }

    fn scrollbar(id: widget::Id) -> widget::Scrollbar<Self::Axis> {
        widget::Scrollbar::x_axis(id)
    }

    fn common_scroll(common: &widget::CommonBuilder) -> Option<&widget::scroll::Scroll> {
        common.maybe_x_scroll.as_ref()
    }

    fn scroll_list_kids<S>(list: List<Self, S>) -> List<Self, S>
        where Self: Sized,
              S: ItemSize,
    {
        list.scroll_kids_horizontally()
    }

    fn position_item<W>(widget: W,
                        last_id: Option<widget::Id>,
                        scroll_trigger_id: widget::Id,
                        first_item_margin: Scalar) -> W
        where W: Widget,
    {
        match last_id {
            None => widget.mid_right_with_margin_on(scroll_trigger_id, first_item_margin)
                .align_top_of(scroll_trigger_id),
            Some(id) => widget.left_from(id, 0.0),
        }
    }

    fn position_scroll_trigger<W>(scroll_trigger: W, list: widget::Id) -> W
        where W: Widget
    {
        scroll_trigger.mid_right_of(list)
    }

    fn size_breadth<W>(widget: W, breadth: Scalar) -> W
        where W: Widget
    {
        widget.h(breadth)
    }

    fn size_length<W>(widget: W, length: Scalar) -> W
        where W: Widget
    {
        widget.w(length)
    }

}

impl Direction for Right {
    type Axis = widget::scroll::X;

    fn ranges(Rect { x, y }: Rect) -> (Range, Range) {
        (x, y)
    }

    fn scrollbar(id: widget::Id) -> widget::Scrollbar<Self::Axis> {
        widget::Scrollbar::x_axis(id)
    }

    fn common_scroll(common: &widget::CommonBuilder) -> Option<&widget::scroll::Scroll> {
        common.maybe_x_scroll.as_ref()
    }

    fn scroll_list_kids<S>(list: List<Self, S>) -> List<Self, S>
        where Self: Sized,
              S: ItemSize,
    {
        list.scroll_kids_horizontally()
    }

    fn position_item<W>(widget: W,
                        last_id: Option<widget::Id>,
                        scroll_trigger_id: widget::Id,
                        first_item_margin: Scalar) -> W
        where W: Widget,
    {
        match last_id {
            None => widget.mid_left_with_margin_on(scroll_trigger_id, first_item_margin)
                .align_top_of(scroll_trigger_id),
            Some(id) => widget.right_from(id, 0.0),
        }
    }

    fn position_scroll_trigger<W>(scroll_trigger: W, list: widget::Id) -> W
        where W: Widget
    {
        scroll_trigger.mid_left_of(list)
    }

    fn size_breadth<W>(widget: W, breadth: Scalar) -> W
        where W: Widget
    {
        widget.h(breadth)
    }

    fn size_length<W>(widget: W, length: Scalar) -> W
        where W: Widget
    {
        widget.w(length)
    }

}
