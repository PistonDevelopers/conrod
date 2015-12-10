//! Types and functionality related to the calculation of a **Graph**'s rendering depth order.

use daggy::Walker;
use std::collections::HashSet;
use super::{
    Graph,
    GraphIndex,
    Node,
    NodeIndex,
};


/// Contains Node indices in order of depth, starting with the deepest.
pub struct DepthOrder {
    /// The primary **Vec** storing the **DepthOrder**'s ordered indices.
    pub indices: Vec<Visitable>,
    /// Used for storing indices of "floating" widgets during depth sorting so that they may be
    /// visited after widgets of the root tree.
    floating: Vec<NodeIndex>,
}


/// Parts of the graph that are significant when visiting and sorting by depth.
///
/// The reason a widget and its scrollbar are separate here is because a widget's scrollbar may
/// sometimes appear on *top* of the widget's children.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Visitable {
    /// The index of some widget in the graph.
    Widget(NodeIndex),
    /// The scrollbar for the widget at the given NodeIndex.
    Scrollbar(NodeIndex),
}


impl DepthOrder {

    /// Construct a new empty **DepthOrder**.
    pub fn new() -> DepthOrder {
        DepthOrder {
            indices: Vec::new(),
            floating: Vec::new(),
        }
    }

    /// Construct a new empty **DepthOrder**.
    ///
    /// There can be at most two indices per widget (the widget and the widget's scrollbar). Thus
    /// we'll reserve double the number of nodes given.
    pub fn with_node_capacity(n_nodes: usize) -> DepthOrder {
        let n_indices = n_nodes * 2;
        DepthOrder {
            indices: Vec::with_capacity(n_indices),
            floating: Vec::with_capacity(n_nodes),
        }
    }

    /// Update the **DepthOrder** (starting with the deepest) for all nodes in the given **Graph**.
    ///
    /// FIXME:
    /// This likely needs to be re-written, and will probably fail for graphs with many floating
    /// widgets instantiated upon other floating widgets.
    ///
    /// The proper algorithm should be a full toposort where the neighbours of each node are
    /// visited in the order specified within `visit_by_depth`.
    ///
    /// The `visit_by_depth` algorithm should not be recursive and instead use either looping,
    /// walking or iteration.
    pub fn update<M, K>(&mut self,
                        graph: &Graph,
                        root: NodeIndex,
                        updated_widgets: &HashSet<NodeIndex>,
                        maybe_captured_mouse: Option<M>,
                        maybe_captured_keyboard: Option<K>)
        where M: GraphIndex,
              K: GraphIndex,
    {
        let DepthOrder { ref mut indices, ref mut floating } = *self;

        let maybe_captured_mouse = maybe_captured_mouse.and_then(|idx| graph.node_index(idx));
        let maybe_captured_keyboard = maybe_captured_keyboard.and_then(|idx| graph.node_index(idx));

        // Clear the buffers and ensure they've enough memory allocated.
        let num_nodes = graph.node_count();
        indices.clear();
        indices.reserve(num_nodes);
        floating.clear();
        floating.reserve(num_nodes);

        // Visit each node in order of depth and add their indices to depth_order.
        // If the widget is floating, then store it in the floating deque instead.
        visit_by_depth(graph,
                       root,
                       updated_widgets,
                       maybe_captured_mouse,
                       maybe_captured_keyboard,
                       indices,
                       floating);

        // Sort the floating widgets so that the ones clicked last come last.
        floating.sort_by(|&a, &b| match (&graph[a], &graph[b]) {
            (&Node::Widget(ref a), &Node::Widget(ref b)) => {
                let a_floating = a.maybe_floating.expect("Not floating");
                let b_floating = b.maybe_floating.expect("Not floating");
                a_floating.time_last_clicked.cmp(&b_floating.time_last_clicked)
            },
            _ => ::std::cmp::Ordering::Equal,
        });

        // Visit all of the floating widgets last.
        while !floating.is_empty() {
            let idx = floating.remove(0);
            visit_by_depth(graph,
                           idx,
                           updated_widgets,
                           maybe_captured_mouse,
                           maybe_captured_keyboard,
                           indices,
                           floating);
        }
    }

}


/// Recursive function for visiting all nodes within the dag.
fn visit_by_depth(graph: &Graph,
                  idx: NodeIndex,
                  updated_widgets: &HashSet<NodeIndex>,
                  maybe_captured_mouse: Option<NodeIndex>,
                  maybe_captured_keyboard: Option<NodeIndex>,
                  depth_order: &mut Vec<Visitable>,
                  floating_deque: &mut Vec<NodeIndex>)
{
    // First, if the current node is a widget and it was set in the current `set_widgets` stage,
    // store its index.
    match graph.widget(idx).is_some() && updated_widgets.contains(&idx) {
        true => depth_order.push(Visitable::Widget(idx)),
        // If the current node is not an updated widget, we're done with this branch.
        false => return,
    }

    // Sort the children of the current node by their `.depth` members.
    // FIXME: We should remove these allocations by storing a `child_sorter` buffer in each Widget
    // node (perhaps in the `Container`).
    let mut child_sorter: Vec<NodeIndex> = graph.depth_children(idx).iter(&graph).nodes().collect();

    // Walking neighbors of a node in our graph returns then in the reverse order in which they
    // were added. Reversing here will give more predictable render order behaviour i.e. widgets
    // instantiated after other widgets will also be rendered after them by default.
    child_sorter.reverse();
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
        let maybe_is_floating = graph.widget(child_idx).map(|w| w.maybe_floating.is_some());

        // Store floating widgets int he floating_deque for visiting after the current tree.
        match maybe_is_floating {
            Some(true) => floating_deque.push(child_idx),
            _          => visit_by_depth(graph,
                                         child_idx,
                                         updated_widgets,
                                         maybe_captured_mouse,
                                         maybe_captured_keyboard,
                                         depth_order,
                                         floating_deque),
        }
    }

    // If the widget is scrollable, we should add its scrollbar to the visit order also.
    if graph.widget_scroll_state(idx).is_some() {
        depth_order.push(Visitable::Scrollbar(idx));
    }
}


