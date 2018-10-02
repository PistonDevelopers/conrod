//! A primitive widget that allows for drawing using a list of triangles.

use {Rect, Point, Positionable, Scalar, Sizeable, Theme, Widget};
use color;
use graph;
use std;
use utils::{vec2_add, vec2_sub};
use widget;

/// A widget that allows for drawing a list of triangles.
#[derive(Copy, Clone, Debug, WidgetCommon_)]
pub struct Triangles<S, I> {
    /// Data necessary and common for all widget builder types.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// Unique styling for the **Triangles**.
    pub style: S,
    /// All the point in the triangle list.
    pub triangles: I,
    /// Whether or not the triangles should be automatically centred to the widget position.
    pub maybe_shift_to_centre_from: Option<Point>,
}

/// Types used as vertices that make up a list of triangles.
pub trait Vertex: Clone + Copy + PartialEq {
    /// The x y location of the vertex.
    fn point(&self) -> Point;
    /// Add the given vector onto the position of self and return the result.
    fn add(self, Point) -> Self;
}

/// Unique styling types for `Triangles`.
pub trait Style: widget::Style + Clone + Send {
    /// The type of vertices that make up the list of triangles for this style.
    type Vertex: Vertex + Send;
}

/// All triangles colored with a single `Color`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SingleColor(pub color::Rgba);

/// Each triangle is colored per vertex.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MultiColor;

/// A single triangle described by three vertices.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Triangle<V>(pub [V; 3])
    where V: Vertex;

/// A point with an associated color.
pub type ColoredPoint = (Point, color::Rgba);

/// Unique state stored between updates for a `Triangles`.
#[derive(Clone, Debug, PartialEq)]
pub struct State<T> {
    /// The triangles that make up the triangles.
    pub triangles: T,
}


impl Vertex for Point {
    fn point(&self) -> Point {
        *self
    }
    fn add(self, add: Point) -> Self {
        vec2_add(self, add)
    }
}

impl Vertex for ColoredPoint {
    fn point(&self) -> Point {
        self.0
    }
    fn add(self, add: Point) -> Self {
        let (p, c) = self;
        (vec2_add(p, add), c)
    }
}

impl Style for SingleColor {
    type Vertex = Point;
}

impl Style for MultiColor {
    type Vertex = ColoredPoint;
}


/// When beginning to build `Triangles` they are initially unpositioned.
///
/// This is an intemediary type which allows the user to choose how to position the bounding
/// rectangle relative to the points.
#[derive(Copy, Clone, Debug)]
pub struct TrianglesUnpositioned<S, I> {
    triangles: Triangles<S, I>,
}


impl<V> Triangle<V>
    where V: Vertex,
{
    /// Shift the triangle by the given amount by adding it onto the position of each point.
    pub fn add(self, amount: Point) -> Self {
        let a = self[0].add(amount);
        let b = self[1].add(amount);
        let c = self[2].add(amount);
        Triangle([a, b, c])
    }

    /// The three points that make up the triangle.
    pub fn points(self) -> [Point; 3] {
        [self[0].point(), self[1].point(), self[2].point()]
    }
}

impl Triangle<Point> {
    /// Convert the `Triangle<Point>` to a `Triangle<ColoredPoint>`.
    pub fn color(self, a: color::Rgba, b: color::Rgba, c: color::Rgba) -> Triangle<ColoredPoint> {
        Triangle([(self[0], a), (self[1], b), (self[2], c)])
    }

    /// Convert the `Triangle<Point>` to a `Triangle<ColoredPoint>` using the given color.
    pub fn color_all(self, color: color::Rgba) -> Triangle<ColoredPoint> {
        Triangle([(self[0], color), (self[1], color), (self[2], color)])
    }
}

