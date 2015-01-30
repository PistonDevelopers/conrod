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
    fn down<C>(self, padding: f64, uic: &UiContext<C>) -> Self;
    fn up<C>(self, padding: f64, uic: &UiContext<C>) -> Self;
    fn left<C>(self, padding: f64, uic: &UiContext<C>) -> Self;
    fn right<C>(self, padding: f64, uic: &UiContext<C>) -> Self;
    fn down_from<C>(self, ui_id: UIID, padding: f64, uic: &UiContext<C>) -> Self;
    fn up_from<C>(self, ui_id: UIID, padding: f64, uic: &UiContext<C>) -> Self;
    fn left_from<C>(self, ui_id: UIID, padding: f64, uic: &UiContext<C>) -> Self;
    fn right_from<C>(self, ui_id: UIID, padding: f64, uic: &UiContext<C>) -> Self;
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
    fn down<C>(self, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(uic.get_prev_uiid()).down(padding);
        self.set(Position([x, y]))
    }
    #[inline]
    fn up<C>(self, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(uic.get_prev_uiid()).up(padding);
        self.set(Position([x, y]))
    }
    #[inline]
    fn left<C>(self, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(uic.get_prev_uiid()).left(padding);
        self.set(Position([x, y]))
    }
    #[inline]
    fn right<C>(self, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(uic.get_prev_uiid()).right(padding);
        self.set(Position([x, y]))
    }

    #[inline]
    fn down_from<C>(self, uiid: u64, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(uiid).down(padding);
        self.set(Position([x, y]))
    }
    #[inline]
    fn up_from<C>(self, uiid: u64, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(uiid).up(padding);
        self.set(Position([x, y]))
    }
    #[inline]
    fn left_from<C>(self, uiid: u64, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(uiid).left(padding);
        self.set(Position([x, y]))
    }
    #[inline]
    fn right_from<C>(self, uiid: u64, padding: f64, uic: &UiContext<C>) -> Self {
        let (x, y) = uic.get_placing(uiid).right(padding);
        self.set(Position([x, y]))
    }
}
