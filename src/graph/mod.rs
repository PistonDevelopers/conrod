

use Scalar;
use daggy::{self, Walker};
use elmesque::Element;
use elmesque::element::layers;
use position::{Depth, Point, Rect};
use self::index_map::IndexMap;
use std::any::Any;
use std::fmt::Debug;
use widget::{self, Widget};


pub use self::index_map::GraphIndex;

mod index_map;


/// An alias for our Graph's Node Index.
pub type NodeIndex = daggy::NodeIndex<u32>;

/// An alias for our Graph's Edge Index.
type EdgeIndex = daggy::EdgeIndex<u32>;

/// An alias for a tuple containing an associated `Edge/NodeIndex` pair.
type IndexPair = (EdgeIndex, NodeIndex);

type Parents = daggy::Parents<Node, Edge, u32>;
type Children = daggy::Children<Node, Edge, u32>;

/// An alias for our Graph's recursive walker.
type RecursiveWalk<F> = daggy::RecursiveWalk<Node, Edge, u32, F>;

/// The state type that we'll dynamically cast to and from `Any` for storage within the cache.
#[derive(Debug)]
pub struct UniqueWidgetState<State, Style> where
    State: Any + Debug,
    Style: Any + Debug,
{
    pub state: State,
    pub style: Style,
}

/// A container for caching a Widget's state inside a Graph Node.
#[derive(Debug)]
pub struct Container {
    /// Dynamically stored widget state.
    pub maybe_state: Option<Box<Any>>,
    /// A unique widget kind identifier.
    pub kind: &'static str,
    /// The rectangle describing the Widget's area.
    pub rect: Rect,
    /// The depth at which the widget will be rendered comparatively to its siblings.
    pub depth: Depth,
    /// The drag state of the Widget.
    pub drag_state: widget::drag::State,
    /// The area in which child widgets are placed.
    pub kid_area: widget::KidArea,
    /// Whether or not the widget is a "Floating" widget.
    ///
    /// See the `Widget::float` docs for an explanation of what this means.
    pub maybe_floating: Option<widget::Floating>,
    /// Scroll related state (is only `Some` if the widget is scrollable)..
    pub maybe_scrolling: Option<widget::scroll::State>,
    /// Whether or not the `Element` for the widget has changed since the last time an `Element`
    /// was requested from the graph.
    pub element_has_changed: bool,
    /// The latest `Element` that has been used for drawing the `Widget`.
    pub maybe_element: Option<Element>,
    /// Whether or not the `Widget`'s cache has been updated since the last update cycle.
    ///
    /// We need to keep track of this as we only want to draw the widget if it has been set.
    pub is_updated: bool,
    /// Whether or not the `Widget`'s cache has was updated during the last update cycle.
    ///
    /// We need to know this so we can check whether or not a widget has been removed.
    pub was_previously_updated: bool,
    /// Whether or not the widget is included when picking widgets by position.
    pub picking_passthrough: bool,
}

/// A node within the UI Graph.
#[derive(Debug)]
enum Node {
    /// The root node and starting point for rendering.
    Root,
    /// A widget constructed by a user.
    Widget(Container),
    /// A placeholder node - used in the case that a child is added to the graph before its parent,
    /// this node is used as a "placeholder parent" until the actual parent is added to the graph.
    Placeholder,
}

/// An edge between nodes within the UI Graph.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Edge {
    /// Describes the relative positioning of widgets.
    ///
    /// When adding an edge *a -> b*, *b* is positioned relatively to *a*.
    Position,
    /// Describes the rendering order of the widgets.
    ///
    /// When adding an edge *a -> b*, *a* is the parent of (and will be rendered before) *b*.
    Depth,
    /// Describes when a widget is used solely as a graphical element for another widget.
    ///
    /// When adding an edge *a -> b*, *b* is considered to be a graphical element of *a*. This
    /// implies several things about *b*:
    ///
    /// - If *b* is picked within either **Graph::pick_widget** or
    /// **Graph::pick_top_scrollable_widget**, it will instead return the index for *a*.
    /// - When determining the **Graph::scroll_offset** for *b*, *a*'s scrolling (if it is
    /// scrollable, that is) will be skipped.
    /// - Any **Graphic** child of *b* will be considered as a **Graphic** child of *a*.
    Graphic,
}

/// An alias for the petgraph::Graph used within our Ui Graph.
type Dag = daggy::Dag<Node, Edge>;

/// Parts of the graph that are significant when visiting and sorting by depth.
/// The reason a widget and its scrollbar are separate here is because a widget's scrollbar may
/// sometimes appear on *top* of the widget's children.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Visitable {
    /// The index of some widget in the graph.
    Widget(NodeIndex),
    /// The scrollbar for the widget at the given NodeIndex.
    Scrollbar(NodeIndex),
}

/// Stores the dynamic state of a UI tree of Widgets.
#[derive(Debug)]
pub struct Graph {
    /// Cached widget state in a directed acyclic graph whose edges describe the rendering tree and
    /// positioning.
    dag: Dag,
    /// A map of the UiId to the graph's indices.
    index_map: IndexMap,
    /// The NodeIndex of the root Node.
    root: NodeIndex,
    /// Contains Node indices in order of depth, starting with the deepest.
    ///
    /// This is updated at the beginning of the `Graph::draw` method.
    depth_order: Vec<Visitable>,
    /// Used for storing indices of "floating" widgets during depth sorting so that they may be
    /// visited after widgets of the root tree.
    floating_deque: Vec<NodeIndex>,
}


/// A common argument when expecting that there is a `NodeIndex`.
const NO_MATCHING_NODE_INDEX: &'static str = "No matching NodeIndex";
/// A common argument when expecting that there is a `WidgetId`.
const NO_MATCHING_WIDGET_ID: &'static str = "No matching WidgetId";


impl Container {

