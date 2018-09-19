//! A widget for viewing and controlling graph structures.

use {color, widget, Color, Colorable, Point, Positionable, Scalar, Widget, Ui, UiCell};
use std::any::{Any, TypeId};
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, Weak};
use utils::{iter_diff, IterDiff};

pub use self::node::{Node, SocketLayout, SocketSide};

pub mod node;

/// Traits required by types that may be used as a graph node identifier.
///
/// This trait has a blanket implementation for all types that satisfy the bounds.
pub trait NodeId: 'static + Copy + Clone + PartialEq + Eq + Hash + Send {}
impl<T> NodeId for T where T: 'static + Copy + Clone + PartialEq + Eq + Hash + Send {}

/// Stores the layout of all nodes within the graph.
///
/// All positions are relative to the centre of the `Graph` widget.
///
/// Nodes can be moved by 
#[derive(Clone, Debug, PartialEq)]
pub struct Layout<NI>
where
    NI: Eq + Hash,
{
    map: HashMap<NI, Point>,
}

impl<NI> Deref for Layout<NI>
where
    NI: NodeId,
{
    type Target = HashMap<NI, Point>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<NI> DerefMut for Layout<NI>
where
    NI: NodeId,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

/// A widget used for visualising and manipulating **Graph** types.
#[derive(Clone, Debug, WidgetCommon_)]
pub struct Graph<'a, N, E>
where
    N: Iterator,
    N::Item: NodeId,
    E: Iterator<Item=(NodeSocket<N::Item>, NodeSocket<N::Item>)>,
{
    /// Data necessary and common for all widget builder types.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// Unique styling for the **Graph**.
    pub style: Style,
    /// All nodes within the graph that the widget is to represent.
    pub nodes: N,
    /// All edges within the graph.
    pub edges: E,
    /// The position of each node within the graph.
    pub layout: &'a Layout<N::Item>,
}

/// Unique styling for the **BorderedRectangle** widget.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// Shape styling for the inner rectangle.
    #[conrod(default = "color::TRANSPARENT")]
    pub background_color: Option<Color>,
}

widget_ids! {
    struct Ids {
        // The rectangle over which all nodes are placed.
        background,
    }
}

/// Unique state for the `Graph`.
pub struct State<NI>
where
    NI: NodeId,
{
    ids: Ids,
    shared: Arc<Mutex<Shared<NI>>>,
}

// State shared between the **Graph**'s **State** and the returned **Session**.
struct Shared<NI>
where
    NI: NodeId,
{
    // A queue of events collected during `set` so that they may be emitted during
    // **SessionEvents**.
    events: VecDeque<Event<NI>>,
    // A mapping from node IDs to their data.
    nodes: HashMap<NI, NodeInner>,
    // A list of indices, one for each node in the graph.
    node_ids: Vec<NI>,
    // A list of all edges where (a, b) represents the directed edge a -> b.
    edges: Vec<(NodeSocket<NI>, NodeSocket<NI>)>,
    // A map from type identifiers to available `widget::Id`s for those types.
    widget_id_map: WidgetIdMap<NI>,
}

// A type for managing the input and output socket layouts.
#[derive(Copy, Clone, Debug)]
struct SocketLayouts {
    input: SocketLayout,
    output: SocketLayout,
}

// A list of `widget::Id`s for a specific type.
#[derive(Default)]
struct TypeWidgetIds {
    // The index of the next `widget::Id` to use for this type.
    next_index: usize,
    // The list of widget IDs.
    widget_ids: Vec<widget::Id>,
}

impl TypeWidgetIds {
    // Return the next `widget::Id` for a widget of the given type.
    //
    // If there are no more `Id`s available for the type, a new one will be generated from the
    // given `widget::id::Generator`.
    fn next_id(&mut self, generator: &mut widget::id::Generator) -> widget::Id {
        loop {
            match self.widget_ids.get(self.next_index).map(|&id| id) {
                None => self.widget_ids.push(generator.next()),
                Some(id) => {
                    self.next_index += 1;
                    break id;
                }
            }
        }
    }
}

