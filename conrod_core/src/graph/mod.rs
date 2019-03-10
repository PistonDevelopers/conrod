//! Conrod uses a directed acyclic graph to manage both storing widgets and describing their
//! relationships.
//!
//! The primary type of interest in this module is the [**Graph**](./struct.Graph) type.

use daggy;
use position::{Axis, Depth, Point, Rect};
use std;
use std::any::Any;
use std::ops::{Index, IndexMut};
use widget::{self, Widget};

pub use daggy::Walker;
pub use self::depth_order::DepthOrder;

pub mod algo;
pub mod depth_order;


/// An alias for our Graph's Edge Index.
pub type EdgeIndex = daggy::EdgeIndex<u32>;

/// An alias for a tuple containing an associated `Edge/widget::Id` pair.
pub type IndexPair = (EdgeIndex, widget::Id);

/// A **Walker** over some node's parent nodes.
pub type Parents = daggy::Parents<Node, Edge, u32>;
/// A **Walker** over some node's child nodes.
pub type Children = daggy::Children<Node, Edge, u32>;

/// An alias for the iterator yielding both **X** and **Y** **Position** parents.
pub type PositionParents =
    std::iter::Chain<std::option::IntoIter<widget::Id>, std::option::IntoIter<widget::Id>>;

/// An alias for some filtered children walker.
pub type FilteredChildren =
    daggy::walker::Filter<Children, fn(&Graph, EdgeIndex, widget::Id) -> bool>;
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
    pub maybe_state: Option<Box<Any + Send>>,
    /// The unique `TypeId` associated with the `Widget::State`.
    ///
    /// This is equal to `std::any::TypeId::of::<Widget::State>()`.
    pub type_id: std::any::TypeId,
    /// The rectangle describing the Widget's area.
    pub rect: Rect,
    /// The depth at which the widget will be rendered comparatively to its siblings.
    pub depth: Depth,
    /// The area in which child widgets are placed.
    pub kid_area: widget::KidArea,
    /// If widget is draggable and is being dragged, this is where it started
    pub maybe_dragged_from: Option<Point>,
    /// Whether or not the widget is a "Floating" widget.
    ///
    /// See the `Widget::float` docs for an explanation of what this means.
    pub maybe_floating: Option<widget::Floating>,
    /// Whether or not children widgets should be cropped to the `kid_area`.
    pub crop_kids: bool,
    /// Scroll related state (is only `Some` if this axis is scrollable).
    pub maybe_x_scroll_state: Option<widget::scroll::StateX>,
    /// Scroll related state (is only `Some` if this axis is scrollable).
    pub maybe_y_scroll_state: Option<widget::scroll::StateY>,
    /// Represents the Widget's position within the overall instantiation ordering of the widgets.
    ///
    /// i.e. if foo's `instantiation_order_idx` is lower than bar's, it means that foo was
    /// instantiated before bar.
    pub instantiation_order_idx: usize,
    /// A function specified by the widget to use when determining whether or not a point is over
    /// it.
    ///
    /// NOTE: See `Wiget::is_over` for more details and a note on possible future plans.
    pub is_over: IsOverFn,
}

/// A wrapper around a `widget::IsOverFn` to make implementing `Debug` easier for `Container`.
#[derive(Copy, Clone)]
pub struct IsOverFn(pub widget::IsOverFn);

