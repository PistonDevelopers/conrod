//! Conrod uses a directed acyclic graph to manage both storing widgets and describing their
//! relationships.
//!
//! The primary type of interest in this module is the [**Graph**](./struct.Graph) type.

use Backend;
use daggy;
use position::{Axis, Depth, Rect};
use self::index_map::IndexMap;
use std::any::Any;
use std::iter;
use std::ops::{Index, IndexMut};
use std::option;
use widget::{self, Widget};

pub use daggy::Walker;
pub use self::depth_order::{DepthOrder, Visitable};
pub use self::index_map::GraphIndex;

pub mod algo;
pub mod depth_order;
mod index_map;


/// An alias for our Graph's Node Index.
pub type NodeIndex = daggy::NodeIndex<u32>;

/// An alias for our Graph's Edge Index.
pub type EdgeIndex = daggy::EdgeIndex<u32>;

/// An alias for a tuple containing an associated `Edge/NodeIndex` pair.
pub type IndexPair = (EdgeIndex, NodeIndex);

/// A **Walker** over some node's parent nodes.
pub type Parents = daggy::Parents<Node, Edge, u32>;
/// A **Walker** over some node's child nodes.
pub type Children = daggy::Children<Node, Edge, u32>;

/// An alias for the iterator yielding both **X** and **Y** **Position** parents.
pub type PositionParents<I> = iter::Chain<option::IntoIter<I>, option::IntoIter<I>>;

/// An alias for some filtered children walker.
pub type FilteredChildren =
    daggy::walker::Filter<Children, fn(&Graph, EdgeIndex, NodeIndex) -> bool>;
/// An alias for a **Walker** over a node's **Depth** children.
pub type DepthChildren = FilteredChildren;
/// An alias for a **Walker** over a node's **X Position** children.
pub type XPositionChildren = FilteredChildren;
/// An alias for a **Walker** over a node's **Y Position** children.
pub type YPositionChildren = FilteredChildren;
/// An alias for a **Walker** over a node's **X** and **Y** **Position** children respectively.
pub type PositionChildren = daggy::walker::Chain<Graph, u32, XPositionChildren, YPositionChildren>;
/// An alias for a **Walker** over a node's **Graphic** children.
pub type GraphicChildren = FilteredChildren;

/// An alias for our Graph's recursive walker.
pub type RecursiveWalk<F> = daggy::walker::Recursive<Graph, u32, F>;

/// An alias for our Graph's **WouldCycle** error type.
pub type WouldCycle = daggy::WouldCycle<Edge>;

/// The state type that we'll dynamically cast to and from `Any` for storage within the cache.
#[derive(Debug)]
pub struct UniqueWidgetState<State, Style> where
    State: Any,
    Style: Any,
{
    /// A **Widget**'s unique "State".
    pub state: State,
    /// A **Widget**'s unique "Style".
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
    /// Scroll related state (is only `Some` if this axis is scrollable).
    pub maybe_x_scroll_state: Option<widget::scroll::StateX>,
    /// Scroll related state (is only `Some` if this axis is scrollable).
    pub maybe_y_scroll_state: Option<widget::scroll::StateY>,
    /// Represents the Widget's position within the overall instantiation ordering of the widgets.
    ///
    /// i.e. if foo's `instantiation_order_idx` is lower than bar's, it means that foo was
    /// instantiated before bar.
    pub instantiation_order_idx: usize,
}

/// A node for use within the **Graph**.
#[derive(Debug)]
pub enum Node {
    /// A widget constructed by a user.
    Widget(Container),
    /// A placeholder node - used when reserving a place for a **Widget** within the **Graph**.
    ///
    /// It may also be used to represent a node that was once pre-occuppied by a widget who was not
    /// `set` during the last `set_widgets` stage.
    Placeholder,
}

/// An edge between nodes within the UI Graph.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Edge {
    /// Describes the relative positioning of widgets.
    ///
    /// When adding an edge *a -> b*, *b* is positioned relatively to *a*.
    Position(Axis),
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

/// The number of different variants within the **Edge** enum.
pub const NUM_EDGE_VARIANTS: usize = 4;

