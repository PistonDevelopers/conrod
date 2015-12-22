//! This module was created in order to keep the `graph` module clean and focused upon the
//! **Graph** data structure behaviour.
//!
//! This module hosts more complex algorithms in which the **Graph** is a key component in
//! producing the desired result.


use daggy::Walker;
use position::{Point, Rect};
use super::depth_order::Visitable;
use super::{
    Graph,
    GraphIndex,
    NodeIndex,
};
use widget;


/// The rectangle that represents the maximum visible area for the widget with the given index.
///
/// Specifically, this considers the cropped scroll area for all parents.
///
/// Otherwise, return None if the widget is hidden.
pub fn visible_area_of_widget<I: GraphIndex>(graph: &Graph, idx: I) -> Option<Rect> {
    visible_area_of_widget_maybe_within_depth(graph, idx, None)
}


/// The rectangle that represents the maximum visible area for the widget with the given index.
///
/// This specifically considers the cropped scroll area for all parents until (and not including)
/// the deepest_parent_idx is reached.
///
/// Otherwise, return None if the widget is hidden.
pub fn visible_area_of_widget_within_depth<I>(graph: &Graph,
                                              idx: I,
                                              deepest_parent_idx: NodeIndex) -> Option<Rect>
    where I: GraphIndex,
{
    visible_area_of_widget_maybe_within_depth(graph, idx, Some(deepest_parent_idx))
}


