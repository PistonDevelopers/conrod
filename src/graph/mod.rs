

use {Scalar};
use elmesque::Element;
use elmesque::element::layers;
use petgraph as pg;
use position::{Depth, Dimensions, Point};
use self::index_map::IndexMap;
use std::any::Any;
use std::fmt::Debug;
use widget::{self, Widget};


pub use self::index_map::GraphIndex;

mod index_map;


/// An alias for our Graph's Node Index.
pub type NodeIndex = pg::graph::NodeIndex<u32>;

/// An alias for our Graph's Edge Index.
type EdgeIndex = pg::graph::EdgeIndex<u32>;

/// The state type that we'll dynamically cast to and from Any for storage within the Cache.
#[derive(Debug)]
pub struct StoredWidget<Sta, Sty>
    where
        Sta: Any + Debug,
        Sty: Any + Debug,
{
    pub state: Sta,
    pub style: Sty,
}

/// A container for storing a Widget's state inside the Cache.
#[derive(Debug)]
pub struct Container {
    /// Dynamically stored widget state.
    pub maybe_state: Option<Box<Any>>,
    /// A unique widget kind identifier.
    pub kind: &'static str,
    /// The dimensions of the Widget's bounding rectangle.
    pub dim: Dimensions,
    /// Centered coords of the widget's position.
    pub xy: Point,
    /// The depth at which the widget will be rendered comparatively to its siblings.
    pub depth: Depth,
    /// The drag state of the Widget.
    pub drag_state: widget::drag::State,
    /// The element used for drawing the widget.
    pub element: Element,
    /// Whether or not the `Widget` has had `.set` called since the last cycle.
    pub has_set: bool,
    /// The area in which child widgets are placed.
    pub kid_area: widget::KidArea,
    /// Whether or not the widget is a "Floating" widget.
    /// See the `Widget::float` docs for an explanation of what this means.
    pub maybe_floating: Option<widget::Floating>,
    /// Scroll related state (is only `Some` if the widget is scrollable)..
    pub maybe_scrolling: Option<widget::scroll::State>,
    /// Whether or not the `Element` for the widget has changed since the last time an `Element`
    /// was requested from the graph.
    pub element_has_changed: bool,
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

/// An alias for the petgraph::Graph used within our Ui Graph.
type PetGraph = pg::Graph<Node, (), pg::Directed>;

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
    /// Cached widget state in a graph whose edges describe the rendering tree and positioning.
    graph: PetGraph,
    /// A map of the UiId to the graph's indices.
    index_map: IndexMap,
    /// The NodeIndex of the root Node.
    root: NodeIndex,
    /// Contains Node indices in order of depth, starting with the deepest.
    /// This is updated at the beginning of the `Graph::draw` method.
    depth_order: Vec<Visitable>,
    /// Used for storing indices of "floating" widgets during depth sorting so that they may be
    /// visited after widgets of the root tree.
    floating_deque: Vec<NodeIndex>,
}


impl Container {

    /// Take the widget state from the container and cast it to type W.
    pub fn take_widget_state<W>(&mut self) -> Option<widget::Cached<W>>
        where
            W: Widget,
            W::State: Any + 'static,
            W::Style: Any + 'static,
    {

        let Container {
            ref mut maybe_state,
            dim,
            xy,
            depth,
            drag_state,
            kid_area,
            maybe_floating,
            maybe_scrolling,
            ..
        } = *self;

        maybe_state.take().map(|any_state| {
            let store: Box<StoredWidget<W::State, W::Style>> = any_state.downcast()
                .ok().expect("Failed to downcast from Box<Any> to required widget::Store.");
            let store: StoredWidget<W::State, W::Style> = *store;
            let StoredWidget { state, style } = store;
            widget::Cached {
                state: state,
                style: style,
                dim: dim,
                xy: xy,
                depth: depth,
                drag_state: drag_state,
                kid_area: kid_area,
                maybe_floating: maybe_floating,
                maybe_scrolling: maybe_scrolling,
            }
        })
    }

}


impl Graph {