impl<V> std::ops::Deref for Triangle<V>
    where V: Vertex,
{
    type Target = [V; 3];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> From<[V; 3]> for Triangle<V>
    where V: Vertex,
{
    fn from(points: [V; 3]) -> Self {
        Triangle(points)
    }
}

impl<V> From<(V, V, V)> for Triangle<V>
    where V: Vertex,
{
    fn from((a, b, c): (V, V, V)) -> Self {
        Triangle([a, b, c])
    }
}

impl<V> Into<[V; 3]> for Triangle<V>
    where V: Vertex,
{
    fn into(self) -> [V; 3] {
        self.0
    }
}

impl<V> Into<(V, V, V)> for Triangle<V>
    where V: Vertex,
{
    fn into(self) -> (V, V, V) {
        (self[0], self[1], self[2])
    }
}


impl<S, I> Triangles<S, I> {
    fn new(style: S, triangles: I) -> Self {
        Triangles {
            common: widget::CommonBuilder::default(),
            style: style,
            triangles: triangles,
            maybe_shift_to_centre_from: None,
        }
    }
}

impl<I> Triangles<SingleColor, I>
    where I: IntoIterator<Item=Triangle<<SingleColor as Style>::Vertex>>,
{
    /// A list of triangles described by the given points.
    ///
    /// All triangles are colored with the given `Color`.
    pub fn single_color<C>(color: C, points: I) -> TrianglesUnpositioned<SingleColor, I>
        where C: Into<color::Rgba>,
    {
        let style = SingleColor(color.into());
        TrianglesUnpositioned::new(Triangles::new(style, points))
    }
}

impl<I> Triangles<MultiColor, I>
    where I: IntoIterator<Item=Triangle<<MultiColor as Style>::Vertex>>,
{
    /// A list of triangles described by the given points.
    ///
    /// Every vertex specifies its own unique color.
    pub fn multi_color(points: I) -> TrianglesUnpositioned<MultiColor, I> {
        TrianglesUnpositioned::new(Triangles::new(MultiColor, points))
    }
}

fn bounding_rect_for_triangles<I, V>(triangles: I) -> Rect
    where I: IntoIterator<Item=Triangle<V>>,
          V: Vertex,
{
    struct TriangleVertices<V> where V: Vertex {
        index: usize,
        triangle: Triangle<V>,
    }

    impl<V> Iterator for TriangleVertices<V> where V: Vertex {
        type Item = V;
        fn next(&mut self) -> Option<Self::Item> {
            let v = self.triangle.get(self.index).map(|&v| v);
            self.index += 1;
            v
        }
    }

    let points = triangles
        .into_iter()
        .flat_map(|t| {
            let vs = TriangleVertices { index: 0, triangle: t };
            vs.map(|v| v.point())
        });
    super::super::bounding_box_for_points(points)
}

impl<S, I> TrianglesUnpositioned<S, I>
    where S: Style,
          I: IntoIterator<Item=Triangle<S::Vertex>>,
          Triangles<S, I>: Widget,
{
    fn new(triangles: Triangles<S, I>) -> Self {
        TrianglesUnpositioned {
            triangles: triangles,
        }
    }

    /// Specify the bounding rectangle for the **Triangles**.
    ///
    /// Typically, the given `Rect` bounds should be the min and max positions along both axes that
    /// are touched by the **Triangles**' points.
    ///
    /// This method is significantly more efficient than `calc_bounding_rect` and
    /// `centre_points_to_bounding_rect` as the bounding rectangle does not have to be calculated
    /// from the **Triangles**' points.
    pub fn with_bounding_rect(self, rect: Rect) -> Triangles<S, I> {
        let TrianglesUnpositioned { triangles } = self;
        let (xy, dim) = rect.xy_dim();
        triangles.wh(dim).xy(xy)
    }

    /// Calculate the position and size of the bounding rectangle from the `Triangles` points. The
    /// resulting bounding rectangle will fit to the absolute co-ordinates of all points.
    ///
    /// In other words, this method will automatically call `Sizeable::wh` and `Positionable::xy`
    /// after calculating the size and position from the given points.
    ///
    /// This requires that the `points` iterator is `Clone` so that we may iterate through and
    /// determine the bounding box of the `points`. If you know the bounds of the rectangle ahead
    /// of time, we recommend calling `with_bounding_rect` instead as it will be significantly
    /// cheaper.
    ///
    /// If you would rather centre the points to the middle of the bounding box, use
    /// [**TrianglesUnpositioned::centre_points_to_bounding_rect**](./struct.TrianglesUnpositioned#method.centre_points_to_bounding_rect)
    /// instead.
    pub fn calc_bounding_rect(self) -> Triangles<S, I>
        where I: Clone,
    {
        let TrianglesUnpositioned { triangles } = self;
        let (xy, dim) = bounding_rect_for_triangles(triangles.triangles.clone()).xy_dim();
        triangles.wh(dim).xy(xy)
    }

    /// Shift the location of the **Triangles** points so that the centre of their bounding
    /// rectangle lies at the position determined for the **Triangles** widget.
    ///
    /// This is useful if your points simply describe a shape and you want to position them using
    /// conrod's auto-layout or **Positionable** and **Sizeable** methods.
    ///
    /// This requires that the `points` iterator is `Clone` so that we may iterate through and
    /// determine the bounding box of the `points`. If you know the bounds of the rectangle ahead
    /// of time, we recommend calling `with_bounding_rect` instead as it will be significantly
    /// cheaper.
    ///
    /// If you would rather calculate the bounding box *from* the given absolute points, use the
    /// [**TrianglesUnpositioned::calc_bounding_rect**](./struct.TrianglesUnpositioned#method.calc_bounding_rect)
    /// instead.
    pub fn centre_points_to_bounding_rect(self) -> Triangles<S, I>
        where I: Clone,
    {
        let TrianglesUnpositioned { mut triangles } = self;
        let (xy, dim) = bounding_rect_for_triangles(triangles.triangles.clone()).xy_dim();
        triangles.maybe_shift_to_centre_from = Some(xy);
        triangles.wh(dim)
    }
}

impl<S, I> Widget for Triangles<S, I>
    where S: Style,
          I: IntoIterator<Item=Triangle<S::Vertex>>,
{
    type State = State<Vec<Triangle<S::Vertex>>>;
    type Style = S;
    type Event = ();

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State {
            triangles: Vec::new(),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn is_over(&self) -> widget::IsOverFn {
        is_over_widget::<S>
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        use utils::{iter_diff, IterDiff};
        let widget::UpdateArgs { rect, state, .. } = args;
        let Triangles { triangles, maybe_shift_to_centre_from, .. } = self;

        // A function that compares the given triangles iterator to the triangles currently owned by
        // `State` and updates only if necessary.
        fn update_triangles<I>(state: &mut widget::State<State<Vec<I::Item>>>, triangles: I)
            where I: IntoIterator,
                  I::Item: PartialEq,
        {
            match iter_diff(&state.triangles, triangles) {
                Some(IterDiff::FirstMismatch(i, mismatch)) => state.update(|state| {
                    state.triangles.truncate(i);
                    state.triangles.extend(mismatch);
                }),
                Some(IterDiff::Longer(remaining)) =>
                    state.update(|state| state.triangles.extend(remaining)),
                Some(IterDiff::Shorter(total)) =>
                    state.update(|state| state.triangles.truncate(total)),
                None => (),
            }
        }

        match maybe_shift_to_centre_from {
            Some(original) => {
                let xy = rect.xy();
                let difference = vec2_sub(xy, original);
                let triangles = triangles.into_iter().map(|tri| tri.add(difference));
                update_triangles(state, triangles)
            },
            None => update_triangles(state, triangles),
        }
    }
}


/// Triangulates the given quad, represented by four points that describe its edges in either
/// clockwise or anti-clockwise order.
///
/// # Example
///
/// The following rectangle
///
/// ```ignore
///
///  a        b
///   --------
///   |      |
///   |      |
///   |      |
///   --------
///  d        c
///
/// ```
///
/// given as
///
/// ```ignore
/// from_quad([a, b, c, d])
/// ```
///
/// returns
///
/// ```ignore
/// (Triangle([a, b, c]), Triangle([a, c, d]))
/// ```
///
/// Here's a basic code example:
///
/// ```
/// extern crate conrod_core;
///
/// use conrod_core::widget::triangles::{from_quad, Triangle};
///
/// fn main() {
///     let a = [0.0, 1.0];
///     let b = [1.0, 1.0];
///     let c = [1.0, 0.0];
///     let d = [0.0, 0.0];
///     let quad = [a, b, c, d];
///     let triangles = from_quad(quad);
///     assert_eq!(triangles, (Triangle([a, b, c]), Triangle([a, c, d])));
/// }
/// ```
#[inline]
pub fn from_quad(points: [Point; 4]) -> (Triangle<Point>, Triangle<Point>) {
    let (a, b, c, d) = (points[0], points[1], points[2], points[3]);
    (Triangle([a, b, c]), Triangle([a, c, d]))
}

impl<V> AsRef<Triangle<V>> for Triangle<V>
where
    V: Vertex,
{
    fn as_ref(&self) -> &Triangle<V> {
        self
    }
}

/// Returns `true` if the given `Point` is over the given `Triangle`.
pub fn is_over_triangle<V>(t: &Triangle<V>, p: Point) -> bool
where
    V: Vertex,
{
    let ps = t.points();
    let (a, b, c) = (ps[0], ps[1], ps[2]);

    fn sign(a: Point, b: Point, c: Point) -> Scalar {
        (a[0] - c[0]) * (b[1] - c[1]) - (b[0] - c[0]) * (a[1] - c[1])
    }

    let b1 = sign(p, a, b) < 0.0;
    let b2 = sign(p, b, c) < 0.0;
    let b3 = sign(p, c, a) < 0.0;

    (b1 == b2) && (b2 == b3)
}

/// Returns `true` if the given `Point` is over any of the given `Triangle`s.
pub fn is_over<V, I, T>(ts: I, p: Point) -> bool
where
    V: Vertex,
    T: AsRef<Triangle<V>>,
    I: IntoIterator<Item=T>,
{
    ts.into_iter().any(|t| is_over_triangle(t.as_ref(), p))
}

/// The function to use for picking whether a given point is over the line.
pub fn is_over_widget<S>(widget: &graph::Container, point: Point, _: &Theme) -> widget::IsOver
where
    S: Style,
{
    widget
        .state_and_style::<State<Vec<Triangle<S::Vertex>>>, S>()
        .map(|widget| is_over(widget.state.triangles.iter().cloned(), point))
        .unwrap_or_else(|| widget.rect.is_over(point))
        .into()
}