/// Logic shared between the `visible_area_of_widget` and `visible_area_of_widget_within_depth`
/// functions.
fn visible_area_of_widget_maybe_within_depth<I>(graph: &Graph,
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
                if let Some(depth_parent_widget) = graph.widget(depth_parent) {
                    if depth_parent_widget.maybe_scrolling.is_some() {

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


/// If the given Point is currently on a Widget, return an index to that widget.
///
/// If the picked widget has a **Graphic** parent, that parent's index will be returned instead.
///
/// This function assumes that the given `depth_order` is up-to-date for the given `graph`.
pub fn pick_widget(graph: &Graph, depth_order: &[Visitable], xy: Point) -> Option<widget::Index> {
    depth_order.iter().rev()
        .find(|&&visitable| {
            match visitable {
                Visitable::Widget(idx) => {
                    if let Some(visible_rect) = visible_area_of_widget(graph, idx) {
                        if visible_rect.is_over(xy) {
                            return true
                        }
                    }
                },
                Visitable::Scrollbar(idx) => {
                    if let Some(scrolling) = graph.widget_scroll_state(idx) {
                        if scrolling.is_over(xy) {
                            return true;
                        }
                    }
                },
            }
            false
        })
        // Extract the node index from the `Visitable`.
        .map(|&visitable| match visitable {
            // Ensure that if we've picked some widget that is a **Graphic** child of some other
            // widget, we return the **Graphic** parent.
            Visitable::Widget(idx) => graph.graphic_parent_recursion(idx)
                .last_node(graph)
                .unwrap_or(idx),
            Visitable::Scrollbar(idx) => idx,
        })
        .and_then(|idx| graph.widget_index(idx))
}


/// If the given **Point** is currently over a scrollable widget, return an index to that widget.
///
/// This function assumes that the given `depth_order` is up-to-date for the given `graph`.
pub fn pick_scrollable_widget(graph: &Graph, depth_order: &[Visitable], xy: Point)
    -> Option<widget::Index>
{
    depth_order.iter().rev()
        .filter_map(|&visitable| match visitable {
            Visitable::Widget(idx) => Some(idx),
            Visitable::Scrollbar(_) => None,
        })
        .find(|&idx| {
            if let Some(ref container) = graph.widget(idx) {
                if container.maybe_scrolling.is_some() {
                    if container.rect.is_over(xy) {
                        return true;
                    }
                }
            }
            false
        })
        .and_then(|idx| graph.widget_index(idx))
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
pub fn bounding_box<I: GraphIndex>(graph: &Graph,
                               include_self: bool,
                               maybe_target_xy: Option<Point>,
                               use_kid_area: bool,
                               idx: I,
                               maybe_deepest_parent_idx: Option<NodeIndex>) -> Option<Rect>
{
    graph.node_index(idx).and_then(|idx| {
        graph.widget(idx).and_then(|container| {

            // If we're to use the kid area, we'll get the rect from that, otherwise we'll use
            // the regular dim and xy.
            //
            // If we're given some deepest parent index, we must only use the visible area within
            // the depth range that approaches it.
            let rect = if let Some(deepest_parent_idx) = maybe_deepest_parent_idx {
                match visible_area_of_widget_within_depth(graph, idx, deepest_parent_idx) {
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

            // Get the deepest parent so that we can consider it in the calculation of the kids
            // bounds.
            let deepest_parent_idx = maybe_deepest_parent_idx.or(Some(idx));

            // An iterator yielding the bounding_box returned by each of our children.
            let mut kids_bounds = graph.depth_children(idx)
                .filter(|g, _, n| !g.graphic_parent::<_, NodeIndex>(n).is_some())
                .iter(graph).nodes()
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
    })
}



/// Calculate the total scroll offset for the widget with the given index.
pub fn scroll_offset<I: GraphIndex>(graph: &Graph, idx: I) -> Point {
    let mut offset = [0.0, 0.0];
    let mut idx = match graph.node_index(idx) {
        Some(idx) => idx,
        // If the ID is not yet present within the graph, just return the zeroed offset.
        None => return offset,
    };

    // Check the widget at the given index for any scroll offset and add it to our total offset.
    let add_to_offset = |offset: &mut Point, idx: NodeIndex| {
        if let Some(scrolling) = graph.widget_scroll_state(idx) {
            let scroll_offset = scrolling.kids_pos_offset();
            offset[0] += scroll_offset[0].round();
            offset[1] += scroll_offset[1].round();
        }
    };

    // We need to calculate the offset by summing the offset of each scrollable parent, while
    // considering relative positioning and graphic parents.
    let mut depth_parents = graph.depth_parent_recursion(idx);
    while let Some(depth_parent) = depth_parents.next_node(graph) {

        // If the widget has some parent graphic edge, we may need to skip its offset.
        // This is because **Graphic** edges describe that the child widget is a graphical
        // element of the parent widget, so despite being instantiated at a higher depth, we
        // should consider its offset equal to the graphic parent.
        if let Some(graphic_parent) = graph.graphic_parent(idx) {
            match graph.position_parent::<_, NodeIndex>(idx) {

                // If we have a graphic parent but no position parent, we should skip straight
                // to the graphic parent.
                None => {
                    idx = graphic_parent;
                    depth_parents = graph.depth_parent_recursion(idx);
                    continue;
                },

                // Otherwise, we need to consider the given position parent.
                Some(position_parent) => {

                    // If our position parent is equal to or some child of our graphic parent,
                    // we don't need any offset.
                    // TODO: Review this part of the algorithm.
                    if position_parent == graphic_parent
                    || graph.does_recursive_depth_edge_exist(graphic_parent, position_parent) {
                        return offset;
                    }
                },

            }
        }

        // Recursively check all nodes with incoming `Position` edges for a parent that
        // matches our own parent. If any match, then we don't need to calculate any
        // additional offset as the widget we are being positioned relatively to has
        // already applied the necessary scroll offset.
        let mut position_parents = graph.position_parent_recursion(idx);
        while let Some(position_parent) = position_parents.next_node(graph) {

            // If our parent depth edge is also the parent position edge, we'll add
            // offset for that parent before finishing.
            if depth_parent == position_parent {
                add_to_offset(&mut offset, depth_parent);
                return offset;
            }

            // If our position_parent has some depth parent or grandparent that matches our
            // current depth_parent_idx, we can assume that the scroll offset for our widget
            // has already been calculated by this position_parent.
            if graph.does_recursive_depth_edge_exist(depth_parent, position_parent) {
                return offset;
            }
        }

        add_to_offset(&mut offset, depth_parent);

        // Set the parent as the new current idx and continue traversing.
        idx = depth_parent;
    }

    offset
}

