
use dimensions::Dimensions;
use point::Point;
use ui_context::UiContext;

/// Callback params.
pub type WidgetNum = uint;
pub type ColNum = uint;
pub type RowNum = uint;
pub type Width = f64;
pub type Height = f64;
pub type PosX = f64;
pub type PosY = f64;

/// Draw a matrix of any rectangular widget type, where the
/// matrix will provide a callback with the widget number,
/// it's `rows` and `cols` position, the width and height
/// for the widget and the location at which the widget
/// should be drawn.
pub struct WidgetMatrixContext<'a> {
    uic: &'a mut UiContext,
    cols: uint,
    rows: uint,
    pos: Point,
    dim: Dimensions,
    cell_pad_w: f64,
    cell_pad_h: f64,
}

/*
/// A cell to be returned via the cell callback.
pub struct MatrixCell<'a>(&'a mut UiContext, WidgetNum, ColNum, RowNum, PosX, PosY, Width, Height);
*/

impl<'a> WidgetMatrixContext<'a> {

    /// The callback called for each widget in the matrix.
    /// This should be called following all builder methods.
    pub fn each_widget(&'a mut self, callback: |&mut UiContext, WidgetNum, ColNum, RowNum, Point, Dimensions|) {
        let widget_w = self.dim[0] / self.cols as f64;
        let widget_h = self.dim[1] / self.rows as f64;
        let mut widget_num = 0u;
        for col in range(0u, self.cols) {
            for row in range(0u, self.rows) {
                callback(
                    self.uic,
                    widget_num,
                    col,
                    row,
                    [self.pos[0] + (widget_w * col as f64) + self.cell_pad_w,
                     self.pos[1] + (widget_h * row as f64) + self.cell_pad_h],
                    [widget_w - self.cell_pad_w * 2.0,
                     widget_h - self.cell_pad_h * 2.0],
                );
                widget_num += 1u;
            }
        }
    }

    /// A builder method for adding padding to the cell.
    pub fn cell_padding(self, w: f64, h: f64) -> WidgetMatrixContext<'a> {
        WidgetMatrixContext { cell_pad_w: w, cell_pad_h: h, ..self }
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
    row: uint,
    col: uint,
    rows: uint,
    cols: uint,
}

impl Iterator for CellIterator {
    fn next
    */

pub trait WidgetMatrixBuilder<'a> {
    /// A widget matrix builder method to be implemented by the UiContext.
    fn widget_matrix(&'a mut self, cols: uint, rows: uint) -> WidgetMatrixContext<'a>;
}

impl<'a> WidgetMatrixBuilder<'a> for UiContext {

    /// Create a widget matrix context.
    fn widget_matrix(&'a mut self, cols: uint, rows: uint) -> WidgetMatrixContext<'a> {
        WidgetMatrixContext {
            uic: self,
            cols: cols,
            rows: rows,
            pos: [0.0, 0.0],
            dim: [256.0, 256.0],
            cell_pad_w: 0.0,
            cell_pad_h: 0.0,
        }
    }
}

impl_positionable!(WidgetMatrixContext);
impl_shapeable!(WidgetMatrixContext);
