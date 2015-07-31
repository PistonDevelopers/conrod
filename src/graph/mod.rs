

use elmesque::{Element, Renderer};
use elmesque::element::layers;
use graphics::Graphics;
use graphics::character::CharacterCache;
use petgraph as pg;
use position::{Depth, Dimensions, Point};
use self::index_map::{IndexMap, GraphIndex};
use std::any::Any;
use std::fmt::Debug;
use widget::{self, Widget, WidgetId};


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
    /// Whether or not the widget has been updated since the last time it was drawn.
    pub has_updated: bool,
    /// The area in which child widgets are placed.
    pub kid_area: widget::KidArea,
    /// Whether or not the widget is a "Floating" widget.
    /// See the `Widget::float` docs for an explanation of what this means.
    pub maybe_floating: Option<widget::Floating>,
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
    depth_order: Vec<NodeIndex>,
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

    /// If there is a Widget for the given index, return a reference to it.
    pub fn get_widget<I: GraphIndex>(&self, idx: I) -> Option<&Container> {
        let Graph { ref index_map, ref graph, .. } = *self;
        index_map.to_node_index(idx).and_then(|idx| match &graph[idx] {
            &Node::Widget(ref container) => Some(container),
            &Node::Placeholder => None,
            _ => unreachable!(),
        })
    }

    /// If there is a Widget for the given Id, return a mutable reference to it.
    pub fn get_widget_mut<I: GraphIndex>(&mut self, idx: I) -> Option<&mut Container> {
        let Graph { ref index_map, ref mut graph, .. } = *self;
        index_map.to_node_index(idx).and_then(move |idx| match &mut graph[idx] {
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
        self.index_map.to_node_index(idx).and_then(|idx| {
            self.graph.neighbors_directed(idx, pg::Incoming).next()
                .and_then(|parent_idx| J::from_idx(parent_idx, &self.index_map))
        })
    }

    /// Returns whether or not the graph contains a widget with the given ID.
    pub fn contains<I: GraphIndex>(&self, idx: I) -> bool {
        self.index_map.to_node_index(idx).is_some()
    }


    /// If the given Point is currently on a Widget, return an index to that widget.
    pub fn pick_widget<I: GraphIndex>(&self, xy: Point) -> Option<I> {
        let Graph { ref depth_order, ref graph, ref index_map, .. } = *self;
        depth_order.iter().rev().find(|&&idx| {
            if let Some(&Node::Widget(ref container)) = graph.node_weight(idx) {
                if ::utils::is_over_rect(container.xy, xy, container.dim) {
                    return true
                }
            }
            false
        }).map(|&idx| I::from_idx(idx, index_map).expect("No matching index"))
    }


    /// Set the parent for the given Widget Id.
    /// This method clears all other incoming edges and ensures that the widget only has a single
    /// parent (incoming edge). This means we can be sure of retaining a tree structure.
    pub fn set_parent_for_widget<I, P>(&mut self, idx: I, maybe_parent_idx: Option<P>) where
        I: GraphIndex,
        P: GraphIndex,
    {
        let Graph { ref mut graph, ref mut index_map, root, .. } = *self;
        let node_idx = index_map.to_node_index(idx).expect("No NodeIndex for given GraphIndex");
        // If no parent id was given, we will set the root as the parent.
        let parent_node_idx = match maybe_parent_idx {
            Some(parent_idx) => match index_map.to_node_index(parent_idx) {
                Some(parent_node_idx) => parent_node_idx,
                // Add a temporary node to the graph at the given parent index so that we can add
                // the edge even in the parent widget's absense. The temporary node should be
                // replaced by the proper widget when it is updated later in the cycle.
                None => {
                    // We *know* that this must be a WidgetId as `.to_node_indx` returned None.
                    let parent_widget_id = parent_idx.expect_widget_id();
                    // Add a placeholder node to act as a parent until the actual parent is placed.
                    let parent_node_idx = graph.add_node(Node::Placeholder);
                    index_map.insert(parent_widget_id, parent_node_idx);
                    parent_node_idx
                },
            },
            None => root,
        };

        // Walk our incoming edges and remove any that aren't connected to our requested parent.
        let mut incoming_edges = graph.walk_edges_directed(node_idx, pg::Incoming);
        let mut already_connected = false;
        while let Some((in_edge_idx, in_node_idx)) = incoming_edges.next_neighbor(graph) {
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


    /// Add a widget to the Graph.
    ///
    /// If a WidgetId is given, create a mapping within the index_map.
    ///
    /// Set the parent of the new widget with the given parent index (or `root` if no parent
    /// index is given). Return the NodeIndex for the Widget's position within the Graph.
    pub fn add_widget<I: GraphIndex>(&mut self,
                                     container: Container,
                                     maybe_id: Option<WidgetId>,
                                     maybe_parent_idx: Option<I>) -> NodeIndex
    {
        let node_idx = self.graph.add_node(Node::Widget(container));
        if let Some(id) = maybe_id {
            self.index_map.insert(id, node_idx);
        }
        self.set_parent_for_widget(node_idx, maybe_parent_idx);
        node_idx
    }


    /// Update the state of the widget with the given WidgetId.
    /// If there is no widget for the given WidgetId, add it to the graph.
    pub fn update_widget<I, W>(&mut self,
                               id: WidgetId,
                               maybe_parent_idx: Option<I>,
                               kind: &'static str,
                               cached: widget::Cached<W>,
                               maybe_new_element: Option<Element>)
        where
            I: GraphIndex,
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
                has_updated: true,
                kid_area: kid_area,
                maybe_floating: maybe_floating,
            }
        };

        match self.contains(id) {

            // If there is no Widget for the given ID we need to add one.
            false => {
                let container = new_container(stored, maybe_new_element);
                self.add_widget(container, Some(id), maybe_parent_idx);
            },

            // If we already have a Widget for the given ID, we need to update it.
            true => {
                self.set_parent_for_widget(id, maybe_parent_idx);

                // We can unwrap here because we know that there is a matching index.
                let node_idx = self.index_map.get_node_index(id)
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
                                    id, &kind, container.kind);
                        }

                        container.maybe_state = Some(Box::new(stored));
                        container.kind = kind;
                        container.xy = xy;
                        container.dim = dim;
                        container.depth = depth;
                        container.drag_state = drag_state;
                        container.has_updated = true;
                        container.kid_area = kid_area;
                        container.maybe_floating = maybe_floating;
                        if let Some(new_element) = maybe_new_element {
                            container.element = new_element;
                        }
                    },

                    // The node that we're updating should only be either a Placeholder or a Widget.
                    _ => unreachable!(),
                }
            },

        };
    }


    /// Draw all widgets within the entire Graph.
    ///
    /// The order in which we will draw all widgets will be a akin to a depth-first search, where
    /// the branches with the highest `depth` are drawn first (unless the branch is on a captured
    /// widget, which will always be drawn last).
    pub fn draw<'a, C, G>(&mut self,
                          maybe_captured_mouse: Option<WidgetId>,
                          maybe_captured_keyboard: Option<WidgetId>,
                          renderer: &mut Renderer<'a, C, G>) where
        C: CharacterCache,
        G: Graphics<Texture = C::Texture>,
    {                   
        self.prepare_to_draw(maybe_captured_mouse, maybe_captured_keyboard);

        // Draw the widgets in order of depth (starting with the deepest).
        for &idx in self.depth_order.iter() {
            if let &mut Node::Widget(ref mut container) = &mut self.graph[idx] {
                if container.has_updated {
                    container.element.draw(renderer);
                    container.has_updated = false;
                }
            }
        }
    }
    
    /// Return an `elmesque::Element` containing all widgets within the entire Graph.
    ///
    /// The order in which we will draw all widgets will be a akin to a depth-first search, where
    /// the branches with the highest `depth` are drawn first (unless the branch is on a captured
    /// widget, which will always be drawn last).
    pub fn element(&mut self,
                   maybe_captured_mouse: Option<WidgetId>,
                   maybe_captured_keyboard: Option<WidgetId>) -> Element
    {
        self.prepare_to_draw(maybe_captured_mouse, maybe_captured_keyboard);

        let Graph {
            ref mut graph,
            ref mut depth_order,
            ..
        } = *self;

        let elements = depth_order.iter().filter_map(|&idx| {
            if let &mut Node::Widget(ref mut container) = &mut graph[idx] {
                if container.has_updated {
                    container.has_updated = false;
                    return Some(container.element.clone());
                }
            }
            None
        }).collect();
        layers(elements)
    }

    // Helper method for logic shared between draw() and element().
    fn prepare_to_draw(&mut self,
                       maybe_captured_mouse: Option<WidgetId>,
                       maybe_captured_keyboard: Option<WidgetId>)
    {
        let Graph {
            ref mut graph,
            ref index_map,
            root,
            ref mut depth_order,
            ref mut floating_deque,
            ..
        } = *self;

        // Ensure that the depth order is up to date.
        update_depth_order(root,
                           maybe_captured_mouse.and_then(|i| index_map.get_node_index(i)),
                           maybe_captured_keyboard.and_then(|i| index_map.get_node_index(i)),
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
                      depth_order: &mut Vec<NodeIndex>,
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
                  depth_order: &mut Vec<NodeIndex>,
                  floating_deque: &mut Vec<NodeIndex>)
{
    // First, store the index of the current node.
    match &graph[idx] {
        &Node::Widget(ref container) if container.has_updated => {
            depth_order.push(idx);
        },
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
}


impl ::std::ops::Index<WidgetId> for Graph {
    type Output = Container;
    fn index<'a>(&'a self, id: WidgetId) -> &'a Container {
        self.get_widget(id).expect("No Widget matching the given ID")
    }
}

impl ::std::ops::IndexMut<WidgetId> for Graph {
    fn index_mut<'a>(&'a mut self, id: WidgetId) -> &'a mut Container {
        self.get_widget_mut(id).expect("No Widget matching the given ID")
    }
}

