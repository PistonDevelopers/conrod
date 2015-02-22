
use dimensions::Dimensions;
use point::Point;
use Position;
use Size;

/// Callback params.
pub type WidgetNum = usize;
pub type ColNum = usize;
pub type RowNum = usize;
pub type Width = f64;
pub type Height = f64;
pub type PosX = f64;
pub type PosY = f64;

/// Draw a matrix of any rectangular widget type, where the
/// matrix will provide a callback with the widget number,
/// it's `rows` and `cols` position, the width and height
/// for the widget and the location at which the widget
/// should be drawn.
#[derive(Copy)]
pub struct WidgetMatrix {
    cols: usize,
    rows: usize,
    pos: Point,
    dim: Dimensions,
    cell_pad_w: f64,
    cell_pad_h: f64,
}

/*
/// A cell to be returned via the cell callback.
pub struct MatrixCell<'a>(&'a mut UiContext, WidgetNum, ColNum, RowNum, PosX, PosY, Width, Height);
*/

impl WidgetMatrix {

    /// The callback called for each widget in the matrix.
    /// This should be called following all builder methods.
    pub fn each_widget<F>(&mut self, mut callback: F)
        where
            F: FnMut(WidgetNum, ColNum, RowNum, Point, Dimensions)
    {
        let widget_w = self.dim[0] / self.cols as f64;
        let widget_h = self.dim[1] / self.rows as f64;
        let mut widget_num = 0;
        for col in 0..self.cols {
            for row in 0..self.rows {
                callback(
                    widget_num,
                    col,
                    row,
                    [self.pos[0] + (widget_w * col as f64) + self.cell_pad_w,
                     self.pos[1] + (widget_h * row as f64) + self.cell_pad_h],
                    [widget_w - self.cell_pad_w * 2.0,
                     widget_h - self.cell_pad_h * 2.0],
                );
                widget_num += 1;
            }
        }
    }

    /// A builder method for adding padding to the cell.
    pub fn cell_padding(self, w: f64, h: f64) -> WidgetMatrix {
        WidgetMatrix { cell_pad_w: w, cell_pad_h: h, ..self }
    }

    /*
    /// Create an iterator over the matrix cells.
    fn iter_cells(&mut self) -> CellIterator {
    }
    */

}

/*
/// A struct used for iterating over the cells of a WidgetMatrix.
pub struct CellIterator {
    row: usize,
    col: usize,
    rows: usize,
    cols: usize,
}

impl Iterator for CellIterator {
    fn next
    */

impl WidgetMatrix {

    /// Create a widget matrix context.
    pub fn new(cols: usize, rows: usize) -> WidgetMatrix {
        WidgetMatrix {
            cols: cols,
            rows: rows,
            pos: [0.0, 0.0],
            dim: [256.0, 256.0],
            cell_pad_w: 0.0,
            cell_pad_h: 0.0,
        }
    }
}

quack! {
    wm: WidgetMatrix[]
    get:
        fn () -> Size [] { Size(wm.pos) }
    set:
        fn (val: Position) [] { wm.pos = val.0 }
        fn (val: Size) [] { wm.dim = val.0 }
    action:
}
