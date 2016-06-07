/// Unique, public widget identifier. Each widget must use a unique `WidgetId` so that it's state
/// can be cached within the `Ui` type. The reason we use a usize is because widgets are cached
/// within a `Graph` whose max number of `Node`s is indexed by usize.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);

impl From<usize> for Id {
    #[inline]
    fn from(u: usize) -> Id { Id(u) }
}

/// Simplify the incrementation of an Id in `for` loops i.e. this allows:
///
/// for i in 0..num_widgets {
///     MyWidget::new().set(MY_WIDGET + i, ui);
/// }
///
impl ::std::ops::Add<usize> for Id {
    type Output = Id;
    fn add(self, rhs: usize) -> Id { Id(self.0 + rhs) }
}