    /// A method for taking only the unique state from the container.
    pub fn take_unique_widget_state<W>(&mut self)
        -> Option<Box<UniqueWidgetState<W::State, W::Style>>> where
        W: Widget,
        W::State: Any + 'static,
        W::Style: Any + 'static,
    {
        self.maybe_state.take().map(|any_state| {
            any_state.downcast().ok()
                .expect("Failed to downcast from `Box<Any>` to the required UniqueWidgetState")
        })
    }

    /// Take the widget state from the container and cast it to type W.
    pub fn take_widget_state<W>(&mut self) -> Option<widget::Cached<W>> where
        W: Widget,
        W::State: Any + 'static,
        W::Style: Any + 'static,
    {
        if self.maybe_state.is_some() {
            let boxed_unique_state = self.take_unique_widget_state::<W>().unwrap();
            let unique_state: UniqueWidgetState<W::State, W::Style> = *boxed_unique_state;
            let UniqueWidgetState { state, style } = unique_state;
            Some(widget::Cached {
                state: state,
                style: style,
                rect: self.rect,
                depth: self.depth,
                drag_state: self.drag_state,
                kid_area: self.kid_area,
                maybe_floating: self.maybe_floating,
                maybe_scrolling: self.maybe_scrolling,
            })
        } else {
            None
        }
    }

}


impl Graph {

    /// Construct a new Graph with the given capacity.
    pub fn with_capacity(capacity: usize) -> Graph {
        let mut dag = Dag::with_capacity(capacity, capacity);
        let root = dag.add_node(Node::Root);
        Graph {
            dag: dag,
            index_map: IndexMap::with_capacity(capacity),
            root: root,
            depth_order: Vec::with_capacity(capacity),
            floating_deque: Vec::with_capacity(capacity),
        }
    }
    
    /// Add a new placeholder node and return it's `NodeIndex` into the `Graph`.
    ///
    /// This method is used by the `widget::set_widget` function when some internal widget does not
    /// yet have it's own `NodeIndex`.
    pub fn add_placeholder(&mut self) -> NodeIndex {
        self.dag.add_node(Node::Placeholder)
    }

    /// If there is a Widget for the given index, return a reference to it.
    pub fn widget<I: GraphIndex>(&self, idx: I) -> Option<&Container> {
        let Graph { ref index_map, ref dag, .. } = *self;
        idx.to_node_index(index_map).and_then(|idx| match &dag[idx] {
            &Node::Widget(ref container) => Some(container),
            _ => None,
        })
    }

    /// If there is a Widget for the given Id, return a mutable reference to it.
    pub fn widget_mut<I: GraphIndex>(&mut self, idx: I) -> Option<&mut Container> {
        let Graph { ref index_map, ref mut dag, .. } = *self;
        idx.to_node_index(index_map).and_then(move |idx| match &mut dag[idx] {
            &mut Node::Widget(ref mut container) => Some(container),
            _ => None,
        })
    }

    /// Converts the given **GraphIndex** into an index of type **J**.
    ///
    /// Returns `None` if there is no direct mapping from the given index to an index of type
    /// **J**.
    pub fn convert_idx<I, J>(&self, idx: I) -> Option<J>
        where I: GraphIndex,
              J: GraphIndex,
    {
        J::from_idx(idx, &self.index_map)
    }

    /// Get the **NodeIndex** for the given **GraphIndex**.
    pub fn node_index<I: GraphIndex>(&self, idx: I) -> Option<NodeIndex> {
        self.convert_idx(idx)
    }

    /// Get the **WidgetId** for the given **GraphIndex**.
    pub fn widget_id<I: GraphIndex>(&self, idx: I) -> Option<widget::Id> {
        self.convert_idx(idx)
    }

    /// If the widget at the given index has some parent along an **Edge** of the given variant,
    /// return an index to it.
    pub fn parent_of<I, J=NodeIndex>(&self, idx: I, edge: Edge) -> Option<J>
        where I: GraphIndex,
              J: GraphIndex,
    {
        self.node_index(idx).and_then(|idx| {
            maybe_parent_edge(&self.dag, idx, edge)
                .and_then(|(_, parent_idx)| J::from_idx(parent_idx, &self.index_map))
        })
    }

    /// Return the index of the parent along the given widget's **Depth** **Edge**.
    pub fn depth_parent_of<I, J=NodeIndex>(&self, idx: I) -> Option<J> where
        I: GraphIndex,
        J: GraphIndex,
    {
        self.parent_of(idx, Edge::Depth)
    }

    /// Return the index of the parent along the given widget's **Depth** **Edge**.
    pub fn position_parent_of<I, J=NodeIndex>(&self, idx: I) -> Option<J> where
        I: GraphIndex,
        J: GraphIndex,
    {
        self.parent_of(idx, Edge::Position)
    }

    /// Return the index of the parent along the given widget's **Depth** **Edge**.
    pub fn graphic_parent_of<I, J=NodeIndex>(&self, idx: I) -> Option<J> where
        I: GraphIndex,
        J: GraphIndex,
    {
        self.parent_of(idx, Edge::Graphic)
    }

    /// If the given Point is currently on a Widget, return an index to that widget.
    pub fn pick_widget<I: GraphIndex>(&self, xy: Point) -> Option<I> {
        let Graph { ref depth_order, ref dag, ref index_map, .. } = *self;
        depth_order.iter().rev()
            .find(|&&visitable| {
                match visitable {
                    Visitable::Widget(idx) => if let Some(visible_rect) = self.visible_area(idx) {
                        if visible_rect.is_over(xy) {
                            return true
                        }
                    },
                    Visitable::Scrollbar(idx) => {
                        if let Some(&Node::Widget(ref container)) = dag.node_weight(idx) {
                            if let Some(ref scrolling) = container.maybe_scrolling {
                                if scrolling.is_over(xy) {
                                    return true;
                                }
                            }
                        }
                    },
                }
                false
            })
            // Extract the node index from the `Visitable`.
            .map(|&visitable| {
                let node_idx = match visitable {
                    // Ensure that if we've picked some widget that is a **Graphic** child of some other
                    // widget, we return the **Graphic** parent.
                    Visitable::Widget(idx) =>
                        parent_graphic_recursion(dag, idx).last_node(dag).unwrap_or(idx),
                    Visitable::Scrollbar(idx) => idx,
                };
                I::from_idx(node_idx, index_map).expect(NO_MATCHING_NODE_INDEX)
            })
    }