    /// Construct a new Graph with the given capacity.
    pub fn with_capacity(capacity: usize) -> Graph {
        let mut graph = PetGraph::with_capacity(capacity, capacity);
        let root = graph.add_node(Node::Root);
        Graph {
            graph: graph,
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
        self.graph.add_node(Node::Placeholder)
    }

    /// If there is a Widget for the given index, return a reference to it.
    pub fn get_widget<I: GraphIndex>(&self, idx: I) -> Option<&Container> {
        let Graph { ref index_map, ref graph, .. } = *self;
        idx.to_node_index(index_map).and_then(|idx| match &graph[idx] {
            &Node::Widget(ref container) => Some(container),
            &Node::Placeholder => None,
            _ => unreachable!(),
        })
    }

    /// If there is a Widget for the given Id, return a mutable reference to it.
    pub fn get_widget_mut<I: GraphIndex>(&mut self, idx: I) -> Option<&mut Container> {
        let Graph { ref index_map, ref mut graph, .. } = *self;
        idx.to_node_index(index_map).and_then(move |idx| match &mut graph[idx] {
            &mut Node::Widget(ref mut container) => Some(container),
            &mut Node::Placeholder => None,
            _ => unreachable!(),
        })
    }

    /// Return the id of the parent for the widget at the given index.
    pub fn parent_of<I, J>(&self, idx: I) -> Option<J> where
        I: GraphIndex,
        J: GraphIndex,
    {
        idx.to_node_index(&self.index_map).and_then(|idx| {
            self.graph.neighbors_directed(idx, pg::Incoming).next()
                .and_then(|parent_idx| J::from_idx(parent_idx, &self.index_map))
        })
    }

    /// Returns whether or not the graph contains a widget with the given ID.
    pub fn contains<I: GraphIndex>(&self, idx: I) -> bool {
        idx.to_node_index(&self.index_map).is_some()
    }


    /// If the given Point is currently on a Widget, return an index to that widget.
    pub fn pick_widget<I: GraphIndex>(&self, xy: Point) -> Option<I> {
        let Graph { ref depth_order, ref graph, ref index_map, .. } = *self;
        depth_order.iter().rev()
            .find(|&&visitable| {
                match visitable {
                    Visitable::Widget(idx) => {
                        if let Some(&Node::Widget(ref container)) = graph.node_weight(idx) {
                            if ::utils::is_over_rect(container.xy, xy, container.dim) {
                                return true
                            }
                        }
                    },
                    Visitable::Scrollbar(idx) => {
                        if let Some(&Node::Widget(ref container)) = graph.node_weight(idx) {
                            if let Some(ref scrolling) = container.maybe_scrolling {
                                if widget::scroll::is_over(scrolling, &container.kid_area, xy) {
                                    return true;
                                }
                            }
                        }
                    },
                }
                false
            })
            .map(|&visitable| match visitable {
                Visitable::Widget(idx) | Visitable::Scrollbar(idx) =>
                    I::from_idx(idx, index_map).expect("No matching index"),
            })
    }


    /// If the given Point is currently over a scrollable widget, return an index to that widget.
    pub fn pick_top_scrollable_widget<I: GraphIndex>(&self, xy: Point) -> Option<I> {
        let Graph { ref depth_order, ref graph, ref index_map, .. } = *self;
        depth_order.iter().rev()
            .filter_map(|&visitable| match visitable {
                Visitable::Widget(idx) => Some(idx),
                Visitable::Scrollbar(_) => None,
            })
            .find(|&idx| {
                if let Some(&Node::Widget(ref container)) = graph.node_weight(idx) {
                    if container.maybe_scrolling.is_some() {
                        if ::utils::is_over_rect(container.xy, xy, container.dim) {
                            return true;
                        }
                    }
                }
                false
            })
            .map(|idx| I::from_idx(idx, index_map).expect("No matching index"))
    }


    /// Calculate the total scroll offset for the widget with the given widget::Index.
    pub fn scroll_offset<I: GraphIndex>(&self, idx: I) -> Point {
        let Graph { ref graph, ref index_map, .. } = *self;
        
        let mut offset = [0.0, 0.0];
        let mut idx = match idx.to_node_index(index_map) {
            Some(idx) => idx,
            // If the ID is not yet present within the graph, return the zeroed offset.
            None => return offset,
        };

        // We know that our graph shouldn't cycle at all, so we can safely use loop to traverse all
        // parent widget nodes and return when there are no more.
        loop {

            // We know that we should only have one incoming edge as we only have one parent.
            idx = match graph.neighbors_directed(idx, pg::Incoming).next() {
                None => return offset,
                Some(parent_idx) => parent_idx,
            };

            if let Some(&Node::Widget(ref container)) = graph.node_weight(idx) {
                if let Some(ref scrolling) = container.maybe_scrolling {

                    // Vertical offset.
                    if let Some(ref bar) = scrolling.maybe_vertical {
                        let offset_frac = bar.offset / bar.max_offset;
                        let visible_height = container.kid_area.dim[1];
                        let y_offset = offset_frac * (bar.total_length - visible_height);
                        offset[1] += y_offset;
                    }

                    // Horizontal offset.
                    if let Some(ref bar) = scrolling.maybe_horizontal {
                        let offset_frac = bar.offset / bar.max_offset;
                        let visible_width = container.kid_area.dim[0];
                        let x_offset = offset_frac * (bar.total_length - visible_width);
                        offset[0] -= x_offset;
                    }
                }
            }
        }
    }


    /// Set the parent for the given Widget Id.
    /// This method clears all other incoming edges and ensures that the widget only has a single
    /// parent (incoming edge). This means we can be sure of retaining a tree structure.
    pub fn set_parent_for_widget<I, P>(&mut self, idx: I, maybe_parent_idx: Option<P>) where
        I: GraphIndex,
        P: GraphIndex,
    {
        let Graph { ref mut graph, ref mut index_map, root, .. } = *self;

        let node_idx = idx.to_node_index(index_map).expect("No NodeIndex for given GraphIndex");
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
                        .expect("No matching WidgetId for parent_idx");
                    // Add a placeholder node to act as a parent until the actual parent is placed.
                    let parent_node_idx = graph.add_node(Node::Placeholder);
                    index_map.insert(parent_widget_id, parent_node_idx);
                    parent_node_idx
                },
            },
            None => root,
        };

        // Check to see if the node already has some parent.
        // Remove the parent if it's not the same as our new parent_node_idx.
        // Keep it if it's the one we want.
        let mut incoming_edges = graph.walk_edges_directed(node_idx, pg::Incoming);
        let mut already_connected = false;
        // Note that we only need to check for *one* parent as there can only ever be one parent
        // per node. We know this, as this method is the only public method that adds edges.
        if let Some((in_edge_idx, in_node_idx)) = incoming_edges.next_neighbor(graph) {
            if in_node_idx == parent_node_idx {
                already_connected = true;
            } else {
                graph.remove_edge(in_edge_idx);
            }
        }

        // If we don't already have an incoming edge from the requested parent, add one.
        if !already_connected {
            graph.add_edge(parent_node_idx, node_idx, ());

            // We can't allow the new connection to cause a cycle.
            if let Some(parent_idx) = maybe_parent_idx {
                if pg::algo::is_cyclic_directed(graph) {
                    panic!("Adding widget (WidgetId: {:?}, NodeIndex: {:?}) with the given \
                            parent (WidgetId: {:?}, NodeIndex: {:?}) caused a cycle within the \
                            Ui Graph.\n{:?}", idx, node_idx, parent_idx, parent_node_idx, graph);
                }
            }
        }
    }


    /// The box that bounds the widget with the given ID as well as all its widget kids.
    /// Bounds are given as (max y, min y, min x, max x) from the given target xy position.
    /// If no target xy is given, the bounds will be given relative to the centre of the widget.
    /// If `use_kid_area` is true, then bounds will be calculated relative to the centre of the
    /// `kid_area` of the widget, rather than the regular dimensions.
    pub fn bounding_box<I: GraphIndex>(&self,
                                       include_self: bool,
                                       target_xy: Option<Point>,
                                       use_kid_area: bool,
                                       idx: I) -> Option<(Scalar, Scalar, Scalar, Scalar)>
    {
        let Graph { ref graph, ref index_map, .. } = *self;

        if let Some(idx) = idx.to_node_index(index_map) {
            if let &Node::Widget(ref container) = &graph[idx] {

                // If we're to use the kid area, we'll get the dim and xy from that.
                let (dim, xy) = if use_kid_area {
                    (container.kid_area.dim, container.kid_area.xy)

                // Otherwise we'll use the regular dim and xy.
                } else {
                    (container.dim, container.xy)
                };

                let target_xy = target_xy.unwrap_or(xy);
                let self_bounds = || {
                    let x_diff = xy[0] - target_xy[0];
                    let y_diff = xy[1] - target_xy[1];
                    let half_w = dim[0] / 2.0;
                    let half_h = dim[1] / 2.0;
                    let top_y = y_diff + half_h;
                    let bottom_y = y_diff - half_h;
                    let left_x = x_diff - half_w;
                    let right_x = x_diff + half_w;
                    (top_y, bottom_y, left_x, right_x)
                };

                // Filter the neighbours so only widget kids' xy and dim are produced.
                let mut kids = graph.neighbors_directed(idx, pg::Outgoing)
                    .filter_map(|kid_idx| self.bounding_box(true, Some(target_xy), false, kid_idx));

                // Work out the initial bounds to use for our max_bounds fold.
                let init_bounds = if include_self {
                    self_bounds()
                } else {
                    match kids.next() {
                        Some(first_kid_bounds) => first_kid_bounds,
                        None => return None,
                    }
                };

                return Some(kids.fold(init_bounds, |max_so_far, kid_bounds| {

                    // max y, min y, min x, max x.
                    type Bounds = (Scalar, Scalar, Scalar, Scalar);

                    // Returns the bounds for the two given sets of bounds.
                    fn max_bounds(a: Bounds, b: Bounds) -> Bounds {
                        (a.0.max(b.0), a.1.min(b.1), a.2.min(b.2), a.3.max(b.3))
                    }

                    max_bounds(max_so_far, kid_bounds)
                }));
            }
        }

        None
    }


    /// Add a widget to the Graph.
    ///
    /// If a WidgetId is given, create a mapping within the index_map.
    ///
    /// Set the parent of the new widget with the given parent index (or `root` if no parent
    /// index is given). Return the NodeIndex for the Widget's position within the Graph.
    pub fn add_widget<I: GraphIndex>(&mut self,
                                     container: Container,
                                     maybe_id: Option<widget::Id>,
                                     maybe_parent_idx: Option<I>) -> NodeIndex
    {
        let node_idx = self.graph.add_node(Node::Widget(container));
        if let Some(id) = maybe_id {
            self.index_map.insert(id, node_idx);
        }
        self.set_parent_for_widget(node_idx, maybe_parent_idx);
        node_idx
    }


    /// Update the state of the widget with the given widget::Index.
    /// If there is no widget for the given widget::Index, add it to the graph.
    pub fn update_widget<I, P, W>(&mut self,
                                  idx: I,
                                  maybe_parent_idx: Option<P>,
                                  kind: &'static str,
                                  cached: widget::Cached<W>,
                                  maybe_new_element: Option<Element>)
        where
            I: GraphIndex,
            P: GraphIndex,
            W: Widget,
            W::State: 'static,
            W::Style: 'static,
    {

        // Destructure the members from the Cached.
        let widget::Cached {
            state,
            style,
            xy,
            dim,
            depth,
            drag_state,
            kid_area,
            maybe_floating,
            maybe_scrolling,
        } = cached;

        let stored: StoredWidget<W::State, W::Style> =
            StoredWidget { state: state, style: style };

        // Construct a new container. This is used if:
        // - There is not yet any node matching the given ID or
        // - The node at the given ID is a placeholder.
        let new_container = |stored: StoredWidget<W::State, W::Style>,
                             maybe_new_element: Option<Element>| {
            Container {
                maybe_state: Some(Box::new(stored)),
                kind: kind,
                xy: xy,
                dim: dim,
                depth: depth,
                drag_state: drag_state,
                element: maybe_new_element.unwrap_or_else(|| ::elmesque::element::empty()),
                has_set: true,
                kid_area: kid_area,
                maybe_floating: maybe_floating,
                maybe_scrolling: maybe_scrolling,
                element_has_changed: true,
            }
        };

        // If we already have a Widget for the given ID, we need to update it.
        if self.contains(idx) {
            self.set_parent_for_widget(idx, maybe_parent_idx);

            // We can unwrap here because we know that there is a matching index.
            let node_idx = idx.to_node_index(&self.index_map)
                .expect("No matching NodeIndex");

            match &mut self.graph[node_idx] {

                // If the node is currently a placeholder, construct the widget variant.
                node @ &mut Node::Placeholder => {
                    let container = new_container(stored, maybe_new_element);
                    *node = Node::Widget(container);
                },

                // Otherwise, update the container that already exists.
                &mut Node::Widget(ref mut container) => {
                    // If the container already exists with the state of some other kind of
                    // widget, we can assume there's been a mistake with the given Id.
                    if container.kind != kind && container.kind != "EMPTY" {
                        panic!("A widget of a different kind already exists at the given UiId \
                                ({:?}). You tried to insert a {:?}, however the existing \
                                widget is a {:?}. Check your widgets' `UiId`s for errors.",
                                idx, &kind, container.kind);
                    }

                    container.maybe_state = Some(Box::new(stored));
                    container.kind = kind;
                    container.xy = xy;
                    container.dim = dim;
                    container.depth = depth;
                    container.drag_state = drag_state;
                    container.has_set = true;
                    container.kid_area = kid_area;
                    container.maybe_floating = maybe_floating;
                    container.maybe_scrolling = maybe_scrolling;
                    if let Some(new_element) = maybe_new_element {
                        container.element = new_element;
                        container.element_has_changed = true;
                    }
                },

                // The node that we're updating should only be either a Placeholder or a Widget.
                _ => unreachable!(),
            }

        // Otherwise if there is no Widget for the given index we need to add one.
        } else {
            // If there is no widget for the given index we can assume that the index is a
            // `widget::Id`, as the only way to procure a NodeIndex is by adding a Widget to the
            // Graph.
            let id = idx.to_widget_id(&self.index_map)
                .expect("Expected a `WidgetId` but the given idx was not one, nor did it match any \
                        known `WidgetId`s within the `Graph`'s `IndexMap`.");
            let container = new_container(stored, maybe_new_element);
            self.add_widget(container, Some(id), maybe_parent_idx);
        }

    }


    /// Return an `elmesque::Element` containing all widgets within the entire Graph.
    ///
    /// The order in which we will draw all widgets will be a akin to a depth-first search, where
    /// the branches with the highest `depth` are drawn first (unless the branch is on a captured
    /// widget, which will always be drawn last).
    pub fn element<M, K>(&mut self,
                         maybe_captured_mouse: Option<M>,
                         maybe_captured_keyboard: Option<K>) -> Element
        where
            M: GraphIndex,
            K: GraphIndex,
    {
        // Convert the GraphIndex for the widget capturing the mouse into a NodeIndex.
        let maybe_captured_mouse = maybe_captured_mouse
            .and_then(|idx| idx.to_node_index(&self.index_map));
        // Convert the GraphIndex for the widget capturing the keyboard into a NodeIndex.
        let maybe_captured_keyboard = maybe_captured_keyboard
            .and_then(|idx| idx.to_node_index(&self.index_map));

        self.prepare_to_draw(maybe_captured_mouse, maybe_captured_keyboard);

        let Graph { ref mut graph, ref depth_order, .. } = *self;

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
                    if let &mut Node::Widget(ref mut container) = &mut graph[idx] {
                        if container.has_set {

                            container.has_set = false;
                            container.element_has_changed = false;

                            if let Some(scroll_group) = scroll_stack.last_mut() {
                                // If there is some current scroll group, we'll push to that.
                                scroll_group.push(container.element.clone());
                            } else {
                                // Otherwise, we'll push straight to our main elements Vec.
                                elements.push(container.element.clone());
                            }

                            // If the current widget is some scrollable widget, we need to add a
                            // new group to the top of our scroll stack.
                            if container.maybe_scrolling.is_some() {
                                scroll_stack.push(Vec::new());
                            }
                        }
                    }
                },

                Visitable::Scrollbar(idx) => {
                    if let &Node::Widget(ref container) = &graph[idx] {
                        if let Some(scrolling) = container.maybe_scrolling {

                            // Now that we've come across a scrollbar, we should pop the group of
                            // elements from the top of our scrollstack for cropping.
                            if let Some(scroll_group) = scroll_stack.pop() {
                                let xy = container.kid_area.xy;
                                let dim = container.kid_area.dim;
                                let element = layers(scroll_group)
                                    .crop(xy[0], xy[1], dim[0], dim[1]);
                                elements.push(element);
                            }

                            // Construct the element for the scrollbar itself.
                            let element = widget::scroll::element(&container.kid_area, scrolling);
                            elements.push(element);
                        }
                    }
                },

            }
        }

        // Convert the Vec<Element> into a single `Element` and return it.
        layers(elements)
    }


    /// Same as `Graph::element`, but only returns a new `Element` if any of the widgets'
    /// `Element`s in the graph have changed.
    pub fn element_if_changed<M, K>(&mut self,
                                    maybe_captured_mouse: Option<M>,
                                    maybe_captured_keyboard: Option<K>) -> Option<Element>
        where
            M: GraphIndex,
            K: GraphIndex,
    {
        // Check whether or not any of the widget's `Element`s have changed.
        let mut has_changed = false;
        for node in self.graph.raw_nodes().iter() {
            if let Node::Widget(ref container) = node.weight {
                if container.element_has_changed {
                    has_changed = true;
                    break;
                }
            }
        }

        match has_changed {
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
            ref mut graph,
            root,
            ref mut depth_order,
            ref mut floating_deque,
            ..
        } = *self;

        // Ensure that the depth order is up to date.
        update_depth_order(root,
                           maybe_captured_mouse,
                           maybe_captured_keyboard,
                           graph,
                           depth_order,
                           floating_deque);
    }
}