impl std::fmt::Debug for IsOverFn {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "IsOverFn")
    }
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
}


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
    pub fn unique_widget_state<W>(&self) -> Option<&UniqueWidgetState<W::State, W::Style>>
        where W: Widget,
              W::State: Any + 'static,
              W::Style: Any + 'static,
    {
        self.state_and_style::<W::State, W::Style>()
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
            .filter(|&i| self[widget::Id::new(i)].is_widget())
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

    /// Add the given **Node** to the graph.
    ///
    /// Computes in **O(1)** time.
    ///
    /// Returns the index of the new node.
    fn add_node(&mut self, node: Node) -> widget::Id {
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
    fn set_edge(&mut self, a: widget::Id, b: widget::Id, edge: Edge) -> Result<EdgeIndex, WouldCycle> {
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
    fn remove_parent_edge(&mut self, id: widget::Id, edge: Edge) -> bool {
        if let Some((edge_idx, _)) = self.parents(id).find(self, |g, e, _| g[e] == edge) {
            self.remove_edge(edge_idx);
            return true;
        }
        false
    }

    /// Add a new placeholder node and return it's `widget::Id` into the `Graph`.
    ///
    /// This method is used by the `widget::set_widget` function when some internal widget does not
    /// yet have it's own `widget::Id`.
    pub fn add_placeholder(&mut self) -> widget::Id {
        self.add_node(Node::Placeholder)
    }

    /// Borrow the node at the given **widget::Id** if there is one.
    pub fn node(&self, idx: widget::Id) -> Option<&Node> {
        self.dag.node_weight(idx)
    }

    /// Mutably borrow the node at the given **widget::Id** if there is one.
    pub fn node_mut(&mut self, idx: widget::Id) -> Option<&mut Node> {
        self.dag.node_weight_mut(idx)
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
    pub fn edge_endpoints(&self, idx: EdgeIndex) -> Option<(widget::Id, widget::Id)> {
        self.dag.edge_endpoints(idx)
    }

    /// If there is a Widget for the given index, return a reference to it.
    pub fn widget(&self, idx: widget::Id) -> Option<&Container> {
        self.node(idx).and_then(|node| match *node {
            Node::Widget(ref container) => Some(container),
            _ => None,
        })
    }

    /// If there is a Widget for the given Id, return a mutable reference to it.
    pub fn widget_mut(&mut self, idx: widget::Id) -> Option<&mut Container> {
        self.node_mut(idx).and_then(|node| match *node {
            Node::Widget(ref mut container) => Some(container),
            _ => None,
        })
    }

    /// A **Walker** type that may be used to step through the parents of the given child node.
    pub fn parents(&self, child: widget::Id) -> Parents {
        self.dag.parents(child)
    }

    /// A **Walker** type that recursively walks the **Graph** using the given `recursive_fn`.
    ///
    /// **Panics** If the given start index does not exist within the **Graph**.
    pub fn recursive_walk<F>(&self, start: widget::Id, recursive_fn: F) -> RecursiveWalk<F>
        where F: FnMut(&Self, widget::Id) -> Option<(EdgeIndex, widget::Id)>
    {
        RecursiveWalk::new(start, recursive_fn)
    }

    /// If the widget at the given index has some parent along an **Edge** of the given variant,
    /// return an index to it.
    pub fn edge_parent(&self, idx: widget::Id, edge: Edge) -> Option<widget::Id> {
        self.parents(idx).find(self, |g, e, _| g[e] == edge).map(|(_, n)| n)
    }

    /// Return the index of the parent along the given widget's **Depth** **Edge**.
    pub fn depth_parent(&self, idx: widget::Id) -> Option<widget::Id> {
        self.edge_parent(idx, Edge::Depth)
    }

    /// Return the index of the parent along the given widget's **Position** **Edge**.
    pub fn x_position_parent(&self, idx: widget::Id) -> Option<widget::Id> {
        self.edge_parent(idx, Edge::Position(Axis::X))
    }

    /// Return the index of the parent along the given widget's **Position** **Edge**.
    pub fn y_position_parent(&self, idx: widget::Id) -> Option<widget::Id> {
        self.edge_parent(idx, Edge::Position(Axis::Y))
    }

    /// Produces an iterator yielding the parents along both the **X** and **Y** **Position**
    /// **Edge**s respectively.
    pub fn position_parents(&self, idx: widget::Id) -> PositionParents {
        self.x_position_parent(idx).into_iter().chain(self.y_position_parent(idx))
    }

    /// Return the index of the parent along the given widget's **Graphic** **Edge**.
    pub fn graphic_parent(&self, idx: widget::Id) -> Option<widget::Id> {
        self.edge_parent(idx, Edge::Graphic)
    }

    /// A **Walker** type that recursively walks **Depth** parents starting from the given node.
    pub fn depth_parent_recursion(&self, idx: widget::Id)
        -> RecursiveWalk<fn(&Graph, widget::Id) -> Option<IndexPair>>
    {
        fn depth_edge_parent(graph: &Graph, idx: widget::Id) -> Option<IndexPair> {
            graph.parents(idx).find(graph, |g, e, _| g[e] == Edge::Depth)
        }
        self.recursive_walk(idx, depth_edge_parent)
    }

    /// A **Walker** type that recursively walks **X** **Position** parents starting from the given
    /// node.
    pub fn x_position_parent_recursion(&self, idx: widget::Id)
        -> RecursiveWalk<fn(&Graph, widget::Id) -> Option<IndexPair>>
    {
        fn x_position_edge_parent(graph: &Graph, idx: widget::Id) -> Option<IndexPair> {
            graph.parents(idx).find(graph, |g, e, _| g[e] == Edge::Position(Axis::X))
        }
        self.recursive_walk(idx, x_position_edge_parent)
    }

    /// A **Walker** type that recursively walks **Y** **Position** parents starting from the given
    /// node.
    pub fn y_position_parent_recursion(&self, idx: widget::Id)
        -> RecursiveWalk<fn(&Graph, widget::Id) -> Option<IndexPair>>
    {
        fn y_position_edge_parent(graph: &Graph, idx: widget::Id) -> Option<IndexPair> {
            graph.parents(idx).find(graph, |g, e, _| g[e] == Edge::Position(Axis::Y))
        }
        self.recursive_walk(idx, y_position_edge_parent)
    }

    /// A **Walker** type that recursively walks **Graphic** parents starting from the given node.
    pub fn graphic_parent_recursion(&self, idx: widget::Id)
        -> RecursiveWalk<fn(&Graph, widget::Id) -> Option<IndexPair>>
    {
        fn graphic_edge_parent(graph: &Graph, idx: widget::Id) -> Option<IndexPair> {
            graph.parents(idx).find(graph, |g, e, _| g[e] == Edge::Graphic)
        }
        self.recursive_walk(idx, graphic_edge_parent)
    }

    /// A **Walker** type that recursively walks **Depth** parents that are scrollable along the
    /// *y* axis for the given node.
    pub fn scrollable_y_parent_recursion(&self, idx: widget::Id)
        -> RecursiveWalk<fn(&Graph, widget::Id) -> Option<IndexPair>>
    {
        fn scrollable_y_parent(graph: &Graph, id: widget::Id) -> Option<IndexPair> {
            let mut depth_parents = graph.depth_parent_recursion(id);
            while let Some((e, n)) = depth_parents.next(graph) {
                if let Some(parent) = graph.widget(n) {
                    if parent.maybe_y_scroll_state.is_some() {
                        return Some((e, n));
                    }
                }
            }
            None
        }
        self.recursive_walk(idx, scrollable_y_parent)
    }

    /// A **Walker** type that recursively walks **Depth** parents that are scrollable along the
    /// *x* axis for the given node.
    pub fn scrollable_x_parent_recursion(&self, idx: widget::Id)
        -> RecursiveWalk<fn(&Graph, widget::Id) -> Option<IndexPair>>
    {
        fn scrollable_x_parent(graph: &Graph, id: widget::Id) -> Option<IndexPair> {
            let mut depth_parents = graph.depth_parent_recursion(id);
            while let Some((e, n)) = depth_parents.next(graph) {
                if let Some(parent) = graph.widget(n) {
                    if parent.maybe_x_scroll_state.is_some() {
                        return Some((e, n));
                    }
                }
            }
            None
        }
        self.recursive_walk(idx, scrollable_x_parent)
    }

    /// A **Walker** type that may be used to step through the children of the given parent node.
    pub fn children(&self, parent: widget::Id) -> Children {
        self.dag.children(parent)
    }

    /// For walking the **Depth** children of the given parent node.
    pub fn depth_children(&self, idx: widget::Id) -> DepthChildren {
        self.children(idx).filter(is_depth_edge)
    }

    /// For walking the **Position(X)** children of the given parent node.
    pub fn x_position_children(&self, idx: widget::Id) -> XPositionChildren {
        self.children(idx).filter(is_x_position_edge)
    }

    /// For walking the **Position(Y)** children of the given parent node.
    pub fn y_position_children(&self, idx: widget::Id) -> YPositionChildren {
        self.children(idx).filter(is_y_position_edge)
    }

    /// For walking the **Position** children of the given parent node.
    ///
    /// This first walks the **Axis::X** children, before walking the **Axis::Y** children.
    pub fn position_children(&self, idx: widget::Id) -> PositionChildren {
        self.x_position_children(idx).chain(self.y_position_children(idx))
    }

    /// For walking the **Graphic** children of the given parent node.
    pub fn graphic_children(&self, idx: widget::Id) -> GraphicChildren {
        self.children(idx).filter(is_graphic_edge)
    }

    /// Does the given edge type exist between the nodes `parent` -> `child`.
    ///
    /// Returns `false` if either of the given node indices do not exist.
    pub fn does_edge_exist<F>(&self, parent: widget::Id, child: widget::Id, is_edge: F) -> bool
        where F: Fn(Edge) -> bool,
    {
        self.parents(child).any(self, |g, e, n| n == parent && is_edge(g[e]))
    }

    /// Does a **Edge::Depth** exist between the nodes `parent` -> `child`.
    ///
    /// Returns `false` if either of the given node indices do not exist.
    pub fn does_depth_edge_exist(&self, parent: widget::Id, child: widget::Id) -> bool {
        self.does_edge_exist(parent, child, |e| e == Edge::Depth)
    }

    /// Does a **Edge::Position** exist between the nodes `parent` -> `child`.
    ///
    /// Returns `false` if either of the given node indices do not exist.
    pub fn does_position_edge_exist(&self, parent: widget::Id, child: widget::Id) -> bool {
        let is_edge = |e| e == Edge::Position(Axis::X) || e == Edge::Position(Axis::Y);
        self.does_edge_exist(parent, child, is_edge)
    }

    /// Does a **Edge::Graphic** exist between the nodes `parent` -> `child`.
    ///
    /// Returns `false` if either of the given node indices do not exist.
    pub fn does_graphic_edge_exist(&self, parent: widget::Id, child: widget::Id) -> bool {
        self.does_edge_exist(parent, child, |e| e == Edge::Graphic)
    }

    /// Are the given `parent` and `child` nodes connected by a single chain of edges of the given
    /// kind?
    ///
    /// i.e. `parent` -> x -> y -> `child`.
    ///
    /// Returns `false` if either of the given node indices do not exist.
    pub fn does_recursive_edge_exist<F>(&self,
                                        parent: widget::Id,
                                        child: widget::Id,
                                        is_edge: F) -> bool
        where F: Fn(Edge) -> bool,
    {
        self.recursive_walk(child, |g, n| g.parents(n).find(g, |g, e, _| is_edge(g[e])))
            .any(self, |_, _, n| n == parent)
    }

    /// Are the given `parent` and `child` nodes connected by a single chain of **Depth** edges?
    ///
    /// i.e. `parent` -> x -> y -> `child`.
    ///
    /// Returns `false` if either of the given node indices do not exist.
    pub fn does_recursive_depth_edge_exist(&self, parent: widget::Id, child: widget::Id) -> bool {
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
    pub fn does_recursive_graphic_edge_exist(&self, parent: widget::Id, child: widget::Id) -> bool {
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
                            root: widget::Id,
                            widget: widget::PreUpdateCache,
                            instantiation_order_idx: usize)
    {
        let widget::PreUpdateCache {
            type_id, id, maybe_parent_id, maybe_x_positioned_relatively_id,
            maybe_y_positioned_relatively_id, rect, depth, kid_area, maybe_dragged_from, maybe_floating,
            crop_kids, maybe_x_scroll_state, maybe_y_scroll_state, maybe_graphics_for, is_over,
        } = widget;

        assert!(self.node(id).is_some(), "No node found for the given widget::Id {:?}", id);

        // Construct a new `Container` to place in the `Graph`.
        let new_container = || Container {
            maybe_state: None,
            type_id: type_id,
            rect: rect,
            depth: depth,
            kid_area: kid_area,
            maybe_dragged_from: maybe_dragged_from,
            maybe_floating: maybe_floating,
            crop_kids: crop_kids,
            maybe_x_scroll_state: maybe_x_scroll_state,
            maybe_y_scroll_state: maybe_y_scroll_state,
            instantiation_order_idx: instantiation_order_idx,
            is_over: IsOverFn(is_over),
        };

        // Retrieves the widget's parent index.
        //
        // `panic!` if the widget does not exist within the graph. This should rarely be the case
        // as all existing `widget::Id`s should be generated from the graph itself.
        //
        // This should only be `None` if the widget is the `root` node (i.e. the `Window` widget).
        let maybe_parent_id = |graph: &mut Self| match maybe_parent_id {
            Some(parent_id) => match graph.node(parent_id).is_some() {
                true => Some(parent_id),
                false => panic!("No node found for the given parent widget::Id {:?}", parent_id),
            },
            // Check that this node is not the root node before using the root node as the parent.
            None => if id == root { None } else { Some(root) },
        };

        // Ensure that we have an `Edge::Depth` in the graph representing the parent.
        if let Some(parent_id) = maybe_parent_id(self) {
            self.set_edge(parent_id, id, Edge::Depth).unwrap();
        }

        match &mut self.dag[id] {

            // If the node is currently a `Placeholder`, construct a new container and use this
            // to set it as the `Widget` variant.
            node @ &mut Node::Placeholder => *node = Node::Widget(new_container()),

            // Otherwise, update the data in the container that already exists.
            &mut Node::Widget(ref mut container) => {

                // If the container already exists with the state of some other kind of
                // widget, we can assume there's been a mistake with the given Id.
                //
                // TODO: It might be overkill to panic here.
                assert!(container.type_id == type_id,
                    "A widget of a different type already exists at the given id \
                    ({:?}). You tried to insert a widget with state of type {:?}, \
                    however the existing widget state is of type {:?}. Check your \
                    `WidgetId`s for errors.",
                    id, &type_id, container.type_id);

                container.type_id = type_id;
                container.rect = rect;
                container.depth = depth;
                container.kid_area = kid_area;
                container.maybe_dragged_from = maybe_dragged_from;
                container.maybe_floating = maybe_floating;
                container.crop_kids = crop_kids;
                container.maybe_x_scroll_state = maybe_x_scroll_state;
                container.maybe_y_scroll_state = maybe_y_scroll_state;
                container.instantiation_order_idx = instantiation_order_idx;
                container.is_over = IsOverFn(is_over);
            },

        }

        // Now that we've updated the widget's cached data, we need to check if we should add any
        // `Edge::Position`s.
        //
        // If the widget is *not* positioned relatively to any other widget, we should ensure that
        // there are no incoming `Position` edges.

        // X
        if let Some(relative_id) = maybe_x_positioned_relatively_id {
            self.set_edge(relative_id, id, Edge::Position(Axis::X)).unwrap();
        } else {
            self.remove_parent_edge(id, Edge::Position(Axis::X));
        }

        // Y
        if let Some(relative_id) = maybe_y_positioned_relatively_id {
            self.set_edge(relative_id, id, Edge::Position(Axis::Y)).unwrap();
        } else {
            self.remove_parent_edge(id, Edge::Position(Axis::Y));
        }

        // Check whether or not the widget is a graphics element for some other widget.
        if let Some(graphic_parent_id) = maybe_graphics_for {
            self.set_edge(graphic_parent_id, id, Edge::Graphic).unwrap();
        // If not, ensure that there is no parent **Graphic** edge from the widget.
        } else {
            self.remove_parent_edge(id, Edge::Graphic);
        }

    }

    /// Cache some `PostUpdateCache` widget data into the graph.
    ///
    /// This is called (via the `ui` module) from within the `widget::set_widget` function after
    /// the `Widget::update` method is called and some new state is returned.
    pub fn post_update_cache<W>(&mut self, widget: widget::PostUpdateCache<W>)
        where W: Widget,
              W::State: 'static,
              W::Style: 'static,
    {
        let widget::PostUpdateCache { id, state, style, .. } = widget;

        // We know that their must be a widget::Id for this id, as `Graph::pre_update_cache` will
        // always be called prior to this method being called.
        if let Some(ref mut container) = self.widget_mut(id) {

            // Construct the `UniqueWidgetState` ready to store as an `Any` within the container.
            let unique_state: UniqueWidgetState<W::State, W::Style> = UniqueWidgetState {
                state: state,
                style: style,
            };

            container.maybe_state = Some(Box::new(unique_state));
        }
    }

}


fn is_depth_edge(g: &Graph, e: EdgeIndex, _: widget::Id) -> bool {
    g[e] == Edge::Depth
}

fn is_x_position_edge(g: &Graph, e: EdgeIndex, _: widget::Id) -> bool {
    g[e] == Edge::Position(Axis::X)
}

fn is_y_position_edge(g: &Graph, e: EdgeIndex, _: widget::Id) -> bool {
    g[e] == Edge::Position(Axis::Y)
}

fn is_graphic_edge(g: &Graph, e: EdgeIndex, _: widget::Id) -> bool {
    g[e] == Edge::Graphic
}


impl Walker<Graph> for Children {
    type Index = u32;
    #[inline]
    fn next(&mut self, graph: &Graph) -> Option<(EdgeIndex, widget::Id)> {
        self.next(&graph.dag)
    }
}

impl Walker<Graph> for Parents {
    type Index = u32;
    #[inline]
    fn next(&mut self, graph: &Graph) -> Option<(EdgeIndex, widget::Id)> {
        self.next(&graph.dag)
    }
}


impl ::std::ops::Index<widget::Id> for Graph {
    type Output = Node;
    fn index<'a>(&'a self, id: widget::Id) -> &'a Node {
        self.node(id).unwrap()
    }
}

impl ::std::ops::IndexMut<widget::Id> for Graph {
    fn index_mut<'a>(&'a mut self, id: widget::Id) -> &'a mut Node {
        self.node_mut(id).unwrap()
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
