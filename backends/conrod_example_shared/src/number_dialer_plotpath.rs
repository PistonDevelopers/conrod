
use conrod_core::{
    Widget,
    Labelable,
    Positionable,
    Scalar,
    Sizeable,
    Ui,
    UiCell,
    widget,
};

use layout::*;

widget_ids! {
    pub struct Ids {
        dialer_title,
        number_dialer,
        plot_path,
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
    pub fn update(&self, ui: &mut UiCell, sine_frequency: &mut f32, canvas: widget::Id, last: widget::Id, space: Scalar) -> widget::Id {
        let ids = &self.ids;

        widget::Text::new("NumberDialer and PlotPath")
            .down_from(last, space)
            .align_middle_x_of(canvas)
            .font_size(SUBTITLE_SIZE)
            .set(ids.dialer_title, ui);

        // Use a `NumberDialer` widget to adjust the frequency of the sine wave below.
        let min = 0.5;
        let max = 200.0;
        let decimal_precision = 1;
        for new_freq in widget::NumberDialer::new(*sine_frequency, min, max, decimal_precision)
            .down(60.0)
            .align_middle_x_of(canvas)
            .w_h(160.0, 40.0)
            .label("F R E Q")
            .set(ids.number_dialer, ui)
        {
            *sine_frequency = new_freq;
        }

        // Use the `PlotPath` widget to display a sine wave.
        let min_x = 0.0;
        let max_x = std::f32::consts::PI * 2.0 * *sine_frequency;
        let min_y = -1.0;
        let max_y = 1.0;
        widget::PlotPath::new(min_x, max_x, min_y, max_y, f32::sin)
            .kid_area_w_of(canvas)
            .h(240.0)
            .down(60.0)
            .align_middle_x_of(canvas)
            .set(ids.plot_path, ui);
        
        ids.plot_path // Return id of widget that the next Gui should be down_from
    }

}
