use point::Point;
use ui_context::UIID;
use UiContext;
use graphics::math::Scalar;

/// A trait that indicates whether or not a widget
/// builder is positionable.
pub trait Positionable: Sized {
    fn point(self, pos: Point) -> Self;
    fn position(self, x: f64, y: f64) -> Self {
        self.point([x, y])
    }
    fn down<C>(self, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(uic.get_prev_uiid()).down(padding);
        self.point([x, y])
    }
    fn up<C>(self, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(uic.get_prev_uiid()).up(padding);
        self.point([x, y])
    }
    fn left<C>(self, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(uic.get_prev_uiid()).left(padding);
        self.point([x, y])
    }
    fn right<C>(self, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(uic.get_prev_uiid()).right(padding);
        self.point([x, y])
    }
    fn down_from<C>(self, ui_id: UIID, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(ui_id).down(padding);
        self.point([x, y])
    }
    fn up_from<C>(self, ui_id: UIID, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(ui_id).up(padding);
        self.point([x, y])
    }
    fn left_from<C>(self, ui_id: UIID, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(ui_id).left(padding);
        self.point([x, y])
    }
    fn right_from<C>(self, ui_id: UIID, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(ui_id).right(padding);
        self.point([x, y])
    }
}

/// Position property.
#[derive(Copy, Clone)]
pub struct Position(pub [Scalar; 2]);
