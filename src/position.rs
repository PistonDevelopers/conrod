
use point::Point;
use ui::{UiId, Ui};

/// Widgets that are positionable.
pub trait Positionable: Sized {

    /// Set the position with some Point.
    fn point(self, pos: Point) -> Self;

    /// Set the position with XY co-ords.
    fn position(self, x: f64, y: f64) -> Self {
        self.point([x, y])
    }

    /// Set the position as below the previous widget.
    fn down<C>(self, padding: f64, ui: &Ui<C>) -> Self {
        let (x, y) = ui.get_placing(ui.get_prev_uiid()).down(padding);
        self.point([x, y])
    }

    /// Set the position as above the previous widget.
    fn up<C>(self, padding: f64, ui: &Ui<C>) -> Self {
        let (x, y) = ui.get_placing(ui.get_prev_uiid()).up(padding);
        self.point([x, y])
    }

    /// Set the position to the left of the previous widget.
    fn left<C>(self, padding: f64, ui: &Ui<C>) -> Self {
        let (x, y) = ui.get_placing(ui.get_prev_uiid()).left(padding);
        self.point([x, y])
    }

    /// Set the position to the right of the previous widget.
    fn right<C>(self, padding: f64, ui: &Ui<C>) -> Self {
        let (x, y) = ui.get_placing(ui.get_prev_uiid()).right(padding);
        self.point([x, y])
    }

    /// Set the position as below the widget with the given UiId.
    fn down_from<C>(self, ui_id: UiId, padding: f64, ui: &Ui<C>) -> Self {
        let (x, y) = ui.get_placing(ui_id).down(padding);
        self.point([x, y])
    }

    /// Set the position as above the widget with the given UiId.
    fn up_from<C>(self, ui_id: UiId, padding: f64, ui: &Ui<C>) -> Self {
        let (x, y) = ui.get_placing(ui_id).up(padding);
        self.point([x, y])
    }

    /// Set the position to the left of the widget with the given UiId.
    fn left_from<C>(self, ui_id: UiId, padding: f64, ui: &Ui<C>) -> Self {
        let (x, y) = ui.get_placing(ui_id).left(padding);
        self.point([x, y])
    }

    /// Set the position to the right of the widget with the given UiId.
    fn right_from<C>(self, ui_id: UiId, padding: f64, ui: &Ui<C>) -> Self {
        let (x, y) = ui.get_placing(ui_id).right(padding);
        self.point([x, y])
    }

}