// A mapping from types to their list of IDs.
#[derive(Default)]
struct WidgetIdMap<NI>
where
    NI: NodeId,
{
    // A map from types to their available `widget::Id`s
    type_widget_ids: HashMap<TypeId, TypeWidgetIds>,
    // A map from node IDs to their `widget::Id`.
    //
    // This is cleared at the end of each `Widget::update` and filled during the `Node`
    // instantiation phase.
    node_widget_ids: HashMap<NI, widget::Id>,
}

impl<NI> WidgetIdMap<NI>
where
    NI: NodeId,
{
    // Resets the index for every `TypeWidgetIds` list to `0`.
    //
    // This should be called at the beginning of the `Graph` update to ensure each widget
    // receives a unique ID. If this is not called, the graph will request more and more
    // `widget::Id`s every update and quickly bloat the `Ui`'s inner widget graph.
    fn reset_indices(&mut self) {
        for type_widget_ids in self.type_widget_ids.values_mut() {
            type_widget_ids.next_index = 0;
        }
    }

    // Clears the `node_id` -> `widget_id` mappings so that they may be recreated during the next
    // node instantiation stage.
    fn clear_node_mappings(&mut self) {
        self.node_widget_ids.clear();
    }

    // Return the next `widget::Id` for a widget of the given type.
    //
    // If there are no more `Id`s available for the type, a new one will be generated from the
    // given `widget::id::Generator`.
    fn next_id_for_node<T>(&mut self, node_id: NI, generator: &mut widget::id::Generator) -> widget::Id
    where
        T: Any,
    {
        let type_id = TypeId::of::<T>();
        let type_widget_ids = self.type_widget_ids.entry(type_id).or_insert_with(TypeWidgetIds::default);
        let widget_id = type_widget_ids.next_id(generator);
        self.node_widget_ids.insert(node_id, widget_id);
        widget_id
    }

    // Return the next `widget::Id` for a widget of the given type.
    //
    // If there are no more `Id`s available for the type, a new one will be generated from the
    // given `widget::id::Generator`.
    fn next_id_for_edge<T>(&mut self, generator: &mut widget::id::Generator) -> widget::Id
    where
        T: Any,
    {
        let type_id = TypeId::of::<T>();
        let type_widget_ids = self.type_widget_ids.entry(type_id).or_insert_with(TypeWidgetIds::default);
        let widget_id = type_widget_ids.next_id(generator);
        widget_id
    }
}

/// An interaction has caused some event to occur.
//
// TODO:
//
// - Hovered near outlet.
// - Edge end hovered near an outlet?
#[derive(Clone, Debug, PartialEq)]
pub enum Event<NI> {
    /// Events associated with nodes.
    Node(NodeEvent<NI>),
    /// Events associated with edges.
    Edge(EdgeEvent<NI>),
}

/// Represents a socket connection on a node.
///
/// Assumed to be either an input or output socket based on its usage within a tuple. E.g. given
/// two sockets *(a, b)*, socket *a*'s `socket_index` refers its index within `a`'s ***output***
/// socket list, while *b*'s refers its index within `b`'s ***input*** socket list.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeSocket<NI> {
    /// The unique identifier for the node.
    pub id: NI,
    /// The index of the socket on one side of the node.
    ///
    /// E.g. if the socket is the 3rd output socket, index would be `2`.
    pub socket_index: usize,
}

/// Events related to adding and removing nodes.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NodeEvent<NI> {
    /// The user attempted to remove the node with the given identifier.
    Remove(NI),
    /// The widget used to represent this `Node` has been dragged.
    Dragged {
        /// Unique identifier of the node being dragged.
        node_id: NI,
        /// The origin of the drag relative to the `Graph` widget position.
        from: Point,
        /// The end of the drag relative to the `Graph` widget position.
        to: Point,
    },
}

/// Events related to adding and removing edges.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum EdgeEvent<NI> {
    /// The user has pressed the given node socket with the left mouse button to begin creating an
    /// edge.
    AddStart(NodeSocket<NI>),
    /// The user has attempted to create an edge between the two given node sockets.
    Add {
        start: NodeSocket<NI>,
        end: NodeSocket<NI>,
    },
    /// The user has cancelled creating an edge from the given socket.
    Cancelled(NodeSocket<NI>),
    /// The user has attempted to remove the edge connecting the two given sockets.
    Remove {
        start: NodeSocket<NI>,
        end: NodeSocket<NI>,
    },
}

