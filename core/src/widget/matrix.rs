//! A helper widget for laying out child widgets in the form of a grid.

use {Scalar, Ui, UiCell, Widget};
use graph;
use utils;
use widget;


/// The number of the widget.
pub type WidgetNum = usize;
/// A column index.
pub type ColNum = usize;
/// A row index.
pub type RowNum = usize;
/// The width of an element.
pub type Width = Scalar;
/// The height of an element.
pub type Height = Scalar;
/// The position of an element along the *x* axis.
pub type PosX = Scalar;
/// The position of an element along the *y* axis.
pub type PosY = Scalar;

/// Draw a matrix of any rectangular widget type, where the matrix will provide a function with
/// the widget number, it's `rows` and `cols` position, the width and height for the widget and
/// the location at which the widget should be drawn.
#[derive(Clone, WidgetCommon_)]
#[allow(missing_copy_implementations)]
pub struct Matrix {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    style: Style,
    cols: usize,
    rows: usize,
}

/// The state of the Matrix, to be cached within the `Ui`'s widget `Graph`.
pub struct State {
    /// A `widget::Id` for every Widget in the Matrix.
    /// This matrix is column major, meaning the outer-most Vec represents each column, and each
    /// inner Vec represents a row.
    indices: Vec<Vec<widget::Id>>,
}

/// Unique styling for the `Matrix`.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// The width of the padding for each matrix element's "cell".
    #[conrod(default = "0.0")]
    pub cell_pad_w: Option<Scalar>,
    /// The height of the padding for each matrix element's "cell".
    #[conrod(default = "0.0")]
    pub cell_pad_h: Option<Scalar>,
}

/// The event type yielded by the `Matrix`.
///
/// This can be used to iterate over each element in the `Matrix`.
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct Elements {
    num_rows: usize,
    num_cols: usize,
    row: usize,
    col: usize,
    matrix_id: widget::Id,
    elem_w: Scalar,
    elem_h: Scalar,
    x_min: Scalar, x_max: Scalar,
    y_min: Scalar, y_max: Scalar,
}

/// Data necessary for instantiating a widget for a single `Matrix` element.
#[derive(Copy, Clone, Debug)]
pub struct Element {
    /// The id generated for the widget.
    pub widget_id: widget::Id,
    /// The row number for the `Element`.
    pub row: usize,
    /// The column number for the `Element`.
    pub col: usize,
    /// The width of the element.
    pub w: Scalar,
    /// The height of the element.
    pub h: Scalar,
    /// The *x* position of the element relative to the centre of the `Matrix`.
    pub rel_x: Scalar,
    /// The *y* position of the element relative to the centre of the `Matrix`.
    pub rel_y: Scalar,
    /// The id of the `Matrix`, used for positioning.
    matrix_id: widget::Id,
}


impl Matrix {

    /// Create a widget matrix context.
    pub fn new(cols: usize, rows: usize) -> Self {
        Matrix {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            cols: cols,
            rows: rows,
        }
    }

    /// A builder method for adding padding to the cell.
    pub fn cell_padding(mut self, w: Scalar, h: Scalar) -> Self {
        self.style.cell_pad_w = Some(w);
        self.style.cell_pad_h = Some(h);
        self
    }

}


impl Widget for Matrix {
    type State = State;
    type Style = Style;
    type Event = Elements;

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State { indices: Vec::new() }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the Matrix.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, rect, style, ui, .. } = args;
        let Matrix { cols, rows, .. } = self;

        // First, check that we have the correct number of columns.
        let num_cols = state.indices.len();
        if num_cols < cols {
            state.update(|state| {
                state.indices.extend((num_cols..cols).map(|_| Vec::with_capacity(rows)));
            });
        }

        // Now check that we have the correct amount of rows in each column.
        for col in 0..cols {
            let num_rows = state.indices[col].len();
            if num_rows < rows {
                let mut id_gen = ui.widget_id_generator();
                state.update(|state| {
                    let extension = (num_rows..rows).map(|_| id_gen.next());
                    state.indices[col].extend(extension);
                });
            }
        }

        let cell_pad_w = style.cell_pad_w(&ui.theme);
        let cell_pad_h = style.cell_pad_h(&ui.theme);
        let (w, h) = rect.w_h();
        let elem_w = w / cols as Scalar;
        let elem_h = h / rows as Scalar;
        let (half_w, half_h) = (w / 2.0, h / 2.0);
        let x_min = -half_w + elem_w / 2.0;
        let x_max = half_w + elem_w / 2.0;
        let y_min = -half_h - elem_h / 2.0;
        let y_max = half_h - elem_h / 2.0;

        let elements = Elements {
            num_rows: rows,
            num_cols: cols,
            row: 0,
            col: 0,
            matrix_id: id,
            elem_w: elem_w - cell_pad_w * 2.0,
            elem_h: elem_h - cell_pad_h * 2.0,
            x_min: x_min,
            x_max: x_max,
            y_min: y_min,
            y_max: y_max,
        };

        elements
    }

}


impl Elements {

    /// Yield the next `Element`.
    pub fn next(&mut self, ui: &Ui) -> Option<Element> {
        let Elements {
            ref mut row,
            ref mut col,
            num_rows,
            num_cols,
            matrix_id,
            elem_w,
            elem_h,
            x_min, x_max,
            y_min, y_max,
        } = *self;

        let (r, c) = (*row, *col);

        // Retrieve the `widget::Id` that was generated for the next `Element`.
        let widget_id = match ui.widget_graph().widget(matrix_id)
            .and_then(|container| container.unique_widget_state::<Matrix>())
            .and_then(|&graph::UniqueWidgetState { ref state, .. }| {
                state.indices.get(c).and_then(|col| col.get(r).map(|&id| id))
            })
        {
            Some(id) => id,
            None => return None,
        };

        // Increment the elem indices.
        *row += 1;
        if *row >= num_rows {
            *row = 0;
            *col += 1;
        }

        let rel_x = utils::map_range(c as Scalar, 0.0, num_cols as Scalar, x_min, x_max);
        let rel_y = utils::map_range(r as Scalar, 0.0, num_rows as Scalar, y_max, y_min);

        Some(Element {
            widget_id: widget_id,
            matrix_id: matrix_id,
            col: c,
            row: r,
            w: elem_w,
            h: elem_h,
            rel_x: rel_x,
            rel_y: rel_y,
        })
    }

}


impl Element {

    /// Sets the given widget as the widget to use for the item.
    ///
    /// Sets the:
    /// - position of the widget.
    /// - dimensions of the widget.
    /// - parent of the widget.
    /// - and finally sets the widget within the `Ui`.
    pub fn set<W>(self, widget: W, ui: &mut UiCell) -> W::Event
        where W: Widget,
    {
        use {Positionable, Sizeable};
        let Element { widget_id, matrix_id, w, h, rel_x, rel_y, .. } = self;
        widget
            .w_h(w, h)
            .x_y_relative_to(matrix_id, rel_x, rel_y)
            .set(widget_id, ui)
    }

}
