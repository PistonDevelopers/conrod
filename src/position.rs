
use point::Point;
use ui_context::UIID;
use UiContext;

/// A trait that indicates whether or not a widget
/// builder is positionable.
pub trait Positionable {
    fn point(self, pos: Point) -> Self;
    fn position(self, x: f64, y: f64) -> Self;
    fn down(self, padding: f64, uic: &UiContext) -> Self;
    fn up(self, padding: f64, uic: &UiContext) -> Self;
    fn left(self, padding: f64, uic: &UiContext) -> Self;
    fn right(self, padding: f64, uic: &UiContext) -> Self;
    fn down_from(self, ui_id: UIID, padding: f64, uic: &UiContext) -> Self;
    fn up_from(self, ui_id: UIID, padding: f64, uic: &UiContext) -> Self;
    fn left_from(self, ui_id: UIID, padding: f64, uic: &UiContext) -> Self;
    fn right_from(self, ui_id: UIID, padding: f64, uic: &UiContext) -> Self;
}
