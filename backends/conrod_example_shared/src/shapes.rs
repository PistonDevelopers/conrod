
use conrod_core::{
    Colorable,
    Widget,
    Positionable,
    Sizeable,
    Ui,
    UiCell,
    widget,
};

use layout::*;

widget_ids! {
    pub struct Ids {
        shapes_canvas,
        rounded_rectangle,
        shapes_left_col,
        shapes_right_col,
        shapes_title,
        line,
        point_path,
        rectangle_fill,
        rectangle_outline,
        trapezoid,
        oval_fill,
        oval_outline,
        circle,
    }
}

pub struct Gui {
    ids: Ids,
}

impl Gui {
    pub fn new(ui: &mut Ui) -> Self {
        Self {
            ids: Ids::new(ui.widget_id_generator()),
        }
    }

    pub fn update(&self, ui: &mut UiCell, canvas: widget::Id) -> widget::Id {
        use std::iter::once;

        let ids = &self.ids;

        widget::Text::new("Lines and Shapes")
            .down(70.0)
            .align_middle_x_of(canvas)
            .font_size(SUBTITLE_SIZE)
            .set(ids.shapes_title, ui);

        // Lay out the shapes in two horizontal columns.
        //
        // TODO: Have conrod provide an auto-flowing, fluid-list widget that is more adaptive for these
        // sorts of situations.
        widget::Canvas::new()
            .down(0.0)
            .align_middle_x_of(canvas)
            .kid_area_w_of(canvas)
            .h(360.0)
            .color(conrod_core::color::TRANSPARENT)
            .pad(MARGIN)
            .flow_down(&[
                (ids.shapes_left_col, widget::Canvas::new()),
                (ids.shapes_right_col, widget::Canvas::new()),
            ])
            .set(ids.shapes_canvas, ui);

        let shapes_canvas_rect = ui.rect_of(ids.shapes_canvas).unwrap();
        let w = shapes_canvas_rect.w();
        let h = shapes_canvas_rect.h() * 5.0 / 6.0;
        let radius = 10.0;
        widget::RoundedRectangle::fill([w, h], radius)
            .color(conrod_core::color::CHARCOAL.alpha(0.25))
            .middle_of(ids.shapes_canvas)
            .set(ids.rounded_rectangle, ui);

        let start = [-40.0, -40.0];
        let end = [40.0, 40.0];
        widget::Line::centred(start, end).mid_left_of(ids.shapes_left_col).set(ids.line, ui);

        let left = [-40.0, -40.0];
        let top = [0.0, 40.0];
        let right = [40.0, -40.0];
        let points = once(left).chain(once(top)).chain(once(right));
        widget::PointPath::centred(points).right(SHAPE_GAP).set(ids.point_path, ui);

        widget::Rectangle::fill([80.0, 80.0]).right(SHAPE_GAP).set(ids.rectangle_fill, ui);

        widget::Rectangle::outline([80.0, 80.0]).right(SHAPE_GAP).set(ids.rectangle_outline, ui);

        let bl = [-40.0, -40.0];
        let tl = [-20.0, 40.0];
        let tr = [20.0, 40.0];
        let br = [40.0, -40.0];
        let points = once(bl).chain(once(tl)).chain(once(tr)).chain(once(br));
        widget::Polygon::centred_fill(points).mid_left_of(ids.shapes_right_col).set(ids.trapezoid, ui);

        widget::Oval::fill([40.0, 80.0]).right(SHAPE_GAP + 20.0).align_middle_y().set(ids.oval_fill, ui);

        widget::Oval::outline([80.0, 40.0]).right(SHAPE_GAP + 20.0).align_middle_y().set(ids.oval_outline, ui);

        widget::Circle::fill(40.0).right(SHAPE_GAP).align_middle_y().set(ids.circle, ui);

        ids.shapes_canvas
    }
}
