//! A simple, non-interactive **Polygon** widget for drawing arbitrary convex shapes.

use {Color, Colorable, Point, Positionable, Sizeable, Theme, Widget};
use graph;
use super::Style;
use widget;
use widget::triangles::Triangle;
use utils::{bounding_box_for_points, vec2_add, vec2_sub};
use polygon2::{triangulate};
use rtriangulate;
/// A basic, non-interactive, arbitrary **Polygon** widget.
///
/// The **Polygon** is described by specifying its corners in order.
///
/// **Polygon** will automatically close all shapes, so the given list of points does not need to
/// start and end with the same position.
#[derive(Copy, Clone, Debug, WidgetCommon_)]
pub struct Polygon<I> {
    /// Data necessary and common for all widget builder types.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// The points describing the corners of the **Polygon**.
    pub points: I,
    /// Unique styling for the **Polygon**.
    pub style: Style,
    /// Whether or not the points should be automatically centred to the widget position.
    pub maybe_shift_to_centre_from: Option<Point>,
    /// reflect y-axis after triangulation
    pub reflect:bool,
    /// Removes faces formed in these points, white_points_index
    pub white_points_index: Option<usize>
}

/// Unique state for the **Polygon**.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// Whether the rectangle is drawn as an outline or a filled color.
    kind: Kind,
    /// An owned version of the points yielded by the **Polygon**'s `points` iterator.
    pub points: Vec<Point>,
    /// reflect y-axis after triangulation
    pub reflect:bool,
    /// Removes faces formed in these points, white_points_index
    pub white_points_index: Option<usize>
}

/// Whether the rectangle is drawn as an outline or a filled color.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Kind {
    /// Only the outline of the rectangle is drawn.
    Outline,
    /// The rectangle area is filled with some color.
    Fill,
}

/// An iterator that triangulates a polygon represented by a sequence of points describing its
/// edges.
#[derive(Clone)]
pub struct Triangles<I> {
    points: I,
}


impl<I> Polygon<I> {

    /// Build a polygon with the given points and style.
    pub fn styled(points: I, style: Style) -> Self {
        Polygon {
            points: points,
            common: widget::CommonBuilder::default(),
            style: style,
            reflect:false,
            maybe_shift_to_centre_from: None,
            white_points_index:None
        }
    }

    /// Build a **Polygon** with the default **Fill** style.
    pub fn fill(points: I) -> Self {
        Polygon::styled(points, Style::fill())
    }

    /// Build a **Polygon** **Fill**ed with the given **Color**.
    pub fn fill_with(points: I, color: Color) -> Self {
        Polygon::styled(points, Style::fill_with(color))
    }

    /// Build a **Polygon** with the default **Outline** style.
    pub fn outline(points: I) -> Self {
        Polygon::styled(points, Style::outline())
    }

    /// Build a **Polygon** **Outline**ed with the given line style.
    pub fn outline_styled(points: I, style: widget::line::Style) -> Self {
        Polygon::styled(points, Style::outline_styled(style))
    }

