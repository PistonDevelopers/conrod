//! A widget for displaying a grid of lines across two axes.

use {Color, Colorable, Point, Scalar, Widget};
use widget::{self, CommonBuilder, UpdateArgs};
use utils::map_range;

/// A widget for displaying a grid of lines across two axes.
#[derive(Copy, Clone, Debug, WidgetCommon_)]
pub struct Grid<X, Y, I> {
    /// Builder parameters that are common to all `Widget`s.
    #[conrod(common_builder)]
    pub common: CommonBuilder,
    /// Unique styling parameters for the `Grid` widget.
    pub style: Style,
    /// The minimum visible bound along the *x* axis.
    pub min_x: X,
    /// The maximum visible bound along the *x* axis.
    pub max_x: X,
    /// The minimum visible bound along the *y* axis.
    pub min_y: Y,
    /// The maximum visible bound along the *y* axis.
    pub max_y: Y,
    /// An offset for all vertical lines distributed across the *x* axis.
    pub x_offset: Option<X>,
    /// An offset for all horizontal lines distributed across the *y* axis.
    pub y_offset: Option<Y>,
    /// An iterator yielding each sequence of lines to be distributed across the grid.
    pub lines: I,
}

/// Unique styling parameters for the `Grid` widget.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// The color of the grid lines.
    #[conrod(default = "theme.shape_color")]
    pub color: Option<Color>,
    /// The thickness of the grid lines.
    #[conrod(default = "1.0")]
    pub thickness: Option<Scalar>,
}

/// A series of lines distributed across an axis.
#[derive(Copy, Clone, Debug)]
pub struct Lines<T> {
    /// The distance that separates each line.
    pub step: T,
    /// An optional offset for the lines along they're axis.
    pub offset: Option<T>,
    /// The thickness of each of the lines drawn.
    ///
    /// If `None`, the `thickness` specified within the `Style` is used.
    pub thickness: Option<Scalar>,
    /// The color of each of the lines drawn.
    ///
    /// If `None`, the `color` specified within the `Style` is used.
    pub color: Option<Color>,
}

/// A series of lines distributed over an axis.
#[derive(Copy, Clone, Debug)]
pub enum Axis<X, Y> {
    /// Vertical lines that are spread across the *x* axis.
    X(Lines<X>),
    /// Horizontal lines that are spread across the *y* axis.
    Y(Lines<Y>),
}

widget_ids! {
    struct Ids {
        lines[],
    }
}

/// Unique state for the `Grid` retained between updates.
pub struct State {
    ids: Ids,
}

impl<T> Lines<T> {
    /// Begin building a new set of lines for the grid `step` distance apart.
    ///
    /// Lines with a `step` that equates to `0.0` or less will not be drawn.
    pub fn step(step: T) -> Self {
        Lines {
            step: step,
            offset: None,
            thickness: None,
            color: None,
        }
    }

    /// Specify an offset for the grid.
    ///
    /// Offsets that are greater than the `step` size will be wrapped around the `step` size.
    pub fn offset(mut self, offset: T) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Specify a unique thickness for these lines.
    pub fn thickness(mut self, thickness: Scalar) -> Self {
        self.thickness = Some(thickness);
        self
    }

    /// Use the specified color to uniquely color the this set of lines.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Move the lines over the X axis.
    pub fn x<Y>(self) -> Axis<T, Y> {
        Axis::X(self)
    }

    /// Move the lines over the Y axis.
    pub fn y<X>(self) -> Axis<X, T> {
        Axis::Y(self)
    }
}

impl<X, Y, I> Grid<X, Y, I> {
    /// Begin building a new `PlotPath` widget instance.
    ///
    /// The first four arguments represent the visible range along both axes.
    ///
    /// The final argument is an iterator yielding `Lines` across either `Axis`. The given lines
    /// will be drawn in the order that they're given.
    pub fn new(min_x: X, max_x: X, min_y: Y, max_y: Y, lines: I) -> Grid<X, Y, I::IntoIter>
    where
        X: Into<Scalar>,
        Y: Into<Scalar>,
        I: IntoIterator<Item = Axis<X, Y>>,
    {
        Grid {
            common: CommonBuilder::default(),
            style: Style::default(),
            min_x: min_x,
            max_x: max_x,
            min_y: min_y,
            max_y: max_y,
            x_offset: None,
            y_offset: None,
            lines: lines.into_iter(),
        }
    }

