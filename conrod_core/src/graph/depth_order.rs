//! Types and functionality related to the calculation of a **Graph**'s rendering depth order.

use daggy::Walker;
use std;
use fnv;
use super::{Graph, Node};
use widget;


/// Contains Node indices in order of depth, starting with the deepest.
#[derive(Debug)]
pub struct DepthOrder {
    /// The primary **Vec** storing the **DepthOrder**'s ordered indices.
    pub indices: Vec<widget::Id>,
    /// Used for storing indices of "floating" widgets during depth sorting so that they may be
    /// visited after widgets of the root tree.
    floating: Vec<widget::Id>,
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
    pub fn update(&mut self,
                  graph: &Graph,
                  root: widget::Id,
                  updated_widgets: &fnv::FnvHashSet<widget::Id>)
    {
        let DepthOrder { ref mut indices, ref mut floating } = *self;

        // Clear the buffers and ensure they've enough memory allocated.
        let num_nodes = graph.node_count();
        indices.clear();
        indices.reserve(num_nodes);
        floating.clear();
        floating.reserve(num_nodes);

        // Visit each node in order of depth and add their indices to depth_order.
        // If the widget is floating, then store it in the floating deque instead.
        visit_by_depth(graph, root, updated_widgets, indices, floating);

        // Sort the floating widgets so that the ones clicked last come last.
        floating.sort_by(|&a, &b| match (&graph[a], &graph[b]) {
            (&Node::Widget(ref a), &Node::Widget(ref b)) => {
                let a_floating = a.maybe_floating.expect("Not floating");
                let b_floating = b.maybe_floating.expect("Not floating");
                a_floating.time_last_clicked.cmp(&b_floating.time_last_clicked)
            },
            _ => std::cmp::Ordering::Equal,
        });

        // Visit all of the floating widgets last.
        while !floating.is_empty() {
            let idx = floating.remove(0);
            visit_by_depth(graph, idx, updated_widgets, indices, floating);
        }
    }

}


/// Recursive function for visiting all nodes within the dag.
fn visit_by_depth(graph: &Graph,
                  idx: widget::Id,
                  updated_widgets: &fnv::FnvHashSet<widget::Id>,
                  depth_order: &mut Vec<widget::Id>,
                  floating_deque: &mut Vec<widget::Id>)
{
    // First, if the current node is a widget and it was set in the current `set_widgets` stage,
    // store its index.
    match graph.widget(idx).is_some() && updated_widgets.contains(&idx) {
        true => depth_order.push(idx),
        // If the current node is not an updated widget, we're done with this branch.
        false => return,
    }

    // Sort the children of the current node by their `.depth` members.
    // FIXME: We should remove these allocations by storing a `child_sorter` buffer in each Widget
    // node (perhaps in the `Container`).
    let mut child_sorter: Vec<widget::Id> = graph.depth_children(idx).iter(&graph).nodes().collect();

    child_sorter.sort_by(|&a, &b| {
        use std::cmp::Ordering;

        if let (&Node::Widget(ref a), &Node::Widget(ref b)) = (&graph[a], &graph[b]) {
            match b.depth.partial_cmp(&a.depth).expect("Depth was NaN!") {
                Ordering::Equal => a.instantiation_order_idx.cmp(&b.instantiation_order_idx),
                ordering => ordering,
            }
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
            _ => visit_by_depth(graph, child_idx, updated_widgets, depth_order, floating_deque),
        }
    }
}


