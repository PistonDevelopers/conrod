//! This module was created in order to keep the `graph` module clean and focused upon the
//! **Graph** data structure behaviour.
//!
//! This module hosts more complex algorithms in which the **Graph** is a key component in
//! producing the desired result.


use daggy::Walker;
use position::{Point, Rect};
use std;
use super::depth_order::Visitable;
use super::{EdgeIndex, Graph, GraphIndex, NodeIndex};
use widget;


// /// A node "walker" that yields all widgets under the given `xy` position in order from top to
// /// bottom.
// #[derive(Clone)]
// pub struct PickWidgets<'a> {
//     xy: Point,
//     rev_depth_order: std::iter::Rev<std::slice::Iter<'a, Visitable>>,
// }

// /// A node "walker" that yields all scrollable widgets under the given `xy` position in order from
// /// top to bottom.
// #[derive(Clone)]
// pub struct PickScrollableWidgets<'a> {
//     pick_widgets: PickWidgets<'a>,
// }

/// A node "walker" that yields all widgets under the given `xy` position in order from top to
/// bottom.
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct PickWidgets {
    xy: Point,
    idx: usize,
}

/// A node "walker" that yields all scrollable widgets under the given `xy` position in order from
/// top to bottom.
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct PickScrollableWidgets {
    pick_widgets: PickWidgets,
}


impl PickWidgets {

    /// The next `Visitable` under the `xy` location.
    ///
    /// A `Visitable` is either a `Widget` or a widget's `Scrollbar`.
    ///
    /// This is called within `PickWidgets::next`.
    pub fn next_visitable(&mut self,
                          graph: &Graph,
                          depth_order: &[Visitable]) -> Option<Visitable>
    {
        while self.idx > 0 {
            self.idx -= 1;
            match depth_order.get(self.idx) {
                None => break,
                Some(&visitable) => match visitable {
                    Visitable::Widget(idx) => {
                        if let Some(visible_rect) = cropped_area_of_widget(graph, idx) {
                            if visible_rect.is_over(self.xy) {
                                return Some(visitable);
                            }
                        }
                    },
                    Visitable::Scrollbar(_) => (),
                },
            }
        }
        None
    }

    /// The `widget::Index` of the next `Widget` under the `xy` location.
    ///
    /// The `Graph` is traversed from the top down.
    pub fn next(&mut self, graph: &Graph, depth_order: &[Visitable]) -> Option<widget::Index> {
        self.next_visitable(graph, depth_order)
            .map(|visitable| match visitable {
                // Ensure that if we've picked some widget that is a **Graphic** child of some
                // other widget, we return the **Graphic** parent.
                Visitable::Widget(idx) => graph.graphic_parent_recursion(idx)
                    .last_node(graph)
                    .unwrap_or(idx),
                Visitable::Scrollbar(idx) => idx,
            })
            .and_then(|idx| graph.widget_index(idx))
    }

}

impl PickScrollableWidgets {

    /// The `widget::Index` of the next scrollable `Widget` under the `xy` location.
    ///
    /// The `Graph` is traversed from the top down.
    pub fn next(&mut self, graph: &Graph, depth_order: &[Visitable]) -> Option<widget::Index> {
        while let Some(visitable) = self.pick_widgets.next_visitable(graph, depth_order) {
            match visitable {
                Visitable::Widget(idx) => {
                    if let Some(ref container) = graph.widget(idx) {
                        if container.maybe_x_scroll_state.is_some()
                        || container.maybe_y_scroll_state.is_some() {
                            if let Some(idx) = graph.widget_index(idx) {
                                return Some(idx);
                            }
                        }
                    }
                },
                Visitable::Scrollbar(_) => (),
            }
        }

        None
    }

}


/// Produces a graph node "walker" that yields all widgets under the given `xy` position in order
/// from top to bottom.
pub fn pick_widgets(depth_order: &[Visitable], xy: Point) -> PickWidgets {
    PickWidgets {
        xy: xy,
        idx: depth_order.len(),
    }
}