/// An alias for the petgraph::Graph used within our Ui Graph.
type Dag = daggy::Dag<Node, Edge>;

/// Stores the dynamic state of a UI tree of Widgets.
#[derive(Debug)]
pub struct Graph {
    /// Cached widget state in a directed acyclic graph whose edges describe the rendering tree and
    /// positioning.
    dag: Dag,
    /// A map of the UiId to the graph's indices.
    index_map: IndexMap,
}


/// A common argument when expecting that there is a `NodeIndex`.
const NO_MATCHING_NODE_INDEX: &'static str = "No matching NodeIndex";


impl Container {

    /// Borrow the **Container**'s unique widget State and Style if there is any.
    pub fn state_and_style<State, Style>(&self) -> Option<&UniqueWidgetState<State, Style>>
        where State: Any + 'static,
              Style: Any + 'static,
    {
        self.maybe_state.as_ref().and_then(|boxed_state| boxed_state.downcast_ref())
    }

    /// Same as [**Container::state_and_style**](./struct.Container#method.state_and_style) but
    /// accessed using a **Widget** type parameter instead of the unique State and Style types.
    pub fn unique_widget_state<B, W>(&self) -> Option<&UniqueWidgetState<W::State, W::Style>>
        where B: Backend,
              W: Widget<B>,
              W::State: Any + 'static,
              W::Style: Any + 'static,
    {
        self.state_and_style::<W::State, W::Style>()
    }

    /// A method for taking only the unique state from the container.
    pub fn take_unique_widget_state<B, W>(&mut self)
        -> Option<Box<UniqueWidgetState<W::State, W::Style>>>
        where B: Backend,
              W: Widget<B>,
              W::State: Any + 'static,
              W::Style: Any + 'static,
    {
        self.maybe_state.take().map(|any_state| {
            any_state.downcast().ok()
                .expect("Failed to downcast from `Box<Any>` to the required UniqueWidgetState")
        })
    }

    /// Take the widget state from the container and cast it to type W.
    pub fn take_widget_state<B, W>(&mut self) -> Option<widget::Cached<B, W>>
        where B: Backend,
              W: Widget<B>,
              W::State: Any + 'static,
              W::Style: Any + 'static,
    {
        if self.maybe_state.is_some() {
            let boxed_unique_state = self.take_unique_widget_state::<B, W>().unwrap();
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
                maybe_x_scroll_state: self.maybe_x_scroll_state,
                maybe_y_scroll_state: self.maybe_y_scroll_state,
            })
        } else {
            None
        }
    }

}


impl Node {

    /// Whether or not the **Node** is of the **Widget** variant.
    pub fn is_widget(&self) -> bool {
        if let Node::Widget(_) = *self { true } else { false }
    }

}


impl Graph {

    /// A new empty **Graph**.
    pub fn new() -> Self {
        Graph {
            dag: Dag::new(),
            index_map: IndexMap::new(),
        }
    }

    /// A new **Graph** with the given node capacity.
    ///
    /// We know that there can be no more than three parents per node as the public API enforces a
    /// maximum of one Depth, Position and Graphic parent each. Thus, we can assume an edge
    /// capacity of exactly three times the given node capacity.
    pub fn with_node_capacity(n_nodes: usize) -> Self {
        let n_edges = n_nodes * NUM_EDGE_VARIANTS;
        Graph {
            dag: Dag::with_capacity(n_nodes, n_edges),
            index_map: IndexMap::with_capacity(n_nodes),
        }
    }

    /// Removes all **Node**s and **Edge**s from the **Graph**.
    pub fn clear(&mut self) {
        self.dag.clear()
    }

    /// The total number of **Node**s in the **Graph**.
    pub fn node_count(&self) -> usize {
        self.dag.node_count()
    }

    /// The total number of **Node::Widget**s in the **Graph**.
    pub fn widget_count(&self) -> usize {
        (0..self.node_count())
            .filter(|&i| self[NodeIndex::new(i)].is_widget())
            .count()
    }

