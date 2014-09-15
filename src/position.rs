
use ui_context::UIID;

/// A trait that indicates whether or not a widget
/// builder is positionable.
pub trait Positionable {
    fn position(self, x: f64, y: f64) -> Self;
    fn down(self, padding: f64) -> Self;
    fn up(self, padding: f64) -> Self;
    fn left(self, padding: f64) -> Self;
    fn right(self, padding: f64) -> Self;
    fn down_from(self, ui_id: UIID, padding: f64) -> Self;
    fn up_from(self, ui_id: UIID, padding: f64) -> Self;
    fn left_from(self, ui_id: UIID, padding: f64) -> Self;
    fn right_from(self, ui_id: UIID, padding: f64) -> Self;
}

