
use point::Point;
use ui_context::UIContext;

/// Callback params.
pub type WidgetNum = uint;
pub type ColNum = uint;
pub type RowNum = uint;
pub type Width = f64;
pub type Height = f64;

/// Draw a matrix of any rectangular widget type, where the
/// matrix will provide a callback with the widget number,
/// it's `rows` and `cols` position, the width and height
/// for the widget and the location at which the widget
/// should be drawn.
pub struct WidgetMatrixContext<'a> {
    uic: &'a mut UIContext,
    cols: uint,
    rows: uint,
    pos: Point<f64>,
    width: f64,
    height: f64,
}

impl<'a> WidgetMatrixContext<'a> {
    /// The callback called for each widget in the matrix.
    /// This should be called following all builder methods.
    pub fn each_widget(&'a mut self, callback: |&mut UIContext, WidgetNum, ColNum, RowNum, Point<f64>, Width, Height|) {
        let widget_w = self.width / self.cols as f64;
        let widget_h = self.height / self.rows as f64;
        let mut widget_num = 0u;
        for col in range(0u, self.cols) {
            for row in range(0u, self.rows) {
                callback(self.uic,
                         widget_num,
                         col,
                         row,
                         Point::new(self.pos.x + (widget_w * col as f64),
                                    self.pos.y + (widget_h * row as f64),
                                    0.0),
                         widget_w,
                         widget_h);
                widget_num += 1u;
            }
        }
    }
}

pub trait WidgetMatrixBuilder<'a> {
    /// A widget matrix builder method to be implemented by the UIContext.
    fn widget_matrix(&'a mut self, cols: uint, rows: uint) -> WidgetMatrixContext<'a>;
}

impl<'a> WidgetMatrixBuilder<'a> for UIContext {

    /// Create a widget matrix context.
    fn widget_matrix(&'a mut self, cols: uint, rows: uint) -> WidgetMatrixContext<'a> {
        WidgetMatrixContext {
            uic: self,
            cols: cols,
            rows: rows,
            pos: Point::new(0.0, 0.0, 0.0),
            width: 256.0,
            height: 256.0,
        }
    }

}

impl_positionable!(WidgetMatrixContext)
impl_shapeable!(WidgetMatrixContext)

