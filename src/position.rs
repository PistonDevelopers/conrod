use piston::quack::{ Pair, Set, SetAt };
use point::Point;
use ui_context::UIID;
use UiContext;
use graphics::vecmath::Scalar;

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

/// Position property.
#[derive(Copy)]
pub struct Position(pub [Scalar; 2]);

impl<T> Positionable for T
    where
        (Position, T): Pair<Data = Position, Object = T> + SetAt
{

    #[inline]
    fn point(self, pos: Point) -> Self {
        self.set(Position(pos))
    }

    #[inline]
    fn position(self, x: f64, y: f64) -> Self {
        self.set(Position([x as Scalar, y as Scalar]))
    }

    #[inline]
    fn down(self, padding: f64, uic: &UiContext) -> Self {
        let (x, y) = uic.get_placing(uic.get_prev_uiid()).down(padding);
        self.set(Position([x, y]))
    }
    #[inline]
    fn up(self, padding: f64, uic: &UiContext) -> Self {
        let (x, y) = uic.get_placing(uic.get_prev_uiid()).up(padding);
        self.set(Position([x, y]))
    }
    #[inline]
    fn left(self, padding: f64, uic: &UiContext) -> Self {
        let (x, y) = uic.get_placing(uic.get_prev_uiid()).left(padding);
        self.set(Position([x, y]))
    }
    #[inline]
    fn right(self, padding: f64, uic: &UiContext) -> Self {
        let (x, y) = uic.get_placing(uic.get_prev_uiid()).right(padding);
        self.set(Position([x, y]))
    }

    #[inline]
    fn down_from(self, uiid: u64, padding: f64, uic: &UiContext) -> Self {
        let (x, y) = uic.get_placing(uiid).down(padding);
        self.set(Position([x, y]))
    }
    #[inline]
    fn up_from(self, uiid: u64, padding: f64, uic: &UiContext) -> Self {
        let (x, y) = uic.get_placing(uiid).up(padding);
        self.set(Position([x, y]))
    }
    #[inline]
    fn left_from(self, uiid: u64, padding: f64, uic: &UiContext) -> Self {
        let (x, y) = uic.get_placing(uiid).left(padding);
        self.set(Position([x, y]))
    }
    #[inline]
    fn right_from(self, uiid: u64, padding: f64, uic: &UiContext) -> Self {
        let (x, y) = uic.get_placing(uiid).right(padding);
        self.set(Position([x, y]))
    }
}