/// The camera used to view the graph.
///
/// The camera supports 2D positioning and zoom.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Camera {
    // The position of the camera over the floorplan.
    //
    // [0.0, 0.0] - the centre of the graph.
    point: Point,
    // The higher the zoom, the closer the floorplan appears.
    //
    // The zoom can be multiplied by a distance in metres to get the equivalent distance as a GUI
    // scalar value.
    //
    // 1.0 - Original resolution.
    // 0.5 - 50% view.
    zoom: Scalar,
}

/// A context for moving through the modes of graph widget instantiation in a type-safe manner.
///
/// The **Session** is shared between 3 stages:
///
/// 1. **SessionEvents**: Emit all graph events that have occurred since the last instantiation.
/// 2. **SessionNodes**: Instantiate all node widgets in the graph.
/// 3. **SessionEdges**: Instantiate all edge widgets in the graph.
///
/// NOTE: This should allow for different instantiation orders, e.g: nodes then edges, all
/// connected components in topo order, edges then nodes, etc.
pub struct Session<NI: NodeId> {
    // The unique identifier used to instantiate the graph widget.
    graph_id: widget::Id,
    // State shared with the `Graph` widget.
    shared: Weak<Mutex<Shared<NI>>>,
}

/// The first stage of the graph's **Session** event.
pub struct SessionEvents<NI: NodeId> {
    session: Session<NI>,
}

/// The second stage of the graph's **Session** event.
pub struct SessionNodes<NI: NodeId> {
    session: Session<NI>,
}

/// The third stage of the graph's **Session** event.
pub struct SessionEdges<NI: NodeId> {
    session: Session<NI>,
}

/// An iterator yielding all pending events.
pub struct Events<'a, NI: NodeId> {
    shared: Arc<Mutex<Shared<NI>>>,
    // Bind the lifetime to the `SessionEvents` so the user can't leak the `Shared` state.
    lifetime: PhantomData<&'a ()>,
}

/// An iterator-like type yielding a `NodeContext` for every node in the graph.
///
/// Each `NodeContext` can be used for instantiating a widget for each node in the graph.
pub struct Nodes<'a, NI: 'a + NodeId> {
    // Index into the `node_ids`, indicating which node we're up to.
    index: usize,
    shared: Arc<Mutex<Shared<NI>>>,
    // The `widget::Id` of the parent graph widget.
    graph_id: widget::Id,
    // Bind the lifetime to the `SessionNodes` so the user can't leak the `Shared` state.
    lifetime: PhantomData<&'a NI>,
}

// Node data stored within the 
#[derive(Copy, Clone)]
struct NodeInner {
    point: Point,
}

/// A context for a node yielded during the node instantiation stage.
///
/// This type can be used to:
///
/// 1. Get the position of the node via `point()`.
/// 2. Get the ID for this node via `node_id()`.
/// 3. Convert into a `NodeWidget` ready for instantiation within the `Ui` via `widget(a_widget)`.
pub struct NodeContext<'a, NI: 'a + NodeId> {
    node_id: NI,
    point: Point,
    // The `widget::Id` of the `NodeContext`'s parent `Graph` widget.
    graph_id: widget::Id,
    shared: Arc<Mutex<Shared<NI>>>,
    // Bind the lifetime to the `SessionNodes` so the user can't leak the `Shared` state.
    lifetime: PhantomData<&'a NI>,
}

/// Returned when a `NodeContext` is assigned a widget.
///
/// This intermediary type allows for accessing the `widget::Id` before the widget itself is
/// instantiated.
pub struct NodeWidget<'a, NI: 'a + NodeId, W> {
    node: NodeContext<'a, NI>,
    widget: W,
    // `None` if not yet requested the `WidgetIdMap`. `Some` if it has.
    widget_id: Cell<Option<widget::Id>>,
}

