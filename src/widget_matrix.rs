
use point::Point;

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
pub fn draw(cols: uint,
            rows: uint,
            pos: Point<f64>,
            width: f64,
            height: f64,
            widget_draw_callback: |WidgetNum, ColNum, RowNum, Point<f64>, Width, Height|) {
    let widget_w = width / cols as f64;
    let widget_h = height / rows as f64;
    let mut widget_num = 0u;
    for col in range(0u, cols) {
        for row in range(0u, rows) {
            widget_draw_callback(widget_num,
                                 col,
                                 row,
                                 Point::new(pos.x + (widget_w * col as f64),
                                            pos.y + (widget_h * row as f64),
                                            0.0),
                                 widget_w,
                                 widget_h);
            widget_num += 1u;
        }
    }
}