    /// Build a new filled **Polygon** whose bounding box is fit to the absolute co-ordinates of
    /// the points.
    ///
    /// This requires that the `points` iterator is `Clone` so that we may iterate through and
    /// determine the bounding box of the `points`.
    ///
    /// If you would rather centre the points to the middle of the bounding box, use
    /// the [**Polygon::centred**](./struct.Polygon#method.centred) methods instead.
    pub fn abs_styled(points: I, style: Style) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        let points_clone = points.clone().into_iter();
        let (xy, dim) = bounding_box_for_points(points_clone).xy_dim();
        Polygon::styled(points, style).wh(dim).xy(xy)
    }

    /// The same as [**Polygon::abs_styled**](./struct.Polygon#method.abs_styled) but builds the
    /// **Polygon** with the default **Fill** style.
    pub fn abs_fill(points: I) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::abs_styled(points, Style::fill())
    }

    /// The same as [**Polygon::abs_styled**](./struct.Polygon#method.abs_styled) but builds the
    /// **Polygon** **Fill**ed with the given **Color**.
    pub fn abs_fill_with(points: I, color: Color) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::abs_styled(points, Style::fill_with(color))
    }

    /// The same as [**Polygon::abs_styled**](./struct.Polygon#method.abs_styled) but builds the
    /// **Polygon** with the default **Outline** style.
    pub fn abs_outline(points: I) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::abs_styled(points, Style::outline())
    }

    /// The same as [**Polygon::abs_styled**](./struct.Polygon#method.abs_styled) but builds the
    /// **Polygon** with the given **Outline** styling.
    pub fn abs_outline_styled(points: I, style: widget::line::Style) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::abs_styled(points, Style::outline_styled(style))
    }

    /// Build a new **Polygon** and shift the location of the points so that the centre of their
    /// bounding rectangle lies at the position determined for the **Polygon** widget.
    ///
    /// This is useful if your points simply describe a shape and you want to position them using
    /// conrod's auto-layout and/or **Positionable** methods.
    ///
    /// If you would rather centre the bounding box to the points, use the
    /// [**Polygon::abs**](./struct.Polygon#method.abs) constructor method instead.
    pub fn centred_styled(points: I, style: Style) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        let points_clone = points.clone().into_iter();
        let (xy, dim) = bounding_box_for_points(points_clone).xy_dim();
        let mut polygon = Polygon::styled(points, style).wh(dim);
        polygon.maybe_shift_to_centre_from = Some(xy);
        polygon
    }

    /// The same as [**Polygon::centred_styled**](./struct.Polygon#method.centred_styled) but
    /// constructs the **Polygon** with the default **Fill** style.
    pub fn centred_fill(points: I) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::centred_styled(points, Style::fill())
    }

    /// The same as [**Polygon::centred_styled**](./struct.Polygon#method.centred_styled) but
    /// constructs the **Polygon** **Fill**ed with the given color.
    pub fn centred_fill_with(points: I, color: Color) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::centred_styled(points, Style::fill_with(color))
    }

    /// The same as [**Polygon::centred_styled**](./struct.Polygon#method.centred_styled) but
    /// constructs the **Polygon** with the default **Outline** style.
    pub fn centred_outline(points: I) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::centred_styled(points, Style::outline())
    }

    /// The same as [**Polygon::centred_styled**](./struct.Polygon#method.centred_styled) but
    /// constructs the **Polygon** **Outline**d with the given styling.
    pub fn centred_outline_styled(points: I, style: widget::line::Style) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::centred_styled(points, Style::outline_styled(style))
    }

}


impl<I> Widget for Polygon<I>
    where I: IntoIterator<Item=Point>,
{
    type State = State;
    type Style = Style;
    type Event = ();

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State {
            kind: Kind::Fill,
            points: Vec::new(),
            white_points_index:None,
            reflect: false
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn is_over(&self) -> widget::IsOverFn {
        is_over_widget
    }

    /// Update the state of the Polygon.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        use utils::{iter_diff, IterDiff};
        let widget::UpdateArgs { rect, state, style, .. } = args;
        let Polygon { points, maybe_shift_to_centre_from,reflect,white_points_index, .. } = self;
        
        // A function that compares the given points iterator to the points currently owned by
        // `State` and updates only if necessary.
        fn update_points<I>(state: &mut widget::State<State>, points: I)
            where I: IntoIterator<Item=Point>,
        {
            match iter_diff(&state.points, points) {
                Some(IterDiff::FirstMismatch(i, mismatch)) => state.update(|state| {
                    state.points.truncate(i);
                    state.points.extend(mismatch);
                }),
                Some(IterDiff::Longer(remaining)) =>
                    state.update(|state| state.points.extend(remaining)),
                Some(IterDiff::Shorter(total)) =>
                    state.update(|state| state.points.truncate(total)),
                None => (),
            }
        
        }

        // Check whether or not we need to centre the points.
        match maybe_shift_to_centre_from {
            Some(original) => {
                let xy = rect.xy();
                let difference = vec2_sub(xy, original);
                let point_iter = points.into_iter().map(|point| vec2_add(point, difference));
                update_points(state,point_iter)
            },
            None => update_points(state, points),
        }

        let kind = match *style {
            Style::Fill(_) => Kind::Fill,
            Style::Outline(_) => Kind::Outline,
        };

        if state.kind != kind {
            state.update(|state| state.kind = kind);
        }
        if state.reflect != reflect {
            state.update(|state| state.reflect = reflect);
        }
        if state.white_points_index != white_points_index {
            state.update(|state| state.white_points_index = white_points_index);
        }
    }

}