/// Produces a graph node "walker" that yields all scrollable widgets under the given `xy` position
/// in order from top to bottom.
pub fn pick_scrollable_widgets(depth_order: &[Visitable], xy: Point) -> PickScrollableWidgets {
    PickScrollableWidgets {
        pick_widgets: pick_widgets(depth_order, xy),
    }
}


/// The rectangle that represents the maximum visible area for the widget with the given index.
///
/// Specifically, this considers the cropped scroll area for all parents.
///
/// Otherwise, return None if the widget is hidden.
pub fn cropped_area_of_widget<I: GraphIndex>(graph: &Graph, idx: I) -> Option<Rect> {
    cropped_area_of_widget_maybe_within_depth(graph, idx, None)
}


/// The rectangle that represents the maximum visible area for the widget with the given index.
///
/// This specifically considers the cropped scroll area for all parents until (and not including)
/// the deepest_parent_idx is reached.
///
/// Otherwise, return None if the widget is hidden.
pub fn cropped_area_of_widget_within_depth<I>(graph: &Graph,
                                              idx: I,
                                              deepest_parent_idx: NodeIndex) -> Option<Rect>
    where I: GraphIndex,
{
    cropped_area_of_widget_maybe_within_depth(graph, idx, Some(deepest_parent_idx))
}


/// Logic shared between the `cropped_area_of_widget` and `cropped_area_of_widget_within_depth`
/// functions.
fn cropped_area_of_widget_maybe_within_depth<I>(graph: &Graph,
                                                idx: I,
                                                deepest_idx: Option<NodeIndex>) -> Option<Rect>
    where I: GraphIndex,
{
    graph.node_index(idx).and_then(|mut idx| {
        graph.widget(idx).and_then(|widget| {
            let mut overlapping_rect = widget.rect;
            let mut depth_parents = graph.depth_parent_recursion(idx);
            while let Some(depth_parent) = depth_parents.next_node(graph) {

                // If the parent's index matches that of the deepest, we're done.
                if Some(depth_parent) == deepest_idx {
                    break;
                }

                // Check to see if our parent is a scrollable widget and whether or not we need to
                // update the overlap.
                //
                // TODO: Consider "cropped area" here once implemented instead of scrolling.
                if let Some(depth_parent_widget) = graph.widget(depth_parent) {
                    if depth_parent_widget.maybe_x_scroll_state.is_some()
                    || depth_parent_widget.maybe_y_scroll_state.is_some() {

                        // If the depth_parent is also a **Graphic** parent, there is no need to
                        // calculate overlap as the child is a graphical element of the parent and
                        // thus is not cropped to it.
                        if !graph.does_graphic_edge_exist(depth_parent, idx) {
                            match overlapping_rect.overlap(depth_parent_widget.kid_area.rect) {
                                Some(overlap) => overlapping_rect = overlap,
                                None => return None,
                            }
                        }
                    }
                }

                // Set the current parent as the new child.
                idx = depth_parent;
            }

            Some(overlapping_rect)
        })
    })
}


