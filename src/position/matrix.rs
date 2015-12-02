use {CharacterCache, Dimension};
use super::{Depth, Dimensions, HorizontalAlign, VerticalAlign, Point, Position, Positionable,
            Scalar, Sizeable};
use theme::Theme;
use ui::{self, Ui};
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
    maybe_position: Option<Position>,
    maybe_x_dimension: Option<Dimension>,
    maybe_y_dimension: Option<Dimension>,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    cell_pad_w: Scalar,
    cell_pad_h: Scalar,
}

impl Matrix {

    /// Start building a new position **Matrix**.
    pub fn new(cols: usize, rows: usize) -> Matrix {
        Matrix {
            cols: cols,
            rows: rows,
            maybe_position: None,
            maybe_x_dimension: None,
            maybe_y_dimension: None,
            maybe_h_align: None,
            maybe_v_align: None,
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

        let pos = self.get_position(&ui.theme);
        let dim = self.get_dim(ui).unwrap_or([0.0, 0.0]);
        let (h_align, v_align) = self.get_alignment(&ui.theme);

        // If we can infer some new current parent from the position, set that as the current
        // parent within the given `Ui`.
        if let Some(id) = ui::parent_from_position(ui, pos) {
            ui::set_current_parent_idx(ui, id);
        }

        let xy = ui.calc_xy(None, pos, dim, h_align, v_align, true);
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

impl Positionable for Matrix {
    #[inline]
    fn position(mut self, pos: Position) -> Self {
        self.maybe_position = Some(pos);
        self
    }
    #[inline]
    fn get_position(&self, theme: &Theme) -> Position {
        self.maybe_position.unwrap_or(theme.position)
    }
    #[inline]
    fn horizontal_align(mut self, h_align: HorizontalAlign) -> Self {
        self.maybe_h_align = Some(h_align);
        self
    }
    #[inline]
    fn vertical_align(mut self, v_align: VerticalAlign) -> Self {
        self.maybe_v_align = Some(v_align);
        self
    }
    #[inline]
    fn get_horizontal_align(&self, theme: &Theme) -> HorizontalAlign {
        self.maybe_h_align.unwrap_or(theme.align.horizontal)
    }
    #[inline]
    fn get_vertical_align(&self, theme: &Theme) -> VerticalAlign {
        self.maybe_v_align.unwrap_or(theme.align.vertical)
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

impl Sizeable for Matrix {
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
    fn get_x_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        const DEFAULT_WIDTH: Dimension = Dimension::Absolute(256.0);
        self.maybe_x_dimension.or_else(|| {
            ui.theme.widget_style::<widget::matrix::Style>(widget::matrix::KIND)
                .map(|default| default.common.maybe_x_dimension.unwrap_or(DEFAULT_WIDTH))
        }).unwrap_or(DEFAULT_WIDTH)
    }
    #[inline]
    fn get_y_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        const DEFAULT_HEIGHT: Dimension = Dimension::Absolute(256.0);
        self.maybe_y_dimension.or_else(|| {
            ui.theme.widget_style::<widget::matrix::Style>(widget::matrix::KIND)
                .map(|default| default.common.maybe_y_dimension.unwrap_or(DEFAULT_HEIGHT))
        }).unwrap_or(DEFAULT_HEIGHT)
    }
}