    /// If the given Point is currently over a scrollable widget, return an index to that widget.
    pub fn pick_top_scrollable_widget<I: GraphIndex>(&self, xy: Point) -> Option<I> {
        let Graph { ref depth_order, ref dag, ref index_map, .. } = *self;
        depth_order.iter().rev()
            .filter_map(|&visitable| match visitable {
                Visitable::Widget(idx) => Some(idx),
                Visitable::Scrollbar(_) => None,
            })
            .find(|&idx| {
                if let Some(&Node::Widget(ref container)) = dag.node_weight(idx) {
                    if container.maybe_scrolling.is_some() {
                        if container.rect.is_over(xy) {
                            return true;
                        }
                    }
                }
                false
            })
            .map(|idx| I::from_idx(idx, index_map).expect(NO_MATCHING_NODE_INDEX))
    }


    /// Calculate the total scroll offset for the widget with the given widget::Index.
    pub fn scroll_offset<I: GraphIndex>(&self, idx: I) -> Point {
        let Graph { ref dag, ref index_map, .. } = *self;

        let mut offset = [0.0, 0.0];
        let mut idx = match idx.to_node_index(index_map) {
            Some(idx) => idx,
            // If the ID is not yet present within the graph, just return the zeroed offset.
            None => return offset,
        };

        // Check the widget at the given index for any scroll offset and add it to our total offset.
        let add_to_offset = |offset: &mut [Scalar; 2], idx: NodeIndex| {
            if let Some(&Node::Widget(ref container)) = dag.node_weight(idx) {
                if let Some(ref scrolling) = container.maybe_scrolling {
                    let scroll_offset = scrolling.kids_pos_offset();
                    offset[0] += scroll_offset[0].round();
                    offset[1] += scroll_offset[1].round();
                }
            }
        };

        // We need to calculate the offset by summing the offset of each scrollable parent, while
        // considering relative positioning and graphic parents.
        let mut depth_parents = parent_depth_recursion(dag, idx);
        while let Some(depth_parent_idx) = depth_parents.next_node(dag) {

            // If the widget has some parent graphic edge, we may need to skip its offset.
            // This is because **Graphic** edges describe that the child widget is a graphical
            // element of the parent widget, so despite being instantiated at a higher depth, we
            // should consider its offset equal to the graphic parent.
            if let Some(graphic_parent_idx) = maybe_parent_graphic_edge(dag, idx).map(|(_, n)| n) {
                match maybe_parent_position_edge(dag, idx).map(|(_, n)| n) {

                    // If we have a graphic parent but no position parent, we should skip straight
                    // to the graphic parent.
                    None => {
                        idx = graphic_parent_idx;
                        depth_parents = parent_depth_recursion(dag, idx);
                        continue;
                    },

                    // Otherwise, we need to consider the given position parent.
                    Some(position_parent_idx) => {

                        // If our position parent is equal to or some child of our graphic parent,
                        // we don't need any offset.
                        // TODO: Review this part of the algorithm.
                        if position_parent_idx == graphic_parent_idx
                        || is_depth_parent_of(position_parent_idx, dag, graphic_parent_idx) {
                            return offset;
                        }
                    },

                }
            }

            // Recursively check all nodes with incoming `Position` edges for a parent that
            // matches our own parent. If any match, then we don't need to calculate any
            // additional offset as the widget we are being positioned relatively to has
            // already applied the necessary scroll offset.
            let mut position_parents = parent_position_recursion(dag, idx);
            while let Some(position_parent_idx) = position_parents.next_node(dag) {

                // If our parent depth edge is also the parent position edge, we'll add
                // offset for that parent before finishing.
                if depth_parent_idx == position_parent_idx {
                    add_to_offset(&mut offset, depth_parent_idx);
                    return offset;
                }

                // If our position_parent has some depth parent or grandparent that matches our
                // current depth_parent_idx, we can assume that the scroll offset for our widget
                // has already been calculated by this position_parent.
                if is_depth_parent_of(position_parent_idx, dag, depth_parent_idx) {
                    return offset;
                }
            }

            add_to_offset(&mut offset, depth_parent_idx);

            // Set the parent as the new current idx and continue traversing.
            idx = depth_parent_idx;
        }

        offset
    }


    /// Set the parent for the given Widget Id.
    /// This method clears all other incoming edges and ensures that the widget only has a single
    /// parent (incoming edge). This means we can be sure of retaining a tree structure.
    fn set_depth_parent<I, P>(&mut self, idx: I, maybe_parent_idx: Option<P>) where
        I: GraphIndex,
        P: GraphIndex,
    {
        let Graph { ref mut dag, ref mut index_map, root, .. } = *self;

        let node_idx = idx.to_node_index(index_map).expect(NO_MATCHING_NODE_INDEX);
        // If no parent id was given, we will set the root as the parent.
        let parent_node_idx = match maybe_parent_idx {
            Some(parent_idx) => match parent_idx.to_node_index(index_map) {
                Some(parent_node_idx) => parent_node_idx,
                // Add a temporary node to the graph at the given parent index so that we can add
                // the edge even in the parent widget's absense. The temporary node should be
                // replaced by the proper widget when it is updated later in the cycle.
                None => {
                    // We *know* that this must be a WidgetId as `.to_node_indx` returned None.
                    let parent_widget_id = parent_idx.to_widget_id(index_map)
                        .expect(NO_MATCHING_WIDGET_ID);
                    // Add a placeholder node to act as a parent until the actual parent is placed.
                    let parent_node_idx = dag.add_node(Node::Placeholder);
                    index_map.insert(parent_widget_id, parent_node_idx);
                    parent_node_idx
                },
            },
            None => root,
        };

        set_edge(dag, parent_node_idx, node_idx, Edge::Depth);
    }


