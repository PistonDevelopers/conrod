
use ::{CharacterCache, GlyphCache, NodeIndex, Scalar, Theme};
use widget::{self, Widget};


/// Reaction params.
pub type WidgetNum = usize;
pub type ColNum = usize;
pub type RowNum = usize;
pub type Width = Scalar;
pub type Height = Scalar;
pub type PosX = Scalar;
pub type PosY = Scalar;

/// Draw a matrix of any rectangular widget type, where the matrix will provide a function with
/// the widget number, it's `rows` and `cols` position, the width and height for the widget and
/// the location at which the widget should be drawn.
#[derive(Copy, Clone)]
pub struct Matrix<F> {
    common: widget::CommonBuilder,
    style: Style,
    cols: usize,
    rows: usize,
    maybe_each_widget: Option<F>,
}

/// The state of the Matrix, to be cached within the `Ui`'s widget `Graph`.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// A `NodeIndex` for every Widget in the Matrix.
    /// This matrix is column major, meaning the outer-most Vec represents each column, and each
    /// inner Vec represents a row.
    indices: Vec<Vec<NodeIndex>>,
}

/// Unique styling for the `Matrix`.
#[derive(Copy, Clone, Debug, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Style {
    /// The width of the padding for each matrix element's "cell".
    maybe_cell_pad_w: Option<Scalar>,
    /// The height of the padding for each matrix element's "cell".
    maybe_cell_pad_h: Option<Scalar>,
}


impl<F> Matrix<F> {

    /// Create a widget matrix context.
    pub fn new(cols: usize, rows: usize) -> Matrix<F> {
        Matrix {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
            cols: cols,
            rows: rows,
            maybe_each_widget: None,
        }
    }

    /// The function that will be called for each and every element in the Matrix.
    /// The function should return the widget that will be displayed in the element associated with
    /// the given row and column number.
    /// Note that the returned Widget's position and dimensions will be overridden with the
    /// dimensions and position of the matrix element's rectangle.
    pub fn each_widget(mut self, each_widget: F) -> Matrix<F> {
        self.maybe_each_widget = Some(each_widget);
        self
    }

    /// A builder method for adding padding to the cell.
    pub fn cell_padding(mut self, w: Scalar, h: Scalar) -> Matrix<F> {
        self.style.maybe_cell_pad_w = Some(w);
        self.style.maybe_cell_pad_h = Some(h);
        self
    }

}