    /// Specify an offset for all vertical lines placed along the X axis.
    pub fn x_offset(mut self, x: X) -> Self {
        self.x_offset = Some(x);
        self
    }

    /// Specify an offset for all horizontal lines placed along the Y axis.
    pub fn y_offset(mut self, y: Y) -> Self {
        self.y_offset = Some(y);
        self
    }
}

impl<X, Y, I> Widget for Grid<X, Y, I>
where
    X: Into<Scalar>,
    Y: Into<Scalar>,
    I: Iterator<Item = Axis<X, Y>>,
{
    type State = State;
    type Style = Style;
    type Event = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State { ids: Ids::new(id_gen) }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the PlotPath.
    fn update(self, args: UpdateArgs<Self>) -> Self::Event {
        let UpdateArgs {
            id,
            state,
            style,
            rect,
            ui,
            ..
        } = args;
        let Grid {
            min_x,
            max_x,
            min_y,
            max_y,
            x_offset,
            y_offset,
            lines,
            ..
        } = self;

        let min_x_f: Scalar = min_x.into();
        let max_x_f: Scalar = max_x.into();
        let len_x_f = max_x_f - min_x_f;
        let min_y_f: Scalar = min_y.into();
        let max_y_f: Scalar = max_y.into();
        let len_y_f = max_y_f - min_y_f;

        let x_to_scalar_len = |x: X| {
            let x_f: Scalar = x.into();
            map_range(x_f, 0.0, len_x_f, 0.0, rect.x.len())
        };

        let y_to_scalar_len = |y: Y| {
            let y_f: Scalar = y.into();
            map_range(y_f, 0.0, len_y_f, 0.0, rect.y.len())
        };

        let color = style.color(&ui.theme);
        let thickness = style.thickness(&ui.theme);
        let x_offset_f = x_offset.map(&x_to_scalar_len).unwrap_or(0.0);
        let y_offset_f = y_offset.map(&y_to_scalar_len).unwrap_or(0.0);
        let mut line_num = 0;

        let x_line = |x: Scalar| -> (Point, Point) {
            let a = [x, rect.y.start];
            let b = [x, rect.y.end];
            (a, b)
        };

        let y_line = |y: Scalar| -> (Point, Point) {
            let a = [rect.x.start, y];
            let b = [rect.x.end, y];
            (a, b)
        };

        macro_rules! draw_lines {
            ($lines:ident, $offset:expr, $to_scalar:ident, $step_range:expr, $line_points:ident) => {{
                let offset = $offset + $lines.offset.map(&$to_scalar).unwrap_or(0.0);
                let thickness = $lines.thickness.unwrap_or(thickness);
                let color = $lines.color.unwrap_or(color);
                let step = $to_scalar($lines.step);
                if step == 0.0 {
                    continue;
                }
                let mut pos = $step_range.start + offset % step;
                while $step_range.is_over(pos) {
                    // The start and end of the line.
                    let (a, b) = $line_points(pos);

                    // The unique identifier for this line.
                    if line_num >= state.ids.lines.len() {
                        state.update(|state| {
                            state.ids.lines.resize(line_num+1, &mut ui.widget_id_generator());
                        });
                    }
                    let line_id = state.ids.lines[line_num];

                    // Draw the line.
                    widget::Line::abs(a, b)
                        .color(color)
                        .thickness(thickness)
                        .parent(id)
                        .graphics_for(id)
                        .set(line_id, ui);

                    pos += step;
                    line_num += 1;
                }
            }};
        };

        for axis in lines {
            match axis {
                Axis::X(lines) => draw_lines!(lines, x_offset_f, x_to_scalar_len, rect.x, x_line),
                Axis::Y(lines) => draw_lines!(lines, y_offset_f, y_to_scalar_len, rect.y, y_line),
            }
        }
    }
}

impl<X, Y, I> Colorable for Grid<X, Y, I> {
    builder_method!(color { style.color = Some(Color) });
}