    /// Set's an edge from *a* to *b*.
    ///
    /// **Panics** if either of the given `GraphIndex` types don't yet have an associated
    /// `NodeIndex` within the **Graph**.
    fn set_edge<A, B>(&mut self, a: A, b: B, edge: Edge)
        where A: GraphIndex,
              B: GraphIndex,
    {
        let a_idx = a.to_node_index(&self.index_map).expect(NO_MATCHING_NODE_INDEX);
        let b_idx = b.to_node_index(&self.index_map).expect(NO_MATCHING_NODE_INDEX);
        set_edge(&mut self.dag, a_idx, b_idx, edge);
    }


    /// The Rect that bounds the kids of the widget with the given ID.
    pub fn kids_bounding_box<I: GraphIndex>(&self, idx: I) -> Option<Rect> {
        idx.to_node_index(&self.index_map)
            .and_then(|idx| bounding_box(self, false, None, true, idx, None))
    }


    /// The rectangle that represents the maximum fully visible area for the widget with the given
    /// index, including considering cropped scroll area.
    ///
    /// Otherwise, return None if the widget is hidden.
    pub fn visible_area<I: GraphIndex>(&self, idx: I) -> Option<Rect> {
        idx.to_node_index(&self.index_map)
            .and_then(|idx| visible_area_within_depth(self, idx, None))
    }

    /// Add a widget to the Graph.
    ///
    /// If a WidgetId is given, create a mapping within the index_map.
    ///
    /// Set the parent of the new widget with the given parent index (or `root` if no parent
    /// index is given). Return the NodeIndex for the Widget's position within the Graph.
    pub fn add_widget<I: GraphIndex>(&mut self,
                                     container: Container,
                                     maybe_widget_id: Option<widget::Id>,
                                     maybe_depth_parent_idx: Option<I>) -> NodeIndex
    {
        let node_idx = self.dag.add_node(Node::Widget(container));
        if let Some(id) = maybe_widget_id {
            self.index_map.insert(id, node_idx);
        }
        self.set_depth_parent(node_idx, maybe_depth_parent_idx);
        node_idx
    }


    /// Cache some `PreUpdateCache` widget data into the graph.
    ///
    /// This is called (via the `ui` module) from within the `widget::set_widget` function prior to
    /// the `Widget::update` method being called.
    ///
    /// This is done so that if this Widget were to internally `set` some other `Widget`s within
    /// its own `update` method, this `Widget`s positioning and dimension data already exists
    /// within the `Graph` for reference.
    pub fn pre_update_cache(&mut self, widget: widget::PreUpdateCache) {
        let widget::PreUpdateCache {
            kind, idx, maybe_parent_idx, maybe_positioned_relatively_idx, rect, depth, kid_area,
            drag_state, maybe_floating, maybe_scrolling, picking_passthrough, maybe_graphics_for,
        } = widget;

        // Construct a new `Container` to place in the `Graph`.
        let new_container = || Container {
            maybe_state: None,
            kind: kind,
            rect: rect,
            depth: depth,
            drag_state: drag_state,
            kid_area: kid_area,
            maybe_floating: maybe_floating,
            maybe_scrolling: maybe_scrolling,
            picking_passthrough: picking_passthrough,
            maybe_element: None,
            element_has_changed: false,
            is_updated: true,
            was_previously_updated: false,
        };

        // If we already have a `Node` in the graph for the given `idx`, we need to update it.
        if let Some(node_idx) = idx.to_node_index(&self.index_map) {

            // Ensure that we have an `Edge::Depth` in the graph representing the parent.
            self.set_depth_parent(idx, maybe_parent_idx);

            match &mut self.dag[node_idx] {

                // If the node is currently a `Placeholder`, construct a new container and use this
                // to set it as the `Widget` variant.
                node @ &mut Node::Placeholder => *node = Node::Widget(new_container()),

                // Otherwise, update the data in the container that already exists.
                &mut Node::Widget(ref mut container) => {

                    // If the container already exists with the state of some other kind of
                    // widget, we can assume there's been a mistake with the given Id.
                    // TODO: It might be overkill to panic here.
                    if container.kind != kind && container.kind != "EMPTY" {
                        panic!("A widget of a different kind already exists at the given idx \
                                ({:?}). You tried to insert a {:?}, however the existing \
                                widget is a {:?}. Check your `WidgetId`s for errors.",
                                idx, &kind, container.kind);
                    }

                    container.kind = kind;
                    container.rect = rect;
                    container.depth = depth;
                    container.drag_state = drag_state;
                    container.kid_area = kid_area;
                    container.maybe_floating = maybe_floating;
                    container.maybe_scrolling = maybe_scrolling;
                    container.is_updated = true;
                },

                // The node that we're updating should only be either a `Placeholder` or a `Widget`.
                _ => unreachable!(),
            }

        // Otherwise if there is no Widget for the given index we need to add one.
        } else {

            // If there is no widget for the given index we can assume that the index is a
            // `widget::Id`, as the only way to procure a NodeIndex is by adding a Widget to the
            // Graph.
            let id = idx.to_widget_id(&self.index_map).expect(NO_MATCHING_WIDGET_ID);
            self.add_widget(new_container(), Some(id), maybe_parent_idx);
        }

        // Now that we've updated the widget's cached data, we need to check if we should add an
        // `Edge::Position`.
        if let Some(relative_idx) = maybe_positioned_relatively_idx {
            self.set_edge(relative_idx, idx, Edge::Position);
        // Otherwise if the widget is not positioned relatively to any other widget, we should
        // ensure that there are no incoming `Position` edges.
        } else {
            self.remove_parent_edge(idx, Edge::Position);
        }

        // Check whether or not the widget is a graphics element for some other widget.
        if let Some(graphic_parent_idx) = maybe_graphics_for {
            self.set_edge(graphic_parent_idx, idx, Edge::Graphic);
        // If not, ensure that there is no parent **Graphic** edge from the widget.
        } else {
            self.remove_parent_edge(idx, Edge::Graphic);
        }

    }

