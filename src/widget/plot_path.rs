use {
    Color,
    Colorable,
    PointPath,
    Positionable,
    Scalar,
    Sizeable,
    Widget,
};
use num;
use utils;
use widget;

/// A widget that plots a series of lines using the given function *x -> y*.
///
/// The function is sampled once per pixel and the result is mapped to the widget's height.
///
/// The resulting "path" is drawn using conrod's `PointPath` primitive widget.
pub struct PlotPath<X, Y, F> {
    common: widget::CommonBuilder,
    style: Style,
    min_x: X,
    max_x: X,
    min_y: Y,
    max_y: Y,
    f: F,
}

/// The unique `WidgetKind` for the `PlotPath`.
pub const KIND: widget::Kind = "PlotPath";

widget_style!{
    KIND;
    /// Unique styling parameters for the `PlotPath` widget.
    style Style {
        /// The thickness of the plotted line.
        - thickness: Scalar { 1.0 }
        /// The color of the line.
        - color: Color { theme.shape_color }
    }
}

/// Unique state stored between updates for the `PlotPath` widget.
#[derive(Debug, PartialEq)]
pub struct State {
    point_path_idx: widget::IndexSlot,
}


impl<X, Y, F> PlotPath<X, Y, F> {

    /// Begin building a new `PlotPath` widget instance.
    pub fn new(min_x: X, max_x: X, min_y: Y, max_y: Y, f: F) -> Self {
        PlotPath {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
            min_x: min_x,
            max_x: max_x,
            min_y: min_y,
            max_y: max_y,
            f: f,
        }
    }

}


impl<X, Y, F> Widget for PlotPath<X, Y, F>
    where X: num::NumCast + Clone,
          Y: num::NumCast + Clone,
          F: FnMut(X) -> Y,
{
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> widget::Kind {
        KIND
    }

    fn init_state(&self) -> Self::State {
        State {
            point_path_idx: widget::IndexSlot::new(),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the PlotPath.
    fn update(self, args: widget::UpdateArgs<Self>) {

        let widget::UpdateArgs { idx, state, style, rect, mut ui, .. } = args;
        let PlotPath { min_x, max_x, min_y, max_y, mut f, .. } = self;

        let y_to_scalar =
            |y| utils::map_range(y, min_y.clone(), max_y.clone(), rect.bottom(), rect.top());
        let scalar_to_x =
            |s| utils::map_range(s, rect.left(), rect.right(), min_x.clone(), max_x.clone());

        let point_iter = (0 .. rect.w() as usize)
            .map(|x_scalar| {
                let x_scalar = x_scalar as Scalar + rect.x.start;
                let x = scalar_to_x(x_scalar);
                let y = f(x);
                let y_scalar = y_to_scalar(y);
                [x_scalar, y_scalar]
            });

        let point_path_idx = state.point_path_idx.get(&mut ui);
        let thickness = style.thickness(ui.theme());
        let color = style.color(ui.theme());
        PointPath::new(point_iter)
            .wh(rect.dim())
            .xy(rect.xy())
            .color(color)
            .thickness(thickness)
            .parent(idx)
            .graphics_for(idx)
            .set(point_path_idx, &mut ui);
    }

}

impl<X, Y, F> Colorable for PlotPath<X, Y, F> {
    builder_method!(color { style.color = Some(Color) });
}