impl<I> Colorable for Polygon<I> {
    fn color(mut self, color: Color) -> Self {
        self.style.set_color(color);
        self
    }
}

impl<T>Polygon<T>{
    /// Reflect y-axis
    pub fn reflect(mut self) ->Self{
        self.reflect = true;
        self
    }
    /// Set White points, Remove faces form in these points
    pub fn white_points_index(mut self,white_points_index:usize)->Self
    {
        self.white_points_index = Some(white_points_index);
        self
    }
}

/// Triangulate the polygon given as a list of `Point`s describing its sides.
///
/// Returns `None` if the given iterator yields less than two points.
pub fn triangles<I>(points: I,reflect:bool,white_points_index:Option<usize>) -> Option<Triangles<I::IntoIter>>
    where I: IntoIterator<Item=Point>,
{
    
    //let points = points.into_iter();
    points.extend([2.0,2.0]);
    let mut points_c = points.into_iter().collect::<Vec<Point>>();
    let mut points_k = vec![];
    if let None = white_points_index{
        let triangles = triangulate(&points_c);
        
        let l = if reflect{
            -1.0
        }else{
            1.0
        };
        for ta in triangles{
            points_k.push([points_c[ta][0],l*points_c[ta][1]]);
        }
    }else if let Some(white_index) = white_points_index{
        let s = points_c.split_at(white_index-1).clone();
        let subject = s.1;
        let clip = s.0;
        let det =  points_c.iter().map(|t| rtriangulate::TriangulationPoint::new(t[0],t[1])).collect::<Vec<rtriangulate::TriangulationPoint<f64>>>();
        let det_clip = clip.iter().map(|t| rtriangulate::TriangulationPoint::new(t[0],t[1])).collect::<Vec<rtriangulate::TriangulationPoint<f64>>>();
        let subject_triangles = rtriangulate::triangulate(&det).unwrap();
        let clip_triangles = rtriangulate::triangulate(&det_clip).unwrap();
        for i in subject_triangles{
            let mut in_white =false;
            let rtriangulate::Triangle(p1,p2,p3) = i;
            for y in &clip_triangles{
                let rtriangulate::Triangle(y1,y2,y3) = y;
                if p1*p1+p2*p2+p3*p3 == y1*y1+y2*y2+y3*y3{
                    in_white = true;
                    break;
                }
            }
            if !in_white{
                points_k.push(points_c.get(p1.clone()).unwrap().clone());
                points_k.push(points_c.get(p2.clone()).unwrap().clone());
                points_k.push(points_c.get(p3.clone()).unwrap().clone());
            }
        }
    }
    Some(Triangles {
        points: points_k.into_iter(),
    })
}

impl<I> Iterator for Triangles<I>
    where I: Iterator<Item=Point>,
{
    type Item = Triangle<Point>;
    fn next(&mut self) -> Option<Self::Item> {
        let point = match self.points.next() {
            Some(p) => p,
            None => return None,
        };
        Some(Triangle([point]))
    }
}

/// Returns `true` if the given `Point` is over the polygon described by the given series of
/// points.
pub fn is_over<I>(points: I, point: Point) -> bool
where
    I: IntoIterator<Item=Point>,
{
    triangles(points).map(|ts| widget::triangles::is_over(ts, point)).unwrap_or(false)
}

/// The function to use for picking whether a given point is over the polygon.
pub fn is_over_widget(widget: &graph::Container, point: Point, _: &Theme) -> widget::IsOver {
    widget
        .state_and_style::<State, Style>()
        .map(|widget| is_over(widget.state.points.iter().cloned(), point))
        .unwrap_or_else(|| widget.rect.is_over(point))
        .into()
}