    /// Cache some `PostUpdateCache` widget data into the graph.
    ///
    /// This is called (via the `ui` module) from within the `widget::set_widget` function after
    /// the `Widget::update` method is called and some new state is returned.
    pub fn post_update_cache<W>(&mut self, widget: widget::PostUpdateCache<W>) where
        W: Widget,
        W::State: 'static,
        W::Style: 'static,
    {
        let widget::PostUpdateCache { idx, state, style, maybe_element, .. } = widget;

        // We know that their must be a NodeIndex for this idx, as `Graph::pre_update_cache` will
        // always be called prior to this method being called.
        if let Some(ref mut container) = self.widget_mut(idx) {

            // If we've been given some new `Element`
            if maybe_element.is_some() {
                container.maybe_element = maybe_element;
                container.element_has_changed = true;
            }

            // Construct the `UniqueWidgetState` ready to store as an `Any` within the container.
            let unique_state: UniqueWidgetState<W::State, W::Style> = UniqueWidgetState {
                state: state,
                style: style,
            };

            container.maybe_state = Some(Box::new(unique_state));
        }
    }


    /// Resets all **Node::Widget**s whose **Container**'s `is_updated` field is set to `false`.
    ///
    /// We do this under the assumption that the widget might reappear during some future call to
    /// **Ui::set_widgets**.
    ///
    /// When reset, **Node::Widget**s will become **Node::Placeholder**s. We do this in order to:
    ///
    /// - Preserve ordering (and in turn the validity) of existing **NodeIndex**s.
    /// - Re-use the **Node** with the same index in the case that it reappears.
    pub fn reset_non_updated_widgets(&mut self) {
        for i in 0..self.dag.raw_nodes().len() {
            let idx = NodeIndex::new(i);
            let should_remove = match self.dag.node_weight_mut(idx) {
                Some(&mut Node::Widget(ref mut container)) => !container.is_updated,
                _ => false,
            };
            if should_remove {
                self.dag[idx] = Node::Placeholder;
            }
        }
    }


    /// Remove the parent edge of the given kind for the given index if there is one.
    ///
    /// Returns `true` if an edge was removed.
    ///
    /// Returns `false` if no edges were removed.
    fn remove_parent_edge<I: GraphIndex>(&mut self, idx: I, edge: Edge) -> bool {
        let Graph { ref mut dag, ref index_map, .. } = *self;
        let node_idx = idx.to_node_index(index_map).expect(NO_MATCHING_NODE_INDEX);
        if let Some((edge_idx, _)) = dag.parents(node_idx).find(dag, |g, e, _| g[e] == edge) {
            dag.remove_edge(edge_idx);
            return true;
        }
        false
    }

    // /// Remove the parent position edge from the widget at the given index.
    // fn remove_parent_position_edge<I: GraphIndex>(&mut self, idx: I) {
    //     let Graph { ref mut dag, ref index_map, .. } = *self;
    //     let node_idx = idx.to_node_index(index_map).expect(NO_MATCHING_NODE_INDEX);
    //     if let Some((edge_idx, _)) = maybe_parent_position_edge(dag, node_idx) {
    //         dag.remove_edge(edge_idx);
    //     }
    // }

    // /// Remove the parent position edge from the widget at the given index.
    // fn remove_parent_position_edge<I: GraphIndex>(&mut self, idx: I) {
    //     let Graph { ref mut dag, ref index_map, .. } = *self;
    //     let node_idx = idx.to_node_index(index_map).expect(NO_MATCHING_NODE_INDEX);
    //     if let Some((edge_idx, _)) = maybe_parent_position_edge(dag, node_idx) {
    //         dag.remove_edge(edge_idx);
    //     }
    // }


    /// Return an `elmesque::Element` containing all widgets within the entire Graph.
    ///
    /// The order in which we will draw all widgets will be a akin to a depth-first search, where
    /// the branches with the highest `depth` are drawn first (unless the branch is on a captured
    /// widget, which will always be drawn last).
    pub fn element(&mut self,
                   maybe_captured_mouse: Option<widget::Index>,
                   maybe_captured_keyboard: Option<widget::Index>) -> Element
    {
        // Convert the GraphIndex for the widget capturing the mouse into a NodeIndex.
        let maybe_captured_mouse = maybe_captured_mouse
            .and_then(|idx| idx.to_node_index(&self.index_map));
        // Convert the GraphIndex for the widget capturing the keyboard into a NodeIndex.
        let maybe_captured_keyboard = maybe_captured_keyboard
            .and_then(|idx| idx.to_node_index(&self.index_map));

        self.prepare_to_draw(maybe_captured_mouse, maybe_captured_keyboard);

        let Graph { ref mut dag, ref depth_order, .. } = *self;

        // The main Vec in which we'll collect all `Element`s.
        let mut elements = Vec::with_capacity(depth_order.len());

        // We'll use our scroll_stack to group children of scrollable widgets so that they may be
        // cropped to their parent's scrollable area.
        // - If we come across a scrollable widget, we push a new "scroll group" Vec to our stack.
        // - If the stack isn't empty we'll push our `Element`s into the topmost (current)
        // "scroll group".
        // - If we come across a `Scrollbar`, we'll pop the top "scroll group", combine them and
        // crop them to the parent's scrollable area before adding them to the main elements Vec.
        let mut scroll_stack: Vec<Vec<Element>> = Vec::new();

        for &visitable in depth_order.iter() {
            match visitable {

                Visitable::Widget(idx) => {
                    if let &mut Node::Widget(ref mut container) = &mut dag[idx] {
                        container.was_previously_updated = container.is_updated;

                        // Push back our `Element` to one of the stacks (if we have one).
                        if let Some(ref element) = container.maybe_element {

                            // If there is some current scroll group, we'll push to that.
                            if let Some(scroll_group) = scroll_stack.last_mut() {
                                scroll_group.push(element.clone());

                            // Otherwise, we'll push straight to our main elements Vec.
                            } else {
                                elements.push(element.clone());
                            }
                        }

                        // Reset the flags for checking whether or not our `Element` has changed or
                        // if the `Widget` has been `set` between calls to `draw`.
                        container.element_has_changed = false;
                        container.is_updated = false;

                        // If the current widget is some scrollable widget, we need to add a
                        // new group to the top of our scroll stack.
                        if container.maybe_scrolling.is_some() {
                            scroll_stack.push(Vec::new());
                        }

                    }
                },

                Visitable::Scrollbar(idx) => {
                    if let &Node::Widget(ref container) = &dag[idx] {
                        if let Some(scrolling) = container.maybe_scrolling {

                            // Now that we've come across a scrollbar, we should pop the group of
                            // elements from the top of our scrollstack for cropping.
                            if let Some(scroll_group) = scroll_stack.pop() {
                                let (x, y, w, h) = scrolling.visible.x_y_w_h();
                                let element = layers(scroll_group).crop(x, y, w, h);
                                if let Some(ref mut group) = scroll_stack.last_mut() {
                                    // If there's still another layer on the scroll stack, add our
                                    // layers `Element` to the end of it.
                                    group.push(element);
                                } else {
                                    // Otherwise, push them into the elements Vec.
                                    elements.push(element);
                                }
                            }

                            // Construct the element for the scrollbar itself.
                            let element = scrolling.element();
                            elements.push(element);
                        }
                    }
                },

            }
        }

        // Convert the Vec<Element> into a single `Element` and return it.
        layers(elements)
    }