    /// The total number of **Edge**s in the **Graph**.
    pub fn edge_count(&self) -> usize {
        self.dag.edge_count()
    }

    /// The current capacity for the **Graph**'s internal node `Vec`.
    pub fn node_capacity(&self) -> usize {
        unimplemented!();
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

    /// Get the **widget::Index** for the given **GraphIndex**.
    ///
    /// First attempts to produce a **WidgetId**, then tries to produce a **NodeIndex**.
    pub fn widget_index<I: GraphIndex>(&self, idx: I) -> Option<widget::Index> {
        self.widget_id(idx).map(|id| widget::Index::Public(id))
            .or_else(|| self.node_index(idx).map(|idx| widget::Index::Internal(idx)))
    }

    /// Add the given **Node** to the graph.
    ///
    /// Computes in **O(1)** time.
    ///
    /// Returns the index of the new node.
    fn add_node(&mut self, node: Node) -> NodeIndex {
        self.dag.add_node(node)
    }

    /// Set the given **Edge** within the graph.
    ///
    /// The added edge will be in the direction `a` -> `b`
    ///
    /// There may only ever be one **Edge** of the given variant between `a` -> `b`. In turn, the
    /// **Graph** could be described as "three rose trees super imposed on top of one another,
    /// where there is one tree for each edge variant".
    ///
    /// Checks if the edge would create a cycle in the **Graph**.
    ///
    /// If adding the edge **would not** cause the graph to cycle, the edge will be added and its
    /// `EdgeIndex` returned.
    ///
    /// If adding the edge **would** cause the graph to cycle, the edge will not be added and
    /// instead a `WouldCycle` error with the given weight will be returned.
    ///
    /// **Panics** if either `a` or `b` do not exist within the **Graph**.
    ///
    /// **Panics** if the **Graph** is at the maximum number of nodes for its index type.
    fn set_edge<A, B>(&mut self, a: A, b: B, edge: Edge) -> Result<EdgeIndex, WouldCycle>
        where A: GraphIndex,
              B: GraphIndex,
    {
        let a = self.node_index(a).expect(NO_MATCHING_NODE_INDEX);
        let b = self.node_index(b).expect(NO_MATCHING_NODE_INDEX);

        // Check to see if the node already has some matching incoming edge.
        // Keep it if it's the one we want. Otherwise, remove any incoming edge that matches the given
        // edge kind but isn't coming from the node that we desire.
        let mut parents = self.parents(b);
        let mut already_set = None;

        while let Some((in_edge_idx, in_node_idx)) = parents.next(self) {
            if edge == self[in_edge_idx] {
                if in_node_idx == a {
                    already_set = Some(in_edge_idx);
                } else {
                    self.remove_edge(in_edge_idx);
                }
                // Note that we only need to check for *one* edge as there can only ever be one
                // parent edge of any kind for each node. We know this, as this method is the only
                // function used by a public method that adds edges.
                break;
            }
        }

        // If we don't already have an incoming edge from the requested parent, add one.
        match already_set {
            Some(edge_idx) => Ok(edge_idx),
            None => self.dag.add_edge(a, b, edge),
        }
    }

    /// Remove and return the **Edge** at the given index.
    ///
    /// Return `None` if it didn't exist.
    fn remove_edge(&mut self, idx: EdgeIndex) -> Option<Edge> {
        self.dag.remove_edge(idx)
    }

    /// Remove the parent edge of the given kind for the given index if there is one.
    ///
    /// Returns `true` if an edge was removed.
    ///
    /// Returns `false` if no edges were removed.
    fn remove_parent_edge<I: GraphIndex>(&mut self, idx: I, edge: Edge) -> bool {
        self.node_index(idx).map(|idx| {
            if let Some((edge_idx, _)) = self.parents(idx).find(self, |g, e, _| g[e] == edge) {
                self.remove_edge(edge_idx);
                return true;
            }
            false
        }).unwrap_or(false)
    }

    /// Add a new placeholder node and return it's `NodeIndex` into the `Graph`.
    ///
    /// This method is used by the `widget::set_widget` function when some internal widget does not
    /// yet have it's own `NodeIndex`.
    pub fn add_placeholder(&mut self) -> NodeIndex {
        self.add_node(Node::Placeholder)
    }

    /// Borrow the node at the given **GraphIndex** if there is one.
    pub fn node<I: GraphIndex>(&self, idx: I) -> Option<&Node> {
        self.node_index(idx).and_then(|idx| self.dag.node_weight(idx))
    }

    /// Mutably borrow the node at the given **GraphIndex** if there is one.
    pub fn node_mut<I: GraphIndex>(&mut self, idx: I) -> Option<&mut Node> {
        self.node_index(idx).and_then(move |idx| self.dag.node_weight_mut(idx))
    }

    /// Borrow the edge at the given **EdgeIndex** if there is one.
    pub fn edge(&self, idx: EdgeIndex) -> Option<&Edge> {
        self.dag.edge_weight(idx)
    }

    /// Mutably borrow the edge at the given **EdgeIndex** if there is one.
    pub fn edge_mut(&mut self, idx: EdgeIndex) -> Option<&mut Edge> {
        self.dag.edge_weight_mut(idx)
    }

    /// Return the parent and child nodes on either end of the **Edge** at the given index.
    pub fn edge_endpoints(&self, idx: EdgeIndex) -> Option<(NodeIndex, NodeIndex)> {
        self.dag.edge_endpoints(idx)
    }

    /// If there is a Widget for the given index, return a reference to it.
    pub fn widget<I: GraphIndex>(&self, idx: I) -> Option<&Container> {
        self.node(idx).and_then(|node| match *node {
            Node::Widget(ref container) => Some(container),
            _ => None,
        })
    }

    /// If there is a Widget for the given Id, return a mutable reference to it.
    pub fn widget_mut<I: GraphIndex>(&mut self, idx: I) -> Option<&mut Container> {
        self.node_mut(idx).and_then(|node| match *node {
            Node::Widget(ref mut container) => Some(container),
            _ => None,
        })
    }

    /// A **Walker** type that may be used to step through the parents of the given child node.
    pub fn parents<I: GraphIndex>(&self, child: I) -> Parents {
        let idx = self.node_index(child).unwrap_or(NodeIndex::end());
        self.dag.parents(idx)
    }

    /// A **Walker** type that recursively walks the **Graph** using the given `recursive_fn`.
    ///
    /// **Panics** If the given start index does not exist within the **Graph**.
    pub fn recursive_walk<I, F>(&self, start: I, recursive_fn: F) -> RecursiveWalk<F>
        where I: GraphIndex,
              F: FnMut(&Self, NodeIndex) -> Option<(EdgeIndex, NodeIndex)>
    {
        let start = self.node_index(start).expect(NO_MATCHING_NODE_INDEX);
        RecursiveWalk::new(start, recursive_fn)
    }

    /// If the widget at the given index has some parent along an **Edge** of the given variant,
    /// return an index to it.
    pub fn edge_parent<I, J>(&self, idx: I, edge: Edge) -> Option<J>
        where I: GraphIndex,
              J: GraphIndex,
    {
        self.parents(idx).find(self, |g, e, _| g[e] == edge).and_then(|(_, n)| self.convert_idx(n))
    }

    /// Return the index of the parent along the given widget's **Depth** **Edge**.
    pub fn depth_parent<I, J>(&self, idx: I) -> Option<J>
        where I: GraphIndex,
              J: GraphIndex,
    {
        self.edge_parent(idx, Edge::Depth)
    }

    /// Return the index of the parent along the given widget's **Position** **Edge**.
    pub fn x_position_parent<I, J>(&self, idx: I) -> Option<J>
        where I: GraphIndex,
              J: GraphIndex,
    {
        self.edge_parent(idx, Edge::Position(Axis::X))
    }

    /// Return the index of the parent along the given widget's **Position** **Edge**.
    pub fn y_position_parent<I, J>(&self, idx: I) -> Option<J>
        where I: GraphIndex,
              J: GraphIndex,
    {
        self.edge_parent(idx, Edge::Position(Axis::Y))
    }

    /// Produces an iterator yielding the parents along both the **X** and **Y** **Position**
    /// **Edge**s respectively.
    pub fn position_parents<I, J>(&self, idx: I) -> PositionParents<J>
        where I: GraphIndex,
              J: GraphIndex,
    {
        self.x_position_parent(idx).into_iter().chain(self.y_position_parent(idx))
    }

    /// Return the index of the parent along the given widget's **Graphic** **Edge**.
    pub fn graphic_parent<I, J>(&self, idx: I) -> Option<J>
        where I: GraphIndex,
              J: GraphIndex,
    {
        self.edge_parent(idx, Edge::Graphic)
    }

    /// A **Walker** type that recursively walks **Depth** parents starting from the given node.
    pub fn depth_parent_recursion<I: GraphIndex>(&self, idx: I)
        -> RecursiveWalk<fn(&Graph, NodeIndex) -> Option<IndexPair>>
    {
        fn depth_edge_parent(graph: &Graph, idx: NodeIndex) -> Option<IndexPair> {
            graph.parents(idx).find(graph, |g, e, _| g[e] == Edge::Depth)
        }
        self.recursive_walk(idx, depth_edge_parent)
    }

    /// A **Walker** type that recursively walks **X** **Position** parents starting from the given
    /// node.
    pub fn x_position_parent_recursion<I: GraphIndex>(&self, idx: I)
        -> RecursiveWalk<fn(&Graph, NodeIndex) -> Option<IndexPair>>
    {
        fn x_position_edge_parent(graph: &Graph, idx: NodeIndex) -> Option<IndexPair> {
            graph.parents(idx).find(graph, |g, e, _| g[e] == Edge::Position(Axis::X))
        }
        self.recursive_walk(idx, x_position_edge_parent)
    }

    /// A **Walker** type that recursively walks **Y** **Position** parents starting from the given
    /// node.
    pub fn y_position_parent_recursion<I: GraphIndex>(&self, idx: I)
        -> RecursiveWalk<fn(&Graph, NodeIndex) -> Option<IndexPair>>
    {
        fn y_position_edge_parent(graph: &Graph, idx: NodeIndex) -> Option<IndexPair> {
            graph.parents(idx).find(graph, |g, e, _| g[e] == Edge::Position(Axis::Y))
        }
        self.recursive_walk(idx, y_position_edge_parent)
    }

    /// A **Walker** type that recursively walks **Graphic** parents starting from the given node.
    pub fn graphic_parent_recursion<I: GraphIndex>(&self, idx: I)
        -> RecursiveWalk<fn(&Graph, NodeIndex) -> Option<IndexPair>>
    {
        fn graphic_edge_parent(graph: &Graph, idx: NodeIndex) -> Option<IndexPair> {
            graph.parents(idx).find(graph, |g, e, _| g[e] == Edge::Graphic)
        }
        self.recursive_walk(idx, graphic_edge_parent)
    }

    /// A **Walker** type that may be used to step through the children of the given parent node.
    pub fn children<I: GraphIndex>(&self, parent: I) -> Children {
        let idx = self.node_index(parent).unwrap_or(NodeIndex::end());
        self.dag.children(idx)
    }

    /// For walking the **Depth** children of the given parent node.
    pub fn depth_children<I: GraphIndex>(&self, idx: I) -> DepthChildren {
        self.children(idx).filter(is_depth_edge)
    }

    /// For walking the **Position(X)** children of the given parent node.
    pub fn x_position_children<I: GraphIndex>(&self, idx: I) -> XPositionChildren {
        self.children(idx).filter(is_x_position_edge)
    }

    /// For walking the **Position(Y)** children of the given parent node.
    pub fn y_position_children<I: GraphIndex>(&self, idx: I) -> YPositionChildren {
        self.children(idx).filter(is_y_position_edge)
    }

    /// For walking the **Position** children of the given parent node.
    ///
    /// This first walks the **Axis::X** children, before walking the **Axis::Y** children.
    pub fn position_children<I: GraphIndex>(&self, idx: I) -> PositionChildren {
        self.x_position_children(idx).chain(self.y_position_children(idx))
    }

    /// For walking the **Graphic** children of the given parent node.
    pub fn graphic_children<I: GraphIndex>(&self, idx: I) -> GraphicChildren {
        self.children(idx).filter(is_graphic_edge)
    }

    /// Does the given edge type exist between the nodes `parent` -> `child`.
    ///
    /// Returns `false` if either of the given node indices do not exist.
    pub fn does_edge_exist<P, C, F>(&self, parent: P, child: C, is_edge: F) -> bool
        where P: GraphIndex,
              C: GraphIndex,
              F: Fn(Edge) -> bool,
    {
        self.node_index(parent).map(|parent| {
            self.parents(child).any(self, |g, e, n| n == parent && is_edge(g[e]))
        }).unwrap_or(false)
    }

    /// Does a **Edge::Depth** exist between the nodes `parent` -> `child`.
    ///
    /// Returns `false` if either of the given node indices do not exist.
    pub fn does_depth_edge_exist<P, C>(&self, parent: P, child: C) -> bool
        where P: GraphIndex,
              C: GraphIndex,
    {
        self.does_edge_exist(parent, child, |e| e == Edge::Depth)
    }

    /// Does a **Edge::Position** exist between the nodes `parent` -> `child`.
    ///
    /// Returns `false` if either of the given node indices do not exist.
    pub fn does_position_edge_exist<P, C>(&self, parent: P, child: C) -> bool
        where P: GraphIndex,
              C: GraphIndex,
    {
        let is_edge = |e| e == Edge::Position(Axis::X) || e == Edge::Position(Axis::Y);
        self.does_edge_exist(parent, child, is_edge)
    }

    /// Does a **Edge::Graphic** exist between the nodes `parent` -> `child`.
    ///
    /// Returns `false` if either of the given node indices do not exist.
    pub fn does_graphic_edge_exist<P, C>(&self, parent: P, child: C) -> bool
        where P: GraphIndex,
              C: GraphIndex,
    {
        self.does_edge_exist(parent, child, |e| e == Edge::Graphic)
    }

    /// Are the given `parent` and `child` nodes connected by a single chain of edges of the given
    /// kind?
    ///
    /// i.e. `parent` -> x -> y -> `child`.
    ///
    /// Returns `false` if either of the given node indices do not exist.
    pub fn does_recursive_edge_exist<P, C, F>(&self, parent: P, child: C, is_edge: F) -> bool
        where P: GraphIndex,
              C: GraphIndex,
              F: Fn(Edge) -> bool,
    {
        self.node_index(parent).map(|parent| {
            self.recursive_walk(child, |g, n| g.parents(n).find(g, |g, e, _| is_edge(g[e])))
                .any(self, |_, _, n| n == parent)
        }).unwrap_or(false)
    }

    /// Are the given `parent` and `child` nodes connected by a single chain of **Depth** edges?
    ///
    /// i.e. `parent` -> x -> y -> `child`.
    ///
    /// Returns `false` if either of the given node indices do not exist.
    pub fn does_recursive_depth_edge_exist<P, C>(&self, parent: P, child: C) -> bool
        where P: GraphIndex,
              C: GraphIndex,
    {
        self.does_recursive_edge_exist(parent, child, |e| e == Edge::Depth)
    }

    // FIXME: This only recurses down the *first* edge that satisfies the predicate, whereas we
    // want to check *every* position parent edge. This means we need to do a DFS or BFS over
    // position edges from the parent node until we find the child node.
    // ///
    // /// Are the given `parent` and `child` nodes connected by a single chain of **Position** edges?
    // ///
    // /// i.e. `parent` -> x -> y -> `child`.
    // ///
    // /// Returns `false` if either of the given node indices do not exist.
    // pub fn does_recursive_position_edge_exist<P, C>(&self, parent: P, child: C) -> bool
    //     where P: GraphIndex,
    //           C: GraphIndex,
    // {
    //     let is_edge = |e| e == Edge::Position(Axis::X) || e == Edge::Position(Axis::Y);
    //     self.does_recursive_edge_exist(parent, child, is_edge)
    // }

    /// Are the given `parent` and `child` nodes connected by a single chain of **Graphic** edges?
    ///
    /// i.e. `parent` -> x -> y -> `child`.
    ///
    /// Returns `false` if either of the given node indices do not exist.
    pub fn does_recursive_graphic_edge_exist<P, C>(&self, parent: P, child: C) -> bool
        where P: GraphIndex,
              C: GraphIndex,
    {
        self.does_recursive_edge_exist(parent, child, |e| e == Edge::Graphic)
    }


    /// Cache some `PreUpdateCache` widget data into the graph.
    ///
    /// This is called (via the `ui` module) from within the `widget::set_widget` function prior to
    /// the `Widget::update` method being called.
    ///
    /// This is done so that if this Widget were to internally `set` some other `Widget`s within
    /// its own `update` method, this `Widget`s positioning and dimension data already exists
    /// within the `Graph` for reference.
    pub fn pre_update_cache(&mut self,
                            root: NodeIndex,
                            widget: widget::PreUpdateCache,
                            instantiation_order_idx: usize)
    {
        let widget::PreUpdateCache {
            kind, idx, maybe_parent_idx, maybe_x_positioned_relatively_idx,
            maybe_y_positioned_relatively_idx, rect, depth, kid_area, drag_state, maybe_floating,
            maybe_x_scroll_state, maybe_y_scroll_state, maybe_graphics_for,
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
            maybe_x_scroll_state: maybe_x_scroll_state,
            maybe_y_scroll_state: maybe_y_scroll_state,
            instantiation_order_idx: instantiation_order_idx,
        };

        // Retrieves the widget's parent index.
        //
        // If a parent index is given but does not yet exist within the **Graph**, add a temporary
        // node to the graph at the given parent index so that we can add the edge even in the
        // parent widget's absense. The temporary node should be replaced by the proper widget when
        // it is updated later in the cycle.
        //
        // If no parent index is given, the **Graph**'s `root` index will be used as the parent.
        let maybe_parent_idx = |graph: &mut Self| match maybe_parent_idx {
            Some(parent_idx) => match graph.node_index(parent_idx) {
                Some(idx) => Some(idx),
                None => {
                    // Add a temporary node to the graph at the given parent index so that we can
                    // add the edge even in the parent widget's absense. The temporary node should
                    // be replaced by the proper widget when it is updated later in the cycle.
                    let parent_node_idx = graph.add_placeholder();
                    // If the parent_idx is a **WidgetId**, add the mapping to our index_map.
                    if let widget::Index::Public(parent_widget_id) = parent_idx {
                        graph.index_map.insert(parent_widget_id, parent_node_idx);
                    }
                    Some(parent_node_idx)
                },
            },
            // Check that this node is not the root node before using the root node as the parent.
            None => graph.node_index(idx)
                .and_then(|idx| if idx == root { None } else { Some(root) }),
        };

        // If we already have a `Node` in the graph for the given `idx`, we need to update it.
        if let Some(node_idx) = self.node_index(idx) {

            // Ensure that we have an `Edge::Depth` in the graph representing the parent.
            if let Some(parent_idx) = maybe_parent_idx(self) {
                self.set_edge(parent_idx, node_idx, Edge::Depth).unwrap();
            }

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
                    container.maybe_x_scroll_state = maybe_x_scroll_state;
                    container.maybe_y_scroll_state = maybe_y_scroll_state;
                    container.instantiation_order_idx = instantiation_order_idx;
                },

            }

        // Otherwise if there is no Widget for the given index we need to add one.
        //
        // If there is no widget for the given index we can assume that the index is a
        // `widget::Id`, as the only way to procure a NodeIndex is by adding a Widget to the
        // Graph.
        } else if let Some(widget_id) = self.widget_id(idx) {
            let node_idx = self.add_node(Node::Widget(new_container()));
            self.index_map.insert(widget_id, node_idx);
            if let Some(parent_idx) = maybe_parent_idx(self) {
                self.set_edge(parent_idx, node_idx, Edge::Depth).unwrap();
            }
        }