/// An iterator-like type yielding a `NodeContext` for every node in the graph.
///
/// Each `NodeContext` can be used for instantiating a widget for each node in the graph.
pub struct Edges<'a, NI: 'a + NodeId> {
    // The index into the `shared.edges` `Vec` that for the next `Edge` that is to be yielded.
    index: usize,
    shared: Arc<Mutex<Shared<NI>>>,
    // The `widget::Id` of the parent graph widget.
    graph_id: widget::Id,
    // Bind the lifetime to the `SessionEdges` so the user can't leak the `Shared` state.
    lifetime: PhantomData<&'a ()>,
}

/// A context for an edge yielded during the edge instantiation stage.
///
/// Tyis type can 
pub struct Edge<'a, NI: NodeId> {
    // The `widget::Id` of the `Edge`'s parent `Graph` widget.
    graph_id: widget::Id,
    // The data shared with the graph state, used to access the `WidgetIdMap`.
    shared: Arc<Mutex<Shared<NI>>>,
    // The start of the edge.
    start: NodeSocket<NI>,
    // The end of the edge.
    end: NodeSocket<NI>,
    // Bind the lifetime to the `SessionEdges` so the user can't leak the `Shared` state.
    lifetime: PhantomData<&'a ()>,
}

/// Returned when an `Edge` is assigned a widget.
///
/// This intermediary type allows for accessing the `widget::Id` before the widget itself is
/// instantiated.
pub struct EdgeWidget<'a, NI: 'a + NodeId, W> {
    edge: Edge<'a, NI>,
    widget: W,
    // `None` if not yet requested the `WidgetIdMap`. `Some` if it has.
    widget_id: Cell<Option<widget::Id>>,
}

// impl<NI> Layout<NI>
// where
//     NI: Eq + Hash,
// {
//     /// The position of the node at the given node identifier.
//     pub fn get(&self, node_id: NI) -> Option<&Point> {
//         self.map.get(&node_id)
//     }
//     /// The position of the node at the given node identifier.
//     pub fn get_mut(&mut self, node_id: NI) -> Option<&mut Point> {
//         self.map.get_mut(&node_id)
//     }
// }

impl<NI> From<HashMap<NI, Point>> for Layout<NI>
where
    NI: NodeId,
{
    fn from(map: HashMap<NI, Point>) -> Self {
        Layout { map }
    }
}

impl<NI> Into<HashMap<NI, Point>> for Layout<NI>
where
    NI: NodeId,
{
    fn into(self) -> HashMap<NI, Point> {
        let Layout { map } = self;
        map
    }
}

impl<NI> SessionEvents<NI>
where
    NI: NodeId,
{
    /// All events that have occurred since the last 
    pub fn events(&self) -> Events<NI> {
        let shared = self.session.shared.upgrade().expect("failed to access `Shared` state");
        Events { shared, lifetime: PhantomData }
    }

    /// Transition from the **SessionEvents** into **SessionNodes** for instantiating nodes.
    pub fn next(self) -> SessionNodes<NI> {
        let SessionEvents { session } = self;
        SessionNodes { session }
    }
}

impl<'a, NI> Iterator for Events<'a, NI>
where
    NI: NodeId,
{
    type Item = Event<NI>;
    fn next(&mut self) -> Option<Self::Item> {
        self.shared.lock()
            .ok()
            .and_then(|mut guard| guard.events.pop_front())
    }
}

impl<NI> SessionNodes<NI>
where
    NI: NodeId,
{
    /// Produce an iterator yielding a `NodeContext` for each node present in the graph.
    pub fn nodes(&mut self) -> Nodes<NI> {
        let graph_id = self.session.graph_id;
        let shared = self.session.shared.upgrade().expect("failed to access `Shared` state");
        Nodes { index: 0, shared, graph_id, lifetime: PhantomData }
    }

    /// Transition from the **SessionNodes** into **SessionEdges** for instantiating edges.
    pub fn next(self) -> SessionEdges<NI> {
        let SessionNodes { session } = self;
        SessionEdges { session }
    }
}

