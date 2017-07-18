//! A simple, non-interactive widget for drawing a single **Oval**.

use {Color, Colorable, Dimensions, Point, Rect, Scalar, Sizeable, Widget};
use super::Style as Style;
use widget;


/// A simple, non-interactive widget for drawing a single **Oval**.
#[derive(Copy, Clone, Debug, WidgetCommon_)]
pub struct Oval {
    /// Data necessary and common for all widget builder types.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// Unique styling.
    pub style: Style,
    /// The number of lines used to draw the edge.
    pub resolution: usize,
}

/// Unique state for the **Oval**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State {
    /// The number of lines used to draw the edge.
    pub resolution: usize,
}

/// The default circle resolution if none is specified.
pub const DEFAULT_RESOLUTION: usize = 50;


impl Oval {

    /// Build an **Oval** with the given dimensions and style.
    pub fn styled(dim: Dimensions, style: Style) -> Self {
        Oval {
            common: widget::CommonBuilder::default(),
            style: style,
            resolution: DEFAULT_RESOLUTION,
        }.wh(dim)
    }

    /// Build a new **Fill**ed **Oval**.
    pub fn fill(dim: Dimensions) -> Self {
        Oval::styled(dim, Style::fill())
    }

    /// Build a new **Oval** **Fill**ed with the given color.
    pub fn fill_with(dim: Dimensions, color: Color) -> Self {
        Oval::styled(dim, Style::fill_with(color))
    }

    /// Build a new **Outline**d **Oval** widget.
    pub fn outline(dim: Dimensions) -> Self {
        Oval::styled(dim, Style::outline())
    }

    /// Build a new **Oval** **Outline**d with the given style.
    pub fn outline_styled(dim: Dimensions, line_style: widget::line::Style) -> Self {
        Oval::styled(dim, Style::outline_styled(line_style))
    }

    /// The number of lines used to draw the edge.
    ///
    /// By default, `DEFAULT_RESOLUTION` is used.
    pub fn resolution(mut self, resolution: usize) -> Self {
        self.resolution = resolution;
        self
    }
}


impl Widget for Oval {
    type State = State;
    type Style = Style;
    type Event = ();

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State {
            resolution: DEFAULT_RESOLUTION,
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { state, .. } = args;
        if state.resolution != self.resolution {
            state.update(|state| state.resolution = self.resolution);
        }
    }

}


impl Colorable for Oval {
    fn color(mut self, color: Color) -> Self {
        self.style.set_color(color);
        self
    }
}


/// An iterator yielding the `Oval`'s edges as a circumference represented as a series of edges.
pub fn circumference(rect: Rect, resolution: usize) -> Circumference {
    use std::f64::consts::PI;
    let (x, y, w, h) = rect.x_y_w_h();
    Circumference {
        index: 0,
        num_points: resolution + 1,
        point: [x, y],
        half_w: w / 2.0,
        half_h: h / 2.0,
        rad_step: 2.0 * PI / resolution as Scalar,
    }
}

/// An iterator yielding the `Oval`'s edges as a circumference represented as a series of edges.
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct Circumference {
    index: usize,
    num_points: usize,
    point: Point,
    rad_step: Scalar,
    half_w: Scalar,
    half_h: Scalar,
}

impl Iterator for Circumference {
    type Item = Point;
    fn next(&mut self) -> Option<Self::Item> {
        let Circumference { ref mut index, num_points, point, rad_step, half_w, half_h } = *self;
        if *index >= num_points { return None; } else { *index += 1; }
        let x = point[0] + half_w * (rad_step * *index as Scalar).cos();
        let y = point[1] + half_h * (rad_step * *index as Scalar).sin();
        Some([x, y])
    }
}