    /// Whether or not any of the Widget `Element`s have changed since the previous call to
    /// `Graph::element`.
    pub fn have_any_elements_changed(&self) -> bool {
        for node in self.dag.raw_nodes().iter() {
            if let Node::Widget(ref container) = node.weight {
                if container.element_has_changed
                || (!container.is_updated && container.was_previously_updated) {
                    return true;
                }
            }
        }
        false
    }


    /// Same as `Graph::element`, but only returns a new `Element` if any of the widgets'
    /// `Element`s in the graph have changed.
    pub fn element_if_changed(&mut self,
                              maybe_captured_mouse: Option<widget::Index>,
                              maybe_captured_keyboard: Option<widget::Index>) -> Option<Element>
    {
        // Only return a new element if one or more of the `Widget` `Element`s have changed.
        match self.have_any_elements_changed() {
            true => Some(self.element(maybe_captured_mouse, maybe_captured_keyboard)),
            false => None,
        }
    }


    // Helper method for logic shared between draw() and element().
    fn prepare_to_draw(&mut self,
                       maybe_captured_mouse: Option<NodeIndex>,
                       maybe_captured_keyboard: Option<NodeIndex>)
    {
        let Graph {
            ref mut dag,
            root,
            ref mut depth_order,
            ref mut floating_deque,
            ..
        } = *self;

        // Ensure that the depth order is up to date.
        update_depth_order(root,
                           maybe_captured_mouse,
                           maybe_captured_keyboard,
                           dag,
                           depth_order,
                           floating_deque);
    }
}


/// The box that bounds the widget with the given ID as well as all its widget kids.
///
/// Bounds are given as (max y, min y, min x, max x) from the given target xy position.
///
/// If no target xy is given, the bounds will be given relative to the centre of the widget.
///
/// If `use_kid_area` is true, then bounds will be calculated relative to the centre of the
/// `kid_area` of the widget, rather than the regular dimensions.
///
/// The `maybe_deepest_parent_idx` refers to the index of the parent that began the recursion, and
/// is used when calculating the "visible area" for each widget. If `None` is given, it means we
/// are the first widget in the recursion, and we will use ourselves as the deepest_parent_idx.
fn bounding_box(graph: &Graph,
                include_self: bool,
                maybe_target_xy: Option<Point>,
                use_kid_area: bool,
                idx: NodeIndex,
                maybe_deepest_parent_idx: Option<NodeIndex>) -> Option<Rect>
{
    graph.widget(idx).and_then(|container| {
        let dag = &graph.dag;

        // If we're to use the kid area, we'll get the rect from that, otherwise we'll use
        // the regular dim and xy.
        //
        // If we're given some deepest parent index, we must only use the visible area within the
        // depth range that approaches it.
        let rect = if let Some(deepest_parent_idx) = maybe_deepest_parent_idx {
            match visible_area_within_depth(graph, idx, Some(deepest_parent_idx)) {
                Some(visible_rect) => if use_kid_area {
                    match visible_rect.overlap(container.kid_area.rect) {
                        Some(visible_kid_area) => visible_kid_area,
                        None => return None,
                    }
                } else {
                    visible_rect
                },
                None => return None,
            }
        } else {
            if use_kid_area { container.kid_area.rect } else { container.rect }
        };

        // Determine our bounds relative to the target_xy position.
        let (xy, dim) = rect.xy_dim();
        let target_xy = maybe_target_xy.unwrap_or(xy);
        let relative_target_xy = ::vecmath::vec2_sub(xy, target_xy);
        let relative_bounds = || Rect::from_xy_dim(relative_target_xy, dim);

        // Get the deepest parent so that we can consider it in the calculation of the kids bounds.
        let deepest_parent_idx = maybe_deepest_parent_idx.or(Some(idx));

        // An iterator yielding the bounding_box returned by each of our children.
        let mut kids_bounds = dag.children(idx)
            .filter(|g, e, n| {
                let is_depth_edge = g[e] == Edge::Depth;
                let is_pickable = match g[n] {
                    Node::Widget(ref container) => !container.picking_passthrough,
                    _ => false,
                };
                let is_graphic = maybe_parent_graphic_edge(dag, n).is_some();
                is_depth_edge && is_pickable && !is_graphic
            })
            .iter(dag).nodes()
            .filter_map(|n| {
                bounding_box(graph, true, Some(target_xy), false, n, deepest_parent_idx)
            });

        // Work out the initial bounds to use for our max_bounds fold.
        let init_bounds = if include_self {
            relative_bounds()
        } else {
            match kids_bounds.next() {
                Some(first_kid_bounds) => first_kid_bounds,
                None => return None,
            }
        };

        // Fold the Rect for each kid into the total encompassing bounds.
        Some(kids_bounds.fold(init_bounds, |a, b| a.max(b)))
    })
}


