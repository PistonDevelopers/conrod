
use position::{Dimensions, Point};
use std::cmp::Ordering;
use widget::{self, WidgetId};


/// Stores the state of a set of owned Widgets.
pub struct Cache {
    /// The states of the cached widgets.
    widgets: Vec<widget::Cached>,
    /// The UiId of the previously updated Widget.
    maybe_prev_widget_id: Option<WidgetId>,
}


impl Cache {

    /// Return the dimensions of a Canvas.
    pub fn widget_size(&self, id: WidgetId) -> Dimensions {
        let (w, h) = self.widgets[id].element.get_size();
        [w as f64, h as f64]
    }

    /// If the given Point is currently on a Widget, return the Id of that widget.
    pub fn pick_widget(&self, xy: Point) -> Option<WidgetId> {
        let mut widgets = self.widgets.iter().enumerate().filter(|&(_, ref widget)| {
            if let "EMPTY" = widget.kind { false } else { true }
        }).collect::<Vec<_>>();
        widgets.sort_by(|&(_, ref a), &(_, ref b)| compare_widget_depth(a, b));
        widgets.iter().rev().find(|&&(_, ref widget)| {
            use utils::is_over_rect;
            let (w, h) = widget.element.get_size();
            is_over_rect(widget.xy, xy, [w as f64, h as f64])
        }).map(|&(id, _)| id)
    }

}


/// Compare the rendering depth of two cached widgets.
fn compare_widget_depth(a: &widget::Cached, b: &widget::Cached) -> Ordering {
    a.depth.partial_cmp(&b.depth).unwrap()
}
