use conrod_core::{
    Positionable,
    Sizeable,
    Ui,
    UiCell,
    Widget,
    widget,
};

use layout::*;

widget_ids! {
    pub struct Ids {
        image_title,
        rust_logo,
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

    /// Returns id of widget that the next Gui should be down_from
    pub fn update(&self, ui: &mut UiCell, rust_logo: conrod_core::image::Id, canvas: widget::Id, last: widget::Id) -> widget::Id {
        let ids = &self.ids;

        widget::Text::new("Image")
            .down_from(last, MARGIN)
            .align_middle_x_of(canvas)
            .font_size(SUBTITLE_SIZE)
            .set(ids.image_title, ui);

        const LOGO_SIDE: conrod_core::Scalar = 144.0;
        widget::Image::new(rust_logo)
            .w_h(LOGO_SIDE, LOGO_SIDE)
            .down(60.0)
            .align_middle_x_of(canvas)
            .set(ids.rust_logo, ui);

        ids.rust_logo // Return id of widget that the next Gui should be down_from
    }
}