/// Update the depth_order (starting with the deepest) for all nodes in the graph.
/// The floating_deque is a pre-allocated deque used for collecting the floating widgets during
/// visiting so that they may be drawn last.
fn update_depth_order(root: NodeIndex,
                      maybe_captured_mouse: Option<NodeIndex>,
                      maybe_captured_keyboard: Option<NodeIndex>,
                      graph: &PetGraph,
                      depth_order: &mut Vec<Visitable>,
                      floating_deque: &mut Vec<NodeIndex>)
{

    // Clear the buffers and ensure they've enough memory allocated.
    let num_nodes = graph.node_count();
    depth_order.clear();
    depth_order.reserve(num_nodes);
    floating_deque.clear();
    floating_deque.reserve(num_nodes);

    // Visit each node in order of depth and add their indices to depth_order.
    // If the widget is floating, then store it in the floating_deque instead.
    visit_by_depth(root,
                   maybe_captured_mouse,
                   maybe_captured_keyboard,
                   graph,
                   depth_order,
                   floating_deque);

    // Sort the floating widgets so that the ones clicked last come last.
    floating_deque.sort_by(|&a, &b| match (&graph[a], &graph[b]) {
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
                       graph,
                       depth_order,
                       floating_deque);
    }
}


/// Recursive function for visiting all nodes within the graph
fn visit_by_depth(idx: NodeIndex,
                  maybe_captured_mouse: Option<NodeIndex>,
                  maybe_captured_keyboard: Option<NodeIndex>,
                  graph: &PetGraph,
                  depth_order: &mut Vec<Visitable>,
                  floating_deque: &mut Vec<NodeIndex>)
{
    // First, store the index of the current node.
    match &graph[idx] {
        &Node::Widget(ref container) if container.has_set =>
            depth_order.push(Visitable::Widget(idx)),
        &Node::Root => (),
        // If the node is neither an updated widget or the Root, we are done with this branch.
        _ => return,
    }

    // Sort the children of the current node by their `.depth` members.
    // FIXME: We should remove these allocations by storing a `child_sorter` buffer in each Widget
    // node (perhaps in the `Container`).
    let mut child_sorter: Vec<NodeIndex> = graph.neighbors_directed(idx, pg::Outgoing).collect();
    child_sorter.sort_by(|&a, &b| {
        use std::cmp::Ordering;
        if Some(a) == maybe_captured_mouse || Some(a) == maybe_captured_keyboard {
            Ordering::Greater
        } else if let (&Node::Widget(ref a), &Node::Widget(ref b)) = (&graph[a], &graph[b]) {
            b.depth.partial_cmp(&a.depth).expect("Depth was NaN!")
        } else {
            Ordering::Equal
        }
    });

    // Then, visit each of the child widgets. If we come across any floating widgets, we'll store
    // those in the floating deque so that we can visit them following the current tree.
    for child_idx in child_sorter.into_iter() {

        // Determine whether or not the node is a floating widget.
        let maybe_is_floating = match graph.node_weight(child_idx) {
            Some(&Node::Widget(ref container)) => Some(container.maybe_floating.is_some()),
            _                                  => None,
        };

        // Store floating widgets int he floating_deque for visiting after the current tree.
        match maybe_is_floating {
            Some(true) => floating_deque.push(child_idx),
            _          => visit_by_depth(child_idx,
                                         maybe_captured_mouse,
                                         maybe_captured_keyboard,
                                         graph,
                                         depth_order,
                                         floating_deque),
        }
    }

    // If the widget is scrollable, we should add its scrollbar to the visit order also.
    if let &Node::Widget(ref container) = &graph[idx] {
        if container.maybe_scrolling.is_some() {
            depth_order.push(Visitable::Scrollbar(idx));
        }
    }
}


impl<I: GraphIndex> ::std::ops::Index<I> for Graph {
    type Output = Container;
    fn index<'a>(&'a self, idx: I) -> &'a Container {
        self.get_widget(idx).expect("No Widget matching the given ID")
    }
}

impl<I: GraphIndex> ::std::ops::IndexMut<I> for Graph {
    fn index_mut<'a>(&'a mut self, idx: I) -> &'a mut Container {
        self.get_widget_mut(idx).expect("No Widget matching the given ID")
    }
}

