use {Backend, CharacterCache, Ui};
use super::{Depth, Dimension, Dimensions, Point, Position, Positionable, Scalar, Sizeable};
use ui;
use widget;

pub type WidgetNum = usize;
pub type ColNum = usize;
pub type RowNum = usize;
pub type Width = f64;
pub type Height = f64;
pub type PosX = f64;
pub type PosY = f64;

/// A type to simplify placement of various widgets in a matrix or grid layout.
#[derive(Copy, Clone, Debug)]
pub struct Matrix {
    cols: usize,
    rows: usize,
    maybe_x_position: Option<Position>,
    maybe_y_position: Option<Position>,
    maybe_x_dimension: Option<Dimension>,
    maybe_y_dimension: Option<Dimension>,
    cell_pad_w: Scalar,
    cell_pad_h: Scalar,
}

impl Matrix {

    /// Start building a new position **Matrix**.
    pub fn new(cols: usize, rows: usize) -> Matrix {
        Matrix {
            cols: cols,
            rows: rows,
            maybe_x_position: None,
            maybe_y_position: None,
            maybe_x_dimension: None,
            maybe_y_dimension: None,
            cell_pad_w: 0.0,
            cell_pad_h: 0.0,
        }
    }

    /// Produce the matrix with the given cell padding.
    pub fn cell_padding(mut self, w: Scalar, h: Scalar) -> Matrix {
        self.cell_pad_w = w;
        self.cell_pad_h = h;
        self
    }

    /// Call the given function for every element in the Matrix.
    pub fn each_widget<C, F>(self, ui: &mut Ui<C>, mut f: F) where
        C: CharacterCache,
        F: FnMut(&mut Ui<C>, WidgetNum, ColNum, RowNum, Point, Dimensions),
    {
        use utils::map_range;

        let x_pos = self.get_x_position(ui);
        let y_pos = self.get_y_position(ui);
        let dim = self.get_wh(ui).unwrap_or([0.0, 0.0]);

        // If we can infer some new current parent from the position, set that as the current
        // parent within the given `Ui`.
        let parent_idx = ui::infer_parent_unchecked(ui, x_pos, y_pos);
        ui::set_current_parent_idx(ui, parent_idx);

        let xy = ui.calc_xy(None, x_pos, y_pos, dim, true);
        let (half_w, half_h) = (dim[0] / 2.0, dim[1] / 2.0);
        let widget_w = dim[0] / self.cols as f64;
        let widget_h = dim[1] / self.rows as f64;
        let x_min = -half_w + widget_w / 2.0;
        let x_max = half_w + widget_w / 2.0;
        let y_min = -half_h - widget_h / 2.0;
        let y_max = half_h - widget_h / 2.0;
        let mut widget_num = 0;
        for col in 0..self.cols {
            for row in 0..self.rows {
                let x = xy[0] + map_range(col as f64, 0.0, self.cols as f64, x_min, x_max);
                let y = xy[1] + map_range(row as f64, 0.0, self.rows as f64, y_max, y_min);
                let w = widget_w - self.cell_pad_w * 2.0;
                let h = widget_h - self.cell_pad_h * 2.0;
                f(ui, widget_num, col, row, [x, y], [w, h]);
                widget_num += 1;
            }
        }
    }

}

impl<B> Positionable<B> for Matrix
    where B: Backend,
{
    #[inline]
    fn x_position(mut self, pos: Position) -> Self {
        self.maybe_x_position = Some(pos);
        self
    }
    #[inline]
    fn y_position(mut self, pos: Position) -> Self {
        self.maybe_y_position = Some(pos);
        self
    }
    #[inline]
    fn get_x_position(&self, ui: &Ui<B>) -> Position {
        self.maybe_x_position.unwrap_or(ui.theme.x_position)
    }
    #[inline]
    fn get_y_position(&self, ui: &Ui<B>) -> Position {
        self.maybe_y_position.unwrap_or(ui.theme.y_position)
    }
    #[inline]
    fn depth(self, _: Depth) -> Self {
        unimplemented!();
    }
    #[inline]
    fn get_depth(&self) -> Depth {
        unimplemented!();
    }
}

impl<B> Sizeable<B> for Matrix
    where B: Backend,
{
    #[inline]
    fn x_dimension(mut self, w: Dimension) -> Self {
        self.maybe_x_dimension = Some(w);
        self
    }
    #[inline]
    fn y_dimension(mut self, h: Dimension) -> Self {
        self.maybe_y_dimension = Some(h);
        self
    }
    #[inline]
    fn get_x_dimension(&self, ui: &Ui<B>) -> Dimension {
        const DEFAULT_WIDTH: Dimension = Dimension::Absolute(256.0);
        self.maybe_x_dimension.or_else(|| {
            ui.theme.widget_style::<widget::matrix::Style>(widget::matrix::KIND)
                .map(|default| default.common.maybe_x_dimension.unwrap_or(DEFAULT_WIDTH))
        }).unwrap_or(DEFAULT_WIDTH)
    }
    #[inline]
    fn get_y_dimension(&self, ui: &Ui<B>) -> Dimension {
        const DEFAULT_HEIGHT: Dimension = Dimension::Absolute(256.0);
        self.maybe_y_dimension.or_else(|| {
            ui.theme.widget_style::<widget::matrix::Style>(widget::matrix::KIND)
                .map(|default| default.common.maybe_y_dimension.unwrap_or(DEFAULT_HEIGHT))
        }).unwrap_or(DEFAULT_HEIGHT)
    }
}