/// Find the absolute `Rect` that bounds all widgets that are `Depth` children of the widget at the
/// given `idx`.
///
/// FIXME: This currently uses call stack recursion to do a depth-first search through all
/// depth_children for the total bounding box. This should use a proper `Dfs` type with it's own
/// stack for safer traversal that won't blow the stack on hugely deep GUIs.
pub fn kids_bounding_box<I>(graph: &Graph,
                            prev_updated: &std::collections::HashSet<NodeIndex>,
                            idx: I) -> Option<Rect>
    where I: GraphIndex,
{
    // When traversing the `depth_kids`, we only want to visit those who:
    // - are not also graphic kid widgets.
    // - are currently active within the `Ui`. In other words, they *were* updated during the last
    // call to `Ui::set_widgets`.
    let kid_filter = &|g: &Graph, _e, n| -> bool {
        let is_not_graphic_kid = !g.graphic_parent::<_, NodeIndex>(n).is_some();
        let is_set = prev_updated.contains(&n);
        is_not_graphic_kid && is_set
    };

    // A function for doing a recursive depth-first search through all depth kids that satisfy the
    // `kid_filter` (see above) in order to find the maximum "bounding box".
    fn kids_dfs<F>(graph: &Graph,
                   idx: NodeIndex,
                   deepest_parent_idx: NodeIndex,
                   kid_filter: &F) -> Option<Rect>
        where F: Fn(&Graph, EdgeIndex, NodeIndex) -> bool,
    {
        // If we're given some deepest parent index, we must only use the visible area within
        // the depth range that approaches it.
        cropped_area_of_widget_within_depth(graph, idx, deepest_parent_idx).map(|rect| {

            // An iterator yielding the bounding_box returned by each of our children.
            let kids_bounds = graph.depth_children(idx).filter(kid_filter).iter(graph).nodes()
                .filter_map(|n| kids_dfs(graph, n, deepest_parent_idx, kid_filter));

            kids_bounds.fold(rect, |max, next| max.max(next))
        })
    }

    graph.node_index(idx).and_then(|idx| {
        graph.widget(idx).and_then(|_| {

            // An iterator yielding the bounding_box returned by each of our children.
            let mut kids_bounds = graph.depth_children(idx).filter(kid_filter).iter(graph).nodes()
                .filter_map(|n| kids_dfs(graph, n, idx, kid_filter));

            kids_bounds.next().map(|first| {
                kids_bounds.fold(first, |max, next| max.max(next))
            })
        })
    })
}


/// Return the `scroll_offset` for the widget at the given index.
///
/// The offset is retrieved from the widget that is the immediate `depth_parent` of the widget at
/// the given `idx` unless:
///
/// - the immediate `depth_parent` of `idx` is also a `graphic_parent` to `idx`. In this case,
/// `NO_OFFSET` will be returned, as child widgets that are graphical elements of their parents
/// should not be affected by scrolling.
/// - one of the position parents also has the same `depth_parent`. In this case, `NO_OFFSET` will
/// be returned, as we know that our scroll offset has already been applied via the widget to which
/// we are relatively positioned.
pub fn scroll_offset<I: GraphIndex>(graph: &Graph, idx: I) -> Point {
    const NO_OFFSET: Point = [0.0, 0.0];

    if let Some(idx) = graph.node_index(idx) {
        if let Some(depth_parent) = graph.depth_parent(idx) {
            if let Some(depth_parent_widget) = graph.widget(depth_parent) {
                if depth_parent_widget.maybe_x_scroll_state.is_some()
                || depth_parent_widget.maybe_y_scroll_state.is_some() {

                    // If our depth parent is also a graphic parent, we don't want any offset.
                    if graph.graphic_parent_recursion(idx)
                        .any(graph, |_g, _e, n| n == depth_parent)
                    {
                        return NO_OFFSET;
                    }

                    // If we have a position_parent along the axis whose depth_parent is the same
                    // as ours, then the offset has already been applied via our relative
                    // positioning.
                    let is_already_offset = |mut position_parents: super::RecursiveWalk<_>| {
                        while let Some(position_parent) = position_parents.next_node(graph) {
                            if graph.depth_parent_recursion(position_parent)
                                .any(graph, |_g, _e, n| n == depth_parent)
                            {
                                return true;
                            }
                        }
                        false
                    };

                    let x_offset = depth_parent_widget.maybe_x_scroll_state.map(|scroll| {
                        let position_parents = graph.x_position_parent_recursion(idx);
                        if is_already_offset(position_parents) { 0.0 } else { scroll.offset }
                    }).unwrap_or(0.0);

                    let y_offset = depth_parent_widget.maybe_y_scroll_state.map(|scroll| {
                        let position_parents = graph.y_position_parent_recursion(idx);
                        if is_already_offset(position_parents) { 0.0 } else { scroll.offset }
                    }).unwrap_or(0.0);

                    return [x_offset, y_offset];
                }
            }
        }
    }

    NO_OFFSET
}
