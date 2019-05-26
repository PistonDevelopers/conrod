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
        title,
        introduction,
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
    pub fn update(&self, ui: &mut UiCell, canvas: widget::Id) -> widget::Id {
        let ids = &self.ids;

        // We'll demonstrate the `Text` primitive widget by using it to draw a title and an
        // introduction to the example.
        const TITLE: &'static str = "All Widgets";
        widget::Text::new(TITLE).font_size(TITLE_SIZE).mid_top_of(canvas).set(ids.title, ui);

        const INTRODUCTION: &'static str =
            "This example aims to demonstrate all widgets that are provided by conrod.\
            \n\nThe widget that you are currently looking at is the Text widget. The Text widget \
            is one of several special \"primitive\" widget types which are used to construct \
            all other widget types. These types are \"special\" in the sense that conrod knows \
            how to render them via `conrod_core::render::Primitive`s.\
            \n\nScroll down to see more widgets!";
        widget::Text::new(INTRODUCTION)
            .padded_w_of(canvas, MARGIN)
            .down(60.0)
            .align_middle_x_of(canvas)
            .center_justify()
            .line_spacing(5.0)
            .set(ids.introduction, ui);

        ids.introduction // Return id of widget that the next Gui should be down_from
    }
}