impl<'a, NI> Iterator for Nodes<'a, NI>
where
    NI: NodeId,
{
    type Item = NodeContext<'a, NI>;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;
        self.shared.lock()
            .ok()
            .and_then(|guard| {
                guard.node_ids
                    .get(index)
                    .and_then(|&id| guard.nodes.get(&id).map(|&inner| (id, inner)))
            })
            .map(|(node_id, NodeInner { point })| {
                NodeContext {
                    node_id,
                    point,
                    graph_id: self.graph_id,
                    shared: self.shared.clone(),
                    lifetime: PhantomData,
                }
            })
    }
}

impl<NI> SessionEdges<NI>
where
    NI: NodeId,
{
    /// Produce an iterator yielding an `Edge` for each node present in the graph.
    pub fn edges(&mut self) -> Edges<NI> {
        let graph_id = self.session.graph_id;
        let shared = self.session.shared.upgrade().expect("failed to access `Shared` state");
        Edges { index: 0, shared, graph_id, lifetime: PhantomData }
    }
}

impl<'a, NI> Iterator for Edges<'a, NI>
where
    NI: NodeId,
{
    type Item = Edge<'a, NI>;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;
        self.shared.lock()
            .ok()
            .and_then(|guard| {
                guard.edges.get(index).map(|&(start, end)| {
                    Edge {
                        graph_id: self.graph_id,
                        shared: self.shared.clone(),
                        start: start,
                        end: end,
                        lifetime: PhantomData,
                    }
                })
            })
    }
}

impl<'a, NI> NodeContext<'a, NI>
where
    NI: NodeId,
{
    /// The unique identifier associated with this node.
    pub fn node_id(&self) -> NI {
        self.node_id
    }

    /// The location of the node.
    pub fn point(&self) -> Point {
        self.point
    }

    /// Specify the widget to use 
    pub fn widget<W>(self, widget: W) -> NodeWidget<'a, NI, W> {
        NodeWidget {
            node: self,
            widget,
            widget_id: Cell::new(None),
        }
    }
}

impl<'a, NI, W> NodeWidget<'a, NI, W>
where
    NI: NodeId,
    W: 'static + Widget,
{
    /// Retrieve the `widget::Id` that will be used to instantiate this node's widget.
    pub fn widget_id(&self, ui: &mut UiCell) -> widget::Id {
        match self.widget_id.get() {
            Some(id) => id,
            None => {
                // Request a `widget::Id` from the `WidgetIdMap`.
                let mut shared = self.node.shared.lock().unwrap();
                let id = shared.widget_id_map
                    .next_id_for_node::<W>(self.node_id, &mut ui.widget_id_generator());
                self.widget_id.set(Some(id));
                id
            },
        }
    }

    /// Map over the inner widget.
    pub fn map<M>(self, map: M) -> Self
    where
        M: FnOnce(W) -> W,
    {
        let NodeWidget { node, mut widget, widget_id } = self;
        widget = map(widget);
        NodeWidget { node, widget, widget_id }
    }

    /// Set the given widget for the node at `node_id()`.
    pub fn set(self, ui: &mut UiCell) -> W::Event {
        let widget_id = self.widget_id(ui);
        let NodeWidget { node, widget, .. } = self;
        widget
            .xy_relative_to(node.graph_id, node.point)
            .parent(node.graph_id)
            .set(widget_id, ui)
    }
}

impl<'a, NI, W> Deref for NodeWidget<'a, NI, W>
where
    NI: NodeId,
{
    type Target = NodeContext<'a, NI>;
    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<'a, NI> Edge<'a, NI>
where
    NI: NodeId,
{
    /// The start (or "input") for the edge.
    ///
    /// This is described via the node's `Id` and the position of its output socket.
    pub fn start(&self) -> NodeSocket<NI> {
        self.start
    }

    /// The end (or "output") for the edge.
    ///
    /// This is described via the node's `Id` and the position of its input socket.
    pub fn end(&self) -> NodeSocket<NI> {
        self.end
    }

    /// The start and end sockets.
    pub fn sockets(&self) -> (NodeSocket<NI>, NodeSocket<NI>) {
        (self.start, self.end)
    }

    /// Specify the widget to use 
    pub fn widget<W>(self, widget: W) -> EdgeWidget<'a, NI, W> {
        EdgeWidget {
            edge: self,
            widget,
            widget_id: Cell::new(None),
        }
    }
}