/// The rectangle that represents the maximum fully visible area for the widget with the given
/// index, including consideration of the cropped scroll area for all parents until (and not
/// including) the deepest_parent_idx is reached (if one is given).
///
/// Otherwise, return None if the widget is hidden.
fn visible_area_within_depth(graph: &Graph,
                             idx: NodeIndex,
                             maybe_deepest_parent_idx: Option<NodeIndex>) -> Option<Rect>
{
    graph.widget(idx).and_then(|widget| {
        let mut overlapping_rect = widget.rect;

        let mut depth_parents = parent_depth_recursion(&graph.dag, idx);
        while let Some(depth_parent) = depth_parents.next_node(&graph.dag) {

            // If the parent's index matches that of the deepest, we're done.
            if Some(depth_parent) == maybe_deepest_parent_idx {
                break;
            }

            // Otherwise, if our parent is scrollable, check for overlap.
            if let Some(depth_parent_widget) = graph.widget(depth_parent) {
                if depth_parent_widget.maybe_scrolling.is_some() {

                    // We only need to update the overlap if the parent is scrollable.
                    match overlapping_rect.overlap(depth_parent_widget.kid_area.rect) {
                        Some(overlap) => overlapping_rect = overlap,
                        None => return None,
                    }
                }
            }
        }

        Some(overlapping_rect)
    })
}


/// Set some given `Edge` between `a` -> `b`, so that it is the only `Edge` of its variant.
fn set_edge(dag: &mut Dag, a: NodeIndex, b: NodeIndex, edge: Edge) {

    // Check to see if the node already has some matching incoming edge.
    // Keep it if it's the one we want. Otherwise, remove any incoming edge that matches the given
    // edge kind but isn't coming from the node that we desire.
    let mut parents = dag.parents(b);
    let mut already_set = false;

    while let Some((in_edge_idx, in_node_idx)) = parents.next(dag) {
        if edge == dag[in_edge_idx] {
            if in_node_idx == a {
                already_set = true;
            } else {
                dag.remove_edge(in_edge_idx);
            }
            // Note that we only need to check for *one* edge as there can only ever be one
            // parent or relative position per node. We know this, as this method is the only
            // function used by a public method that adds edges.
            break;
        }
    }

    // If we don't already have an incoming edge from the requested parent, add one.
    if !already_set {
        // Add a Depth edge from a -> b.
        if let Err(_) = dag.add_edge(a, b, edge) {
            use std::io::Write;
            writeln!(::std::io::stderr(),
                     "Error: Adding a connection from node {:?} to node {:?} would cause a cycle \
                     within the Graph.", a, b).unwrap();
        }
    }

}


/// Return the parent edge of the given kind (and the attached parent Node) if one exists.
///
/// We know that there may be at most one parent graphic edge, as the only publicly exposed way to
/// add an edge to the graph is via the `set_edge` method.
fn maybe_parent_edge(dag: &Dag, idx: NodeIndex, edge: Edge) -> Option<IndexPair> {
    dag.parents(idx).find(dag, |g, e, _| g[e] == edge)
}

/// Return the parent **Depth** edge (and the attached parent Node) if one exists.
///
/// We know that there may be at most one parent depth edge, as the only publicly exposed way to
/// add an edge to the graph is via the `set_edge` method.
fn maybe_parent_depth_edge(dag: &Dag, idx: NodeIndex) -> Option<IndexPair> {
    maybe_parent_edge(dag, idx, Edge::Depth)
}

/// Return the parent relative **Position** edge (and the attached Node) if one exists.
///
/// We know that there may be at most one parent relative position edge, as the only publicly
/// exposed way to add an edge to the graph is via the `set_edge` method.
fn maybe_parent_position_edge(dag: &Dag, idx: NodeIndex) -> Option<IndexPair> {
    maybe_parent_edge(dag, idx, Edge::Position)
}

/// Return the parent **Graphic** edge (and the attached parent Node) if one exists.
///
/// We know that there may be at most one parent graphic edge, as the only publicly exposed way to
/// add an edge to the graph is via the `set_edge` method.
fn maybe_parent_graphic_edge(dag: &Dag, idx: NodeIndex) -> Option<IndexPair> {
    maybe_parent_edge(dag, idx, Edge::Graphic)
}


/// Recursively walk through all parent **Depth** **Edge**s starting from the given index.
fn parent_depth_recursion(dag: &Dag, idx: NodeIndex)
    -> RecursiveWalk<fn(&Dag, NodeIndex) -> Option<IndexPair>>
{
    dag.recursive_walk(idx, maybe_parent_depth_edge)
}

/// Recursively walk through all parent **Position** **Edge**s starting from the given index.
fn parent_position_recursion(dag: &Dag, idx: NodeIndex)
    -> RecursiveWalk<fn(&Dag, NodeIndex) -> Option<IndexPair>>
{
    dag.recursive_walk(idx, maybe_parent_position_edge)
}

/// Recursively walk through all parent **Graphic** **Edge**s starting from the given index.
fn parent_graphic_recursion(dag: &Dag, idx: NodeIndex)
    -> RecursiveWalk<fn(&Dag, NodeIndex) -> Option<IndexPair>>
{
    dag.recursive_walk(idx, maybe_parent_graphic_edge)
}


