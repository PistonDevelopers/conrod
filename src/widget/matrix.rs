
use graphics::character::CharacterCache;
use position::{self, Depth, Dimensions, HorizontalAlign, Point, Position, VerticalAlign};
use theme::Theme;
use ui::{self, GlyphCache, Ui};

/// Reaction params.
pub type WidgetNum = usize;
pub type ColNum = usize;
pub type RowNum = usize;
pub type Width = f64;
pub type Height = f64;
pub type PosX = f64;
pub type PosY = f64;

/// Draw a matrix of any rectangular widget type, where the matrix will provide a function with
/// the widget number, it's `rows` and `cols` position, the width and height for the widget and
/// the location at which the widget should be drawn.
#[derive(Copy, Clone)]
pub struct Matrix {
    cols: usize,
    rows: usize,
    pos: Position,
    dim: Dimensions,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    cell_pad_w: f64,
    cell_pad_h: f64,
}

// /// A cell to be returned via the cell reaction.
// pub struct MatrixCell<'a>(&'a mut UiContext, WidgetNum, ColNum, RowNum, PosX, PosY, Width, Height);

impl Matrix {

    /// Create a widget matrix context.
    pub fn new(cols: usize, rows: usize) -> Matrix {
        Matrix {
            cols: cols,
            rows: rows,
            pos: Position::default(),
            dim: [256.0, 256.0],
            maybe_h_align: None,
            maybe_v_align: None,
            cell_pad_w: 0.0,
            cell_pad_h: 0.0,
        }
    }

    /// The reaction called for each widget in the matrix. This should be called following all
    /// builder methods.
    pub fn each_widget<C, F>(&mut self, ui: &mut Ui<C>, mut react: F)
        where
            F: FnMut(&mut Ui<C>, WidgetNum, ColNum, RowNum, Point, Dimensions)
    {
        use utils::map_range;
        if let Some(id) = ui::parent_from_position(ui, self.pos) {
            ui::set_current_parent_id(ui, id);
        }
        let dim = self.dim;
        let h_align = self.maybe_h_align.unwrap_or(ui.theme.align.horizontal);
        let v_align = self.maybe_v_align.unwrap_or(ui.theme.align.vertical);
        let xy = ui.get_xy(self.pos, dim, h_align, v_align);
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
                react(ui, widget_num, col, row, [x, y], [w, h]);
                widget_num += 1;
            }
        }
    }

    /// A builder method for adding padding to the cell.
    pub fn cell_padding(self, w: f64, h: f64) -> Matrix {
        Matrix { cell_pad_w: w, cell_pad_h: h, ..self }
    }

}

impl position::Positionable for Matrix {
    fn position(mut self, pos: Position) -> Self {
        self.pos = pos;
        self
    }
    fn get_position(&self, _: &Theme) -> Position { self.pos }
    #[inline]
    fn horizontal_align(self, h_align: HorizontalAlign) -> Self {
        Matrix { maybe_h_align: Some(h_align), ..self }
    }
    #[inline]
    fn vertical_align(self, v_align: VerticalAlign) -> Self {
        Matrix { maybe_v_align: Some(v_align), ..self }
    }
    fn get_horizontal_align(&self, theme: &Theme) -> HorizontalAlign {
        self.maybe_h_align.unwrap_or(theme.align.horizontal)
    }
    fn get_vertical_align(&self, theme: &Theme) -> VerticalAlign {
        self.maybe_v_align.unwrap_or(theme.align.vertical)
    }
    fn depth(self, _depth: Depth) -> Self { self }
    fn get_depth(&self) -> Depth { 0.0 }
}

impl position::Sizeable for Matrix {
    #[inline]
    fn width(self, w: f64) -> Self {
        let h = self.dim[1];
        Matrix { dim: [w, h], ..self }
    }
    #[inline]
    fn height(self, h: f64) -> Self {
        let w = self.dim[0];
        Matrix { dim: [w, h], ..self }
    }
    fn get_width<C: CharacterCache>(&self, _theme: &Theme, _: &GlyphCache<C>) -> f64 { self.dim[0] }
    fn get_height(&self, _theme: &Theme) -> f64 { self.dim[1] }
}