/// Returns the `widget::Id` for a node if one exists.
///
/// Returns `None` if there is no `Graph` for the given `graph_id` or if there is not yet a
/// `widget::Id` for the given `node_id`.
///
/// This will always return `None` if called between calls to the `Graph::set` and node
/// instantiation stages, as `widget::Id`s for nodes are only populated during the node
/// instantiation stage.
pub fn node_widget_id<NI>(node_id: NI, graph_id: widget::Id, ui: &Ui) -> Option<widget::Id>
where
    NI: NodeId,
{
    ui.widget_graph()
        .widget(graph_id)
        .and_then(|container| container.state_and_style::<State<NI>, Style>())
        .and_then(|unique| {
            let shared = unique.state.shared.lock().unwrap();
            shared.widget_id_map.node_widget_ids.get(&node_id).map(|&id| id)
        })
}

/// Returns the `widget::Id`s for the start and end nodes.
///
/// `Edge`s can only exist for the lifetime of a `SessionEdges`, thus we assume that there will
/// always be a `Graph` for the edge's `graph_id` and that there will always be a `widget::Id`
/// for the start and end nodes.
///
/// **Panic!**s if the given `Ui` is not the same one used to create the edge's parent Graph
/// widget.
pub fn edge_node_widget_ids<NI>(edge: &Edge<NI>, ui: &Ui) -> (widget::Id, widget::Id)
where
    NI: NodeId,
{
    ui.widget_graph()
        .widget(edge.graph_id)
        .and_then(|container| container.state_and_style::<State<NI>, Style>())
        .map(|unique| {
            let shared = unique.state.shared.lock().unwrap();
            let a = shared.widget_id_map.node_widget_ids.get(&edge.start.id).map(|&id| id);
            let b = shared.widget_id_map.node_widget_ids.get(&edge.end.id).map(|&id| id);
            (a.expect("no `widget::Id` for start node"), b.expect("no `widget::Id` for end node"))
        })
        .expect("no graph associated with edge's `graph_id` was found")
}

impl<'a, NI, W> EdgeWidget<'a, NI, W>
where
    NI: NodeId,
    W: 'static + Widget,
{
    /// Retrieve the `widget::Id` that will be used to instantiate this edge's widget.
    pub fn widget_id(&self, ui: &mut UiCell) -> widget::Id {
        match self.widget_id.get() {
            Some(id) => id,
            None => {
                // Request a `widget::Id` from the `WidgetIdMap`.
                let mut shared = self.edge.shared.lock().unwrap();
                let id = shared.widget_id_map.next_id_for_edge::<W>(&mut ui.widget_id_generator());
                self.widget_id.set(Some(id));
                id
            },
        }
    }

    /// Apply the given function to the inner widget.
    pub fn map<M>(self, map: M) -> Self
    where
        M: FnOnce(W) -> W,
    {
        let EdgeWidget { edge, mut widget, widget_id } = self;
        widget = map(widget);
        EdgeWidget { edge, widget, widget_id }
    }

    /// Set the given widget for the edge.
    pub fn set(self, ui: &mut UiCell) -> W::Event {
        let widget_id = self.widget_id(ui);
        let EdgeWidget { edge, widget, .. } = self;
        widget
            .parent(edge.graph_id)
            .set(widget_id, ui)
    }
}

