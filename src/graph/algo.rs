//! This module was created in order to keep the `graph` module clean and focused upon the
//! **Graph** data structure behaviour.
//!
//! This module hosts more complex algorithms in which the **Graph** is a key component in
//! producing the desired result.


use daggy::Walker;
use position::{Point, Rect};
use std::collections::HashSet;
use super::depth_order::Visitable;
use super::{EdgeIndex, Graph, GraphIndex, NodeIndex};
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
                    if let Some(widget) = graph.widget(idx) {
                        if let Some(x_scroll_state) = widget.maybe_x_scroll_state {
                            if x_scroll_state.is_over(xy, widget.kid_area.rect) {
                                return true;
                            }
                        }
                        if let Some(y_scroll_state) = widget.maybe_y_scroll_state {
                            if y_scroll_state.is_over(xy, widget.kid_area.rect) {
                                return true;
                            }
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
                if container.maybe_x_scroll_state.is_some()
                || container.maybe_y_scroll_state.is_some() {
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
///
/// FIXME: This currently uses call stack recursion to do a depth-first search through all
/// depth_children for the total bounding box. This should use a proper `Dfs` type with it's own
/// stack for safer traversal that won't blow the stack on hugely deep GUIs.
pub fn bounding_box<I>(graph: &Graph,
                       prev_updated: &HashSet<NodeIndex>,
                       include_self: bool,
                       maybe_target_xy: Option<Point>,
                       use_kid_area: bool,
                       idx: I,
                       maybe_deepest_parent_idx: Option<NodeIndex>) -> Option<Rect>
    where I: GraphIndex,
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
                .filter(|g, _, n| {
                    let is_not_graphic_kid = !g.graphic_parent::<_, NodeIndex>(n).is_some();
                    let is_set = prev_updated.contains(&n);
                    is_not_graphic_kid && is_set
                })
                .iter(graph).nodes()
                .filter_map(|n| {
                    let include_self = true;
                    let use_kid_area = false;
                    bounding_box(graph,
                                 prev_updated,
                                 include_self,
                                 Some(target_xy),
                                 use_kid_area,
                                 n,
                                 deepest_parent_idx)
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
                        let mut position_parents = graph.x_position_parent_recursion(idx);
                        if is_already_offset(position_parents) { 0.0 } else { scroll.offset }
                    }).unwrap_or(0.0);

                    let y_offset = depth_parent_widget.maybe_y_scroll_state.map(|scroll| {
                        let mut position_parents = graph.y_position_parent_recursion(idx);
                        if is_already_offset(position_parents) { 0.0 } else { scroll.offset }
                    }).unwrap_or(0.0);

                    return [x_offset, y_offset];
                }
            }
        }
    }

    NO_OFFSET
}