/// Whether or not the given `possible_parent` is a **Depth** parent of the widget at the given
/// `idx`.
fn is_depth_parent_of(idx: NodeIndex, dag: &Dag, possible_parent: NodeIndex) -> bool {
    parent_depth_recursion(dag, idx).any(dag, |_, _, n| n == possible_parent)
}

/// Whether or not the given `possible_parent` is a **Position** parent of the widget at the given
/// `idx`.
fn is_position_parent_of(idx: NodeIndex, dag: &Dag, possible_parent: NodeIndex) -> bool {
    parent_position_recursion(dag, idx).any(dag, |_, _, n| n == possible_parent)
}

/// Whether or not the given `possible_parent` is a **Graphic** parent of the widget at the given
/// `idx`.
fn is_graphic_parent_of(idx: NodeIndex, dag: &Dag, possible_parent: NodeIndex) -> bool {
    parent_graphic_recursion(dag, idx).any(dag, |_, _, n| n == possible_parent)
}


/// Update the depth_order (starting with the deepest) for all nodes in the graph.
/// The floating_deque is a pre-allocated deque used for collecting the floating widgets during
/// visiting so that they may be drawn last.
fn update_depth_order(root: NodeIndex,
                      maybe_captured_mouse: Option<NodeIndex>,
                      maybe_captured_keyboard: Option<NodeIndex>,
                      dag: &Dag,
                      depth_order: &mut Vec<Visitable>,
                      floating_deque: &mut Vec<NodeIndex>)
{

    // Clear the buffers and ensure they've enough memory allocated.
    let num_nodes = dag.node_count();
    depth_order.clear();
    depth_order.reserve(num_nodes);
    floating_deque.clear();
    floating_deque.reserve(num_nodes);

    // Visit each node in order of depth and add their indices to depth_order.
    // If the widget is floating, then store it in the floating_deque instead.
    visit_by_depth(root,
                   maybe_captured_mouse,
                   maybe_captured_keyboard,
                   dag,
                   depth_order,
                   floating_deque);

    // Sort the floating widgets so that the ones clicked last come last.
    floating_deque.sort_by(|&a, &b| match (&dag[a], &dag[b]) {
        (&Node::Widget(ref a), &Node::Widget(ref b)) => {
            let a_floating = a.maybe_floating.expect("Not floating");
            let b_floating = b.maybe_floating.expect("Not floating");
            a_floating.time_last_clicked.cmp(&b_floating.time_last_clicked)
        },
        _ => ::std::cmp::Ordering::Equal,
    });

    // Visit all of the floating widgets last.
    while !floating_deque.is_empty() {
        let idx = floating_deque.remove(0);
        visit_by_depth(idx,
                       maybe_captured_mouse,
                       maybe_captured_keyboard,
                       dag,
                       depth_order,
                       floating_deque);
    }
}


/// Recursive function for visiting all nodes within the dag.
fn visit_by_depth(idx: NodeIndex,
                  maybe_captured_mouse: Option<NodeIndex>,
                  maybe_captured_keyboard: Option<NodeIndex>,
                  dag: &Dag,
                  depth_order: &mut Vec<Visitable>,
                  floating_deque: &mut Vec<NodeIndex>)
{
    // First, store the index of the current node.
    match &dag[idx] {
        &Node::Widget(ref container) if container.is_updated =>
            depth_order.push(Visitable::Widget(idx)),
        &Node::Root => (),
        // If the node is neither an updated widget or the Root, we are done with this branch.
        _ => return,
    }

    // Sort the children of the current node by their `.depth` members.
    // FIXME: We should remove these allocations by storing a `child_sorter` buffer in each Widget
    // node (perhaps in the `Container`).
    let mut child_sorter: Vec<NodeIndex> = dag.children(idx).iter(&dag)
        .filter(|&(e, _)| dag[e] == Edge::Depth)
        .map(|(_, n)| n)
        .collect();
    // Walking neighbors of a node in our graph returns then in the reverse order in which they
    // were added. Reversing here will give more predictable render order behaviour i.e. widgets
    // instantiated after other widgets will also be rendered after them by default.
    child_sorter.reverse();
    child_sorter.sort_by(|&a, &b| {
        use std::cmp::Ordering;
        if Some(a) == maybe_captured_mouse || Some(a) == maybe_captured_keyboard {
            Ordering::Greater
        } else if let (&Node::Widget(ref a), &Node::Widget(ref b)) = (&dag[a], &dag[b]) {
            b.depth.partial_cmp(&a.depth).expect("Depth was NaN!")
        } else {
            Ordering::Equal
        }
    });

    // Then, visit each of the child widgets. If we come across any floating widgets, we'll store
    // those in the floating deque so that we can visit them following the current tree.
    for child_idx in child_sorter.into_iter() {

        // Determine whether or not the node is a floating widget.
        let maybe_is_floating = match dag.node_weight(child_idx) {
            Some(&Node::Widget(ref container)) => Some(container.maybe_floating.is_some()),
            _                                  => None,
        };

        // Store floating widgets int he floating_deque for visiting after the current tree.
        match maybe_is_floating {
            Some(true) => floating_deque.push(child_idx),
            _          => visit_by_depth(child_idx,
                                         maybe_captured_mouse,
                                         maybe_captured_keyboard,
                                         dag,
                                         depth_order,
                                         floating_deque),
        }
    }

    // If the widget is scrollable, we should add its scrollbar to the visit order also.
    if let &Node::Widget(ref container) = &dag[idx] {
        if container.maybe_scrolling.is_some() {
            depth_order.push(Visitable::Scrollbar(idx));
        }
    }
}


impl<I: GraphIndex> ::std::ops::Index<I> for Graph {
    type Output = Container;
    fn index<'a>(&'a self, idx: I) -> &'a Container {
        self.widget(idx).expect("No Widget matching the given ID")
    }
}

impl<I: GraphIndex> ::std::ops::IndexMut<I> for Graph {
    fn index_mut<'a>(&'a mut self, idx: I) -> &'a mut Container {
        self.widget_mut(idx).expect("No Widget matching the given ID")
    }
}

