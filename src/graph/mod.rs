

use elmesque::{Element, Renderer};
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
}

/// A node within the UI Graph.
#[derive(Debug)]
enum Node {
    /// The root node and starting point for rendering.
    Root,
    /// A widget constructed by a user.
    Widget(Container),
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
        }
    }

    /// If there is a Widget for the given index, return a reference to it.
    pub fn get_widget<I: GraphIndex>(&self, idx: I) -> Option<&Container> {
        let Graph { ref index_map, ref graph, .. } = *self;
        index_map.to_node_index(idx).map(|idx| {
            match &graph[idx] {
                &Node::Widget(ref container) => container,
                _ => unreachable!(),
            }
        })
    }

    /// If there is a Widget for the given Id, return a mutable reference to it.
    pub fn get_widget_mut<I: GraphIndex>(&mut self, idx: I) -> Option<&mut Container> {
        let Graph { ref index_map, ref mut graph, .. } = *self;
        index_map.to_node_index(idx).map(move |idx| match &mut graph[idx] {
            &mut Node::Widget(ref mut container) => container,
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
        // We want to do a "parent first" check for the fastest search.
        let mut maybe_picked = None;

        loop {
            // The index of our current node is the current picked widget or the root node.
            let idx = maybe_picked.unwrap_or(self.root);

            // Iterator over each of the children widgets to the currently picked idx.
            let kids = self.graph.neighbors_directed(idx, pg::Outgoing);

            // Filter kids so that only those that are under the cursor remain.
            let mut picked_kids = kids.filter_map(|kid_idx| {
                let widget = match self.graph[kid_idx] {
                    Node::Widget(ref container) => container,
                    _ => unreachable!(),
                };
                match widget.kind {
                    "EMPTY" => None,
                    _ => {
                        let (w, h) = widget.element.get_size();
                        match ::utils::is_over_rect(widget.xy, xy, [w as f64, h as f64]) {
                            true => Some((kid_idx, widget.depth)),
                            false => None,
                        }
                    },
                }
            });

            // The top widget (with the lowest depth) is our picked widget.
            let maybe_picked_kid = picked_kids.next().map(|(kid_idx, depth)| {
                picked_kids.fold((kid_idx, depth), |(i, top_depth), (kid_idx, depth)| {
                    if depth > top_depth { (i, top_depth) } else { (kid_idx, depth) }
                })
            });

            // If we picked a new widget, set it, otherwise we're done.
            maybe_picked = match maybe_picked_kid {
                None => break,
                Some((picked_kid_idx, _)) => Some(picked_kid_idx),
            }
        }

        maybe_picked.map(|idx| I::from_idx(idx, &self.index_map).unwrap())
    }


    /// Set the parent for the given Widget Id.
    /// This method clears all other incoming edges and ensures that the widget only has a single
    /// parent (incoming edge). This means we can be sure of retaining a tree structure.
    pub fn set_parent_for_widget<I, P>(&mut self, idx: I, maybe_parent_idx: Option<P>) where
        I: GraphIndex,
        P: GraphIndex,
    {
        let Graph { ref mut graph, ref index_map, root, .. } = *self;
        let node_idx = index_map.to_node_index(idx).expect("No NodeIndex for given GraphIndex");
        // If no parent id was given, we will set the root as the parent.
        let parent_node_idx = match maybe_parent_idx {
            Some(idx) => match index_map.to_node_index(idx) {
                Some(idx) => idx,
                // If there is not yet a matching index in the graph for the given parent index, it
                // may be that the parent widget has not yet been added. For now, we'll just return
                // out of the function and not set any parent.
                //
                // FIXME: A better way to handle this might be to add a temporary node to the graph
                // at the given parent index so that we can add the edge even in the parent widget's
                // absense. The temporary node could be replaced by the proper widget later in the
                // cycle. If the temporary node hasn't been replaced by the time `Graph::draw` is
                // called, we could assume that there has been some error with the parent id for
                // this widget and notify the user accordingly.
                None => return,
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
                    panic!("Adding widget {:?} with the given parent {:?} caused a cycle \
                           within the Ui Graph.\n{:?}", idx, parent_idx, graph);
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
        } = cached;

        let stored: StoredWidget<W::State, W::Style> = StoredWidget { state: state, style: style };

        match self.contains(id) {

            // If there is no Widget for the given ID we need to add one.
            false => {
                let container = Container {
                    maybe_state: Some(Box::new(stored)),
                    kind: kind,
                    xy: xy,
                    dim: dim,
                    depth: depth,
                    drag_state: drag_state,
                    element: maybe_new_element.unwrap_or_else(|| ::elmesque::element::empty()),
                    has_updated: true,
                    kid_area: kid_area,
                };
                self.add_widget(container, Some(id), maybe_parent_idx);
            },

            // If we already have a Widget for the given ID, we need to update it.
            true => {
                self.set_parent_for_widget(id, maybe_parent_idx);
                let container = self.get_widget_mut(id).unwrap();

                // If the container already exists with the state of some other kind of widget, we
                // can assume there's been a mistake with the given Id.
                if container.kind != kind && container.kind != "EMPTY" {
                    panic!("A widget of a different kind already exists at the given UiId ({:?}).
                            You tried to insert a {:?}, however the existing widget is a {:?}.
                            Check your widgets' `UiId`s for errors.",
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
                if let Some(new_element) = maybe_new_element {
                    container.element = new_element;
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
        let Graph { ref mut graph, ref index_map, root, .. } = *self;
        draw_node(root,
                  maybe_captured_mouse.map(|id| index_map[id]),
                  maybe_captured_keyboard.map(|id| index_map[id]),
                  graph,
                  renderer);
    }

}


/// A recursive function for drawing a graph from a given node index.
///
/// The order in which nodes will be visited is akin to a depth-first search, where the branches
/// with the greatest `depth` are drawn first (unless the branch is on a captured widget, which
/// will always be drawn last).
fn draw_node<'a, C, G>(idx: NodeIndex,
                       maybe_captured_mouse: Option<NodeIndex>,
                       maybe_captured_keyboard: Option<NodeIndex>,
                       graph: &mut PetGraph,
                       renderer: &mut Renderer<'a, C, G>) where
    C: CharacterCache,
    G: Graphics<Texture = C::Texture>,
{

    // First, draw the widget at this node.
    if let &mut Node::Widget(ref mut container) = &mut graph[idx] {
        if container.has_updated {
            container.element.draw(renderer);
            container.has_updated = false;
        }
    }

    // Collect each of the child branches to sort them by depth.
    // TODO: We could remove these allocations if we were to store this sorting buffer in
    // `Container`.
    let mut kids: Vec<NodeIndex> = graph.neighbors_directed(idx, pg::Outgoing).collect();
    kids.sort_by(|&a, &b| {
        use std::cmp::Ordering;
        if Some(a) == maybe_captured_mouse || Some(a) == maybe_captured_keyboard {
            Ordering::Greater
        } else if let (&Node::Widget(ref a), &Node::Widget(ref b)) = (&graph[a], &graph[b]) {
            b.depth.partial_cmp(&a.depth).unwrap()
        } else {
            Ordering::Equal
        }
    });

    // Now draw each of the kid nodes.
    for kid_idx in kids.into_iter() {
        draw_node(kid_idx, maybe_captured_mouse, maybe_captured_keyboard, graph, renderer);
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