        // Now that we've updated the widget's cached data, we need to check if we should add any
        // `Edge::Position`s.
        //
        // If the widget is *not* positioned relatively to any other widget, we should ensure that
        // there are no incoming `Position` edges.

        // X
        if let Some(relative_idx) = maybe_x_positioned_relatively_idx {
            self.set_edge(relative_idx, idx, Edge::Position(Axis::X)).unwrap();
        } else {
            self.remove_parent_edge(idx, Edge::Position(Axis::X));
        }

        // Y
        if let Some(relative_idx) = maybe_y_positioned_relatively_idx {
            self.set_edge(relative_idx, idx, Edge::Position(Axis::Y)).unwrap();
        } else {
            self.remove_parent_edge(idx, Edge::Position(Axis::Y));
        }

        // Check whether or not the widget is a graphics element for some other widget.
        if let Some(graphic_parent_idx) = maybe_graphics_for {
            self.set_edge(graphic_parent_idx, idx, Edge::Graphic).unwrap();
        // If not, ensure that there is no parent **Graphic** edge from the widget.
        } else {
            self.remove_parent_edge(idx, Edge::Graphic);
        }

    }

    /// Cache some `PostUpdateCache` widget data into the graph.
    ///
    /// This is called (via the `ui` module) from within the `widget::set_widget` function after
    /// the `Widget::update` method is called and some new state is returned.
    pub fn post_update_cache<B, W>(&mut self, widget: widget::PostUpdateCache<B, W>)
        where B: Backend,
              W: Widget<B>,
              W::State: 'static,
              W::Style: 'static,
    {
        let widget::PostUpdateCache { idx, state, style, .. } = widget;

        // We know that their must be a NodeIndex for this idx, as `Graph::pre_update_cache` will
        // always be called prior to this method being called.
        if let Some(ref mut container) = self.widget_mut(idx) {

            // Construct the `UniqueWidgetState` ready to store as an `Any` within the container.
            let unique_state: UniqueWidgetState<W::State, W::Style> = UniqueWidgetState {
                state: state,
                style: style,
            };

            container.maybe_state = Some(Box::new(unique_state));
        }
    }

}