impl<'a, F, W> Widget for Matrix<F> where
    W: Widget,
    F: FnMut(WidgetNum, ColNum, RowNum) -> W
{
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder { &self.common }
    fn common_mut(&mut self) -> &mut widget::CommonBuilder { &mut self.common }
    fn unique_kind(&self) -> &'static str { "Matrix" }
    fn init_state(&self) -> State {
        State { indices: Vec::new() }
    }
    fn style(&self) -> Style { self.style.clone() }

    fn default_width<C: CharacterCache>(&self, theme: &Theme, _: &GlyphCache<C>) -> Scalar {
        const DEFAULT_WIDTH: Scalar = 128.0;
        theme.maybe_matrix.as_ref().map(|default| {
            default.common.maybe_width.unwrap_or(DEFAULT_WIDTH)
        }).unwrap_or(DEFAULT_WIDTH)
    }

    fn default_height(&self, theme: &Theme) -> Scalar {
        const DEFAULT_HEIGHT: Scalar = 64.0;
        theme.maybe_matrix.as_ref().map(|default| {
            default.common.maybe_height.unwrap_or(DEFAULT_HEIGHT)
        }).unwrap_or(DEFAULT_HEIGHT)
    }

    /// Update the state of the Matrix.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let Matrix { cols, rows, maybe_each_widget, .. } = self;

        // First, check that we have the correct number of columns and rows.
        let num_cols = state.view().indices.len();
        let num_rows = state.view().indices.get(0).map(|col| col.len()).unwrap_or(0);
        let maybe_new_indices = if num_cols < cols || num_rows < rows {
            let mut total_cols: Vec<_> = state.view().indices.iter()
                .map(|col| col.clone())
                .chain((num_cols..cols).map(|_| Vec::with_capacity(rows)))
                .collect();
            for col in total_cols.iter_mut() {
                let rows_in_col = col.len();
                if rows_in_col < rows {
                    col.extend((rows_in_col..rows).map(|_| ui.new_unique_node_index()));
                }
            }
            Some(total_cols)
        } else {
            None
        };

        // A function to simplify getting the current slice of indices.
        fn get_indices<'a>(maybe_new: &'a Option<Vec<Vec<NodeIndex>>>,
                           state: &'a widget::State<State>) -> &'a [Vec<NodeIndex>] {
            maybe_new.as_ref().map(|is| &is[..]).unwrap_or_else(|| &state.view().indices[..])
        }

        // We only need to worry about element calculations if we actually have rows and columns.
        if rows > 0 && cols > 0 {
            // Likewise, there must also be some function to give us the widgets.
            if let Some(mut each_widget) = maybe_each_widget {

                let cell_pad_w = style.cell_pad_w(ui.theme());
                let cell_pad_h = style.cell_pad_h(ui.theme());
                let (w, h) = rect.w_h();
                let widget_w = w / cols as Scalar;
                let widget_h = h / rows as Scalar;
                let (half_w, half_h) = (w / 2.0, h / 2.0);
                let x_min = -half_w + widget_w / 2.0;
                let x_max = half_w + widget_w / 2.0;
                let y_min = -half_h - widget_h / 2.0;
                let y_max = half_h - widget_h / 2.0;

                let mut widget_num = 0;
                let indices = get_indices(&maybe_new_indices, state);
                for col in 0..cols {
                    for row in 0..rows {
                        use position::{Positionable, Sizeable};
                        use utils::map_range;
                        let rel_x = map_range(col as Scalar, 0.0, cols as Scalar, x_min, x_max);
                        let rel_y = map_range(row as Scalar, 0.0, rows as Scalar, y_max, y_min);
                        let w = widget_w - cell_pad_w * 2.0;
                        let h = widget_h - cell_pad_h * 2.0;
                        let widget_idx = indices[col][row];
                        each_widget(widget_num, col, row)
                            .dim([w, h])
                            .relative_to(idx, [rel_x, rel_y])
                            .set(widget_idx, &mut ui);
                        widget_num += 1;
                    }
                }
            }
        }

        if let Some(new_indices) = maybe_new_indices {
            state.update(|state| state.indices = new_indices);
        }
    }

    /// Construct an Element from the given DropDownList State.
    fn draw<C: CharacterCache>(_args: widget::DrawArgs<Self, C>) -> ::Element {
        // We don't need to draw anything, as DropDownList is entirely composed of other widgets.
        ::elmesque::element::empty()
    }

}


impl Style {

    /// Constructor for a new default Matrix Style.
    pub fn new() -> Style {
        Style {
            maybe_cell_pad_w: None,
            maybe_cell_pad_h: None,
        }
    }

    /// Get the width of the padding for each matrix element's cell.
    pub fn cell_pad_w(&self, theme: &Theme) -> Scalar {
        const DEFAULT_CELL_PAD_W: Scalar = 0.0;
        self.maybe_cell_pad_w.or(theme.maybe_matrix.as_ref().map(|default| {
            default.style.maybe_cell_pad_w.unwrap_or(DEFAULT_CELL_PAD_W)
        })).unwrap_or(DEFAULT_CELL_PAD_W)
    }

    /// Get the height of the padding for each matrix element's cell.
    pub fn cell_pad_h(&self, theme: &Theme) -> Scalar {
        const DEFAULT_CELL_PAD_H: Scalar = 0.0;
        self.maybe_cell_pad_h.or(theme.maybe_matrix.as_ref().map(|default| {
            default.style.maybe_cell_pad_h.unwrap_or(DEFAULT_CELL_PAD_H)
        })).unwrap_or(DEFAULT_CELL_PAD_H)
    }

}