impl<'a, N, E> Graph<'a, N, E>
where
    N: Iterator,
    N::Item: NodeId,
    E: Iterator<Item=(NodeSocket<N::Item>, NodeSocket<N::Item>)>,
{
    /// Begin building a new **Graph** widget.
    pub fn new<NI, EI>(nodes: NI, edges: EI, layout: &'a Layout<NI::Item>) -> Self
    where
        NI: IntoIterator<IntoIter=N, Item=N::Item>,
        EI: IntoIterator<IntoIter=E, Item=(NodeSocket<N::Item>, NodeSocket<N::Item>)>,
    {
        Graph {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            nodes: nodes.into_iter(),
            edges: edges.into_iter(),
            layout: layout,
        }
    }

    /// Color the **Graph**'s rectangular area with the given color.
    pub fn background_color(mut self, color: Color) -> Self {
        self.style.background_color = Some(color);
        self
    }
}

impl<'a, N, E> Widget for Graph<'a, N, E>
where
    N: Iterator,
    N::Item: NodeId,
    E: Iterator<Item=(NodeSocket<N::Item>, NodeSocket<N::Item>)>,
{
    type State = State<N::Item>;
    type Style = Style;
    type Event = SessionEvents<N::Item>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        let events = VecDeque::new();
        let nodes = HashMap::new();
        let node_ids = Vec::new();
        let edges = Vec::new();
        let type_widget_ids = HashMap::new();
        let node_widget_ids = HashMap::new();
        let widget_id_map = WidgetIdMap { type_widget_ids, node_widget_ids };
        let shared = Shared { events, nodes, node_ids, edges, widget_id_map };
        State {
            ids: Ids::new(id_gen),
            shared: Arc::new(Mutex::new(shared)),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, style, rect, ui, .. } = args;
        let Graph { nodes, edges, layout, .. } = self;
        let mut shared = state.shared.lock().unwrap();

        // Reset the WidgetIdMap indices.
        shared.widget_id_map.reset_indices();

        // Compare the existing node indices with the new iterator.
        match iter_diff(&shared.node_ids, nodes) {
            Some(diff) => match diff {
                IterDiff::FirstMismatch(i, mismatch) => {
                    shared.node_ids.truncate(i);
                    shared.node_ids.extend(mismatch);
                },
                IterDiff::Longer(remaining) => {
                    shared.node_ids.extend(remaining);
                },
                IterDiff::Shorter(total) => {
                    shared.node_ids.truncate(total);
                },
            },
            None => (),
        }

        // Compare the existing edges with the new iterator.
        match iter_diff(&shared.edges, edges) {
            Some(diff) => match diff {
                IterDiff::FirstMismatch(i, mismatch) => {
                    shared.edges.truncate(i);
                    shared.edges.extend(mismatch);
                },
                IterDiff::Longer(remaining) => {
                    shared.edges.extend(remaining);
                },
                IterDiff::Shorter(total) => {
                    shared.edges.truncate(total);
                },
            },
            None => (),
        }

        // Use `shared.node_ids` and `shared.edges` to fill `shared.nodes`.
        shared.nodes.clear();
        for i in 0..shared.node_ids.len() {
            // Retrieve the node ID.
            let node_id = shared.node_ids[i];

            // Get the node position, falling back to 0.0, 0.0 if none was given.
            let point = layout.map.get(&node_id).map(|&p| p).unwrap_or([0.0; 2]);

            // Check to see if this widget has been dragged since the last update.
            let point = match shared.widget_id_map.node_widget_ids.get(&node_id).map(|&w| w) {
                None => point,
                Some(widget_id) => {
                    let (dragged_x, dragged_y) = ui.widget_input(widget_id)
                        .drags()
                        .left()
                        .fold((0.0, 0.0), |(x, y), d| (x + d.delta_xy[0], y + d.delta_xy[1]));

                    // If dragging would not move the widget, we're done.
                    if dragged_x == 0.0 && dragged_y == 0.0 {
                        point
                    } else {
                        let to = [point[0] + dragged_x, point[1] + dragged_y];
                        let node_event = NodeEvent::Dragged { node_id, from: point, to };
                        let event = Event::Node(node_event);
                        shared.events.push_back(event);
                        to
                    }
                },
            };

            let node = NodeInner { point };
            shared.nodes.insert(node_id, node);
        }

        let background_color = style.background_color(&ui.theme);
        widget::Rectangle::fill(rect.dim())
            .xy(rect.xy())
            .color(background_color)
            .parent(id)
            .graphics_for(id)
            .set(state.ids.background, ui);

        // Clear the old node->widget mappings ready for node instantiation.
        shared.widget_id_map.clear_node_mappings();

        let graph_id = id;
        let shared = Arc::downgrade(&state.shared);
        let session = Session { graph_id, shared };
        SessionEvents { session }
    }
}
