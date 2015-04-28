
use color::Color;
use position::Direction;

/// Unique identifier for Canvasses.
pub type CanvasId = usize;

/// A tool for flexibly designing and guiding widget layout.
pub struct Canvas {
    id: CanvasId,
    maybe_kids: Option<(Direction, Vec<Canvas>)>,
    maybe_dim: Option<f64>,
    maybe_adjustable: Option<Bounds>,
    style: Style,
}

/// Describes the style of a Canvas.
pub struct Style {
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_color: Option<Color>,
    padding: Padding,
    margin: Margin,
}

/// The distance between the edge of a widget and the inner edge of a Canvas' frame.
pub struct Padding {
    maybe_top: Option<f64>,
    maybe_bottom: Option<f64>,
    maybe_left: Option<f64>,
    maybe_right: Option<f64>,
}

/// The distance between the edget of a Canvas' outer dimensions and the outer edge of its frame.
pub struct Margin {
    maybe_top: Option<f64>,
    maybe_bottom: Option<f64>,
    maybe_left: Option<f64>,
    maybe_right: Option<f64>,
}

/// The minimum and maximum for a dimension of a Canvas.
pub struct Bounds {
    min: f64,
    max: f64,
}

impl Canvas {

    /// Construct a default Canvas.
    pub fn new(id: CanvasId) -> Canvas {
        Canvas {
            id: id,
            maybe_kids: None,
            maybe_dim: None,
            maybe_adjustable: None,
            style: Style::new(),
        }
    }

    /// Construct an adjustable Canvas.
    pub fn adjustable(id: CanvasId, min: f64, max: f64) -> Canvas {
        Canvas { maybe_adjustable: Some(Bounds { min: min, max: max }), ..Canvas::new(id) }
    }

    /// Set the dimension of the Canvas.
    pub fn dim(mut self, dim: f64) -> Canvas {
        self.maybe_dim = Some(dim);
        self
    }
    
    /// Set the child Canvasses of the Canvas.
    pub fn kids(mut self, dir: Direction, kids: Vec<Canvas>) -> Canvas {
        self.maybe_kids = Some((dir, kids));
        self
    }
}


impl Style {

    /// Construct a default Style.
    pub fn new() -> Style {
        Style {
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_color: None,
            padding: Padding::new(),
            margin: Margin::new(),
        }
    }

}


impl Padding {

    /// Construct a defualt Padding.
    pub fn new() -> Padding {
        Padding {
            maybe_top: None,
            maybe_bottom: None,
            maybe_left: None,
            maybe_right: None,
        }
    }

}


impl Margin {

    /// Construct a defualt Margin.
    pub fn new() -> Margin {
        Margin {
            maybe_top: None,
            maybe_bottom: None,
            maybe_left: None,
            maybe_right: None,
        }
    }

}


// fn test() {
// 
//     const MASTER: CanvasId = 0;
//     const HEADER: CanvasId = 1;
//     const BODY: CanvasId = 2;
//     const LEFT_COLUMN: CanvasId = 3;
//     const MIDDLE_COLUMN: CanvasId = 4;
//     const RIGHT_COLUMN: CanvasId = 5;
//     const FOOTER: CanvasId = 6;
// 
//     let canvas = Canvas::new(MASTER).kids(Direction::Down, vec![
//         Canvas::adjustable(HEADER, 20.0, 50.0).dim(40.0),
//         Canvas::new(BODY).kids(Direction::Right, vec![
//             Canvas::new(LEFT_COLUMN),
//             Canvas::new(MIDDLE_COLUMN),
//             Canvas::new(RIGHT_COLUMN),
//         ]),
//         Canvas::new(FOOTER)
//     ]);
// 
// }