fn is_depth_edge(g: &Graph, e: EdgeIndex, _: NodeIndex) -> bool {
    g[e] == Edge::Depth
}

fn is_x_position_edge(g: &Graph, e: EdgeIndex, _: NodeIndex) -> bool {
    g[e] == Edge::Position(Axis::X)
}

fn is_y_position_edge(g: &Graph, e: EdgeIndex, _: NodeIndex) -> bool {
    g[e] == Edge::Position(Axis::Y)
}

fn is_graphic_edge(g: &Graph, e: EdgeIndex, _: NodeIndex) -> bool {
    g[e] == Edge::Graphic
}


impl Walker<Graph> for Children {
    type Index = u32;
    #[inline]
    fn next(&mut self, graph: &Graph) -> Option<(EdgeIndex, NodeIndex)> {
        self.next(&graph.dag)
    }
}

impl Walker<Graph> for Parents {
    type Index = u32;
    #[inline]
    fn next(&mut self, graph: &Graph) -> Option<(EdgeIndex, NodeIndex)> {
        self.next(&graph.dag)
    }
}


impl<I: GraphIndex> ::std::ops::Index<I> for Graph {
    type Output = Node;
    fn index<'a>(&'a self, idx: I) -> &'a Node {
        self.node(idx).unwrap()
    }
}

impl<I: GraphIndex> ::std::ops::IndexMut<I> for Graph {
    fn index_mut<'a>(&'a mut self, idx: I) -> &'a mut Node {
        self.node_mut(idx).unwrap()
    }
}

impl Index<EdgeIndex> for Graph {
    type Output = Edge;
    fn index<'a>(&'a self, idx: EdgeIndex) -> &'a Edge {
        self.edge(idx).unwrap()
    }
}

impl IndexMut<EdgeIndex> for Graph {
    fn index_mut<'a>(&'a mut self, idx: EdgeIndex) -> &'a mut Edge {
        self.edge_mut(idx).unwrap()
    }
}
