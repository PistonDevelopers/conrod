use {
    Backend,
    Color,
    IndexSlot,
    NodeIndex,
    Rectangle,
    Scalar,
};
use widget;

/// A helper widget, useful for instantiating a sequence of widgets in a vertical list.
pub struct List<F> {
    common: widget::CommonBuilder,
    style: Style,
    item_h: Scalar,
    num_items: u32,
    maybe_item: Option<F>,
}

/// Unique kind for the widget.
pub const KIND: widget::Kind = "List";

widget_style! {
    KIND;
    /// Unique styling for the Widget.
    style Style {
        /// Color of the List's area.
        - color: Color { theme.shape_color }
    }
}

/// Represents the state of the List widget.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    scroll_trigger_idx: IndexSlot,
    item_indices: Vec<NodeIndex>,
}

/// The data necessary for instantiating a single item within a `List`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Item {
    /// The index of the item within the list.
    pub list_idx: usize,
    /// The index generated for 
    pub widget_idx: NodeIndex,
}

impl<F> List<F> {

    /// Create a List context to be built upon.
    pub fn new(num_items: u32, item_height: Scalar) -> Self {
        List {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
            maybe_item: None,
        }.crop_kids()
    }

    /// A function used to instantiate each item in the list.
    ///
    pub fn item(mut self, f: F) -> Self
        where F: FnMut(Item),
    {
        self.item = Some(f);
        self
    }

}

impl Item {

    /// Sets the position and size for the `Widget` used in this `List` `Item`'s position.
    pub fn layout<W>(&self, widget: W) -> W
        where W: Widget,
    {
        widget
    }

}

impl<F> Widget for List<F>
    where F: FnMut(Item),
{
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> widget::Kind {
        KIND
    }

    fn init_state(&self) -> State {
        State {
            scroll_trigger_idx: IndexSlot::new(),
            item_indices: Vec::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the List.
    fn update<B: Backend>(self, args: widget::UpdateArgs<Self, B>) {
        let widget::UpdateArgs { idx, state, style, rect, mut ui, .. } = args;
        let List { mut maybe_item, item_h, num_items, .. } = self;

        let color = style.color(&ui.theme);
        let total_item_h = num_items as Scalar * item_h;

        // The widget used to scroll the `List`'s range.
        //
        // By using one long `Rectangle` widget to trigger the scrolling, this allows us to only
        // instantiate the visible items.
        let scroll_trigger_idx = state.scroll_trigger_idx.get(&mut ui);
        Rectangle::fill([rect.w(), item_h])
            .color(color)
            .parent(idx)
            .set(scroll_trigger_idx, &mut ui);

        let scroll_trigger_rect = ui.rect_of(scroll_triggeer_idx).unwrap();
        let num_visible_items = (rect.h() / total_item_h * num_items as Scalar) as usize;
        let hidden_range_length = scroll_trigger_rect.h() - rect.h();
        let num_top_hidden_items = hidden_range_length / num_items as Scalar;

        let first_visible_item_idx = num_top_hidden_items.ceil() as usize;
        let first_visible_item_margin = first_visible_item_idx as conrod::Scalar * item_h;
        let end_of_visible_idx_range = first_visible_item_idx + num_visible_items;
        let visible_idx_range = first_visible_item_idx..end_of_visible_idx_range;

        // Ensure there are at least as many indices as there are visible items.
        let num_indices = state.item_indices.len();
        if num_indices < num_visible_items {
            state.update(|state| {
                let extension = (num_indices..num_visible_items)
                    .map(|_| ui.new_unique_node_index());
                state.item_indices.extend(extension);
            });
        }

        let mut item_fn = match maybe_item {
            Some(f) => f,
            None => return,
        };

        let iter = visible_idx_range.zip(state.scale_indices.iter());
        for (i, &node_index) in iter {

            let item = Item {
                idx: idx,
                first_visible_item_margin: first_visible_item_margin,
            };

            item_fn(item, &mut ui);
        }
    }
}
