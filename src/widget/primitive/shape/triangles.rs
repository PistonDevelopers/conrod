

/// A widget that allows for drawing a list of triangles.
///
/// Drawing lists of triangles maps particularly nicely to 
///
/// - Solid colours
/// - Color per triangle
/// - Color per vertex
///
#[derive(Copy, Clone, Debug, WidgetCommon_)]
pub struct Triangles<S, I> {
    /// Data necessary and common for all widget builder types.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// Unique styling for the **Triangles**.
    pub style: S,
    /// All the point in the triangle list.
    pub points: I,
}


pub trait TriangleStyle {
    type Vertex;
}

/// All triangles colored with a single `Color`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SolidColor(Color);

/// Each triangle is colored per vertex.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ColorPerVertex;


impl TriangleStyle for Solid {
    type Vertex = Point;
}

impl TriangleStyle for Colored {
    type Vertex = (Point, Color);
}


// ///
// pub trait Points {
// }
// 
// pub struct FixedSizeArray


impl<S, I> Triangles<S, I> {
    fn new(style: S, points: I) -> Self {
        Triangles {
            common: widget::CommonBuilder::default(),
            style: style,
            points: points,
        }
    }
}

impl<I> Triangles<Solid, I> {
    /// A list of triangles described by the given points.
    ///
    /// All triangles are colored with the given `Color`.
    pub fn solid_color(color: Color, points: I) -> Self
        where I: Iterator<Item=Solid::Vertex>,
    {
        let style = Solid { color: color };
        Triangles::new(style, points)
    }
}

impl<I> Triangles<Colored, I> {
    /// A list of triangles described by the given points.
    ///
    /// Every vertex specifies its own unique color.
    pub fn color_per_vertex(points: I) -> Self
        where I: Iterator<Item=ColorPerVertex::Vertex>,
    {
        Triangles::new(ColorPerVertex, points)
    }
}
