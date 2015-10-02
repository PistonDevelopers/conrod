
use Scalar;
use color::Color;
use graphics::character::CharacterCache;
use position::{Dimensions, Direction, Point, Positionable, Sizeable};
use super::canvas;
use widget::{self, Widget};
use ui::Ui;


/// A type of Canvas for flexibly designing and guiding widget layout as splits of a window.
pub struct Split<'a> {
    id: widget::Id,
    style: canvas::Style,
    maybe_splits: Option<(Direction, &'a [Split<'a>])>,
    maybe_length: Option<Scalar>,
    // TODO: Maybe use the `CommonBuilder` (to be used for the wrapped `Canvas`) here instead?
    is_h_scrollable: bool,
    is_v_scrollable: bool,
}


impl<'a> Split<'a> {

    /// Construct a default Canvas Split.
    pub fn new(id: widget::Id) -> Split<'a> {
        Split {
            id: id,
            style: canvas::Style::new(),
            maybe_splits: None,
            maybe_length: None,
            is_h_scrollable: false,
            is_v_scrollable: false,
        }
    }

    /// Set the dimension of the Split.
    pub fn length(mut self, length: Scalar) -> Split<'a> {
        self.maybe_length = Some(length);
        self
    }

    /// Set the child Canvas Splits of the current Canvas flowing in a given direction.
    pub fn flow(mut self, dir: Direction, splits: &'a [Split<'a>]) -> Split<'a> {
        self.maybe_splits = Some((dir, splits));
        self
    }

    /// Set the child Canvasses flowing downwards.
    pub fn flow_down(self, splits: &'a [Split<'a>]) -> Split<'a> {
        self.flow(Direction::Down, splits)
    }

    /// Set the child Canvasses flowing upwards.
    pub fn flow_up(self, splits: &'a [Split<'a>]) -> Split<'a> {
        self.flow(Direction::Up, splits)
    }

    /// Set the child Canvasses flowing to the right.
    pub fn flow_right(self, splits: &'a [Split<'a>]) -> Split<'a> {
        self.flow(Direction::Right, splits)
    }

    /// Set the child Canvasses flowing to the left.
    pub fn flow_left(self, splits: &'a [Split<'a>]) -> Split<'a> {
        self.flow(Direction::Left, splits)
    }

    /// Set the padding from the left edge.
    pub fn pad_left(mut self, pad: Scalar) -> Split<'a> {
        self.style.padding.maybe_left = Some(pad);
        self
    }

    /// Set the padding from the right edge.
    pub fn pad_right(mut self, pad: Scalar) -> Split<'a> {
        self.style.padding.maybe_right = Some(pad);
        self
    }

    /// Set the padding from the top edge.
    pub fn pad_top(mut self, pad: Scalar) -> Split<'a> {
        self.style.padding.maybe_top = Some(pad);
        self
    }

    /// Set the padding from the bottom edge.
    pub fn pad_bottom(mut self, pad: Scalar) -> Split<'a> {
        self.style.padding.maybe_bottom = Some(pad);
        self
    }

    /// Set the padding for all edges.
    pub fn pad(self, pad: Scalar) -> Split<'a> {
        self.pad_left(pad).pad_right(pad).pad_top(pad).pad_bottom(pad)
    }

    /// Set the margin from the left edge.
    pub fn margin_left(mut self, mgn: Scalar) -> Split<'a> {
        self.style.margin.maybe_left = Some(mgn);
        self
    }

    /// Set the margin from the right edge.
    pub fn margin_right(mut self, mgn: Scalar) -> Split<'a> {
        self.style.margin.maybe_right = Some(mgn);
        self
    }

    /// Set the margin from the top edge.
    pub fn margin_top(mut self, mgn: Scalar) -> Split<'a> {
        self.style.margin.maybe_top = Some(mgn);
        self
    }

    /// Set the margin from the bottom edge.
    pub fn margin_bottom(mut self, mgn: Scalar) -> Split<'a> {
        self.style.margin.maybe_bottom = Some(mgn);
        self
    }

    /// Set the margin for all edges.
    pub fn margin(self, mgn: Scalar) -> Split<'a> {
        self.margin_left(mgn).margin_right(mgn).margin_top(mgn).margin_bottom(mgn)
    }

    /// Set whether or not the Canvas' `KidArea` is scrollable (the default is false).
    /// If a widget is scrollable and it has children widgets that fall outside of its `KidArea`,
    /// the `KidArea` will become scrollable.
    pub fn scrolling(mut self, scrollable: bool) -> Self {
        self.is_v_scrollable = scrollable;
        self.is_h_scrollable = scrollable;
        self
    }

    /// Same as `Split::scrolling`, however only activates vertical scrolling.
    pub fn vertical_scrolling(mut self, scrollable: bool) -> Self {
        self.is_v_scrollable = scrollable;
        self
    }

    /// Same as `Split::scrolling`, however only activates horizontal scrolling.
    pub fn horizontal_scrolling(mut self, scrollable: bool) -> Self {
        self.is_h_scrollable = scrollable;
        self
    }

    /// Store the Canvas and its children within the `Ui`.
    pub fn set<C>(self, ui: &mut Ui<C>) where C: CharacterCache {
        let dim = [ui.win_w as f64, ui.win_h as f64];
        self.into_ui(dim, [0.0, 0.0], None, ui);
    }

    /// Construct a Canvas from a Split.
    fn into_ui<C>(&self,
                  dim: Dimensions,
                  xy: Point,
                  maybe_parent: Option<widget::Id>,
                  ui: &mut Ui<C>)
        where C: CharacterCache
    {
        use vecmath::{vec2_add, vec2_sub};

        let Split {
            id,
            ref style,
            ref maybe_splits,
            is_v_scrollable,
            is_h_scrollable,
            ..
        } = *self;

        let frame = style.frame(&ui.theme);
        let pad = style.padding(&ui.theme);
        let mgn = style.margin(&ui.theme);

        let mgn_offset = [(mgn.left - mgn.right), (mgn.bottom - mgn.top)];
        let dim = vec2_sub(dim, [mgn.left + mgn.right, mgn.top + mgn.bottom]);
        let frame_dim = vec2_sub(dim, [frame * 2.0; 2]);
        let pad_offset = [(pad.bottom - pad.top), (pad.left - pad.right)];
        let pad_dim = vec2_sub(frame_dim, [pad.left + pad.right, pad.top + pad.bottom]);

        // Offset xy so that it is in the center of the given margin.
        let xy = vec2_add(xy, mgn_offset);

        if let Some((direction, splits)) = *maybe_splits {
            use Direction::{Up, Down, Left, Right};

            // Offset xy so that it is in the center of the padded area.
            let xy = vec2_add(xy, pad_offset);
            let (stuck_length, num_not_stuck) =
                splits.iter().fold((0.0, splits.len()), |(total, remaining), split| {
                    match split.maybe_length {
                        Some(length) => (total + length, remaining - 1),
                        None => (total, remaining),
                    }
                });

            // Dimensions for Splits that haven't been given a specific length.
            let split_dim = match num_not_stuck {
                0 => [0.0, 0.0],
                _ => match direction {
                    Up   | Down  => {
                        let remaining_height = pad_dim[1] - stuck_length;
                        let height = match remaining_height > 0.0 {
                            true  => remaining_height / num_not_stuck as f64,
                            false => 0.0,
                        };
                        [pad_dim[0], height]
                    },
                    Left | Right => {
                        let remaining_width = pad_dim[0] - stuck_length;
                        let width = match remaining_width > 0.0 {
                            true  => remaining_width / num_not_stuck as f64,
                            false => 0.0
                        };
                        [width, pad_dim[1]]
                    },
                },
            };

            // The length of the previous split.
            let mut prev_length = 0.0;

            // Initialise the `current_xy` at the beginning of the pad_dim.
            let mut current_xy = match direction {
                Down  => [xy[0], xy[1] + pad_dim[1] / 2.0],
                Up    => [xy[0], xy[1] - pad_dim[1] / 2.0],
                Left  => [xy[0] + pad_dim[0] / 2.0, xy[1]],
                Right => [xy[0] - pad_dim[0] / 2.0, xy[1]],
            };

            // Update every split within the Ui.
            for split in splits.iter() {
                let split_dim = match split.maybe_length {
                    Some(len) => match direction {
                        Up   | Down  => [split_dim[0], len],
                        Left | Right => [len, split_dim[1]],
                    },
                    None => split_dim,
                };

                // Shift xy into position for the current split.
                match direction {
                    Down => {
                        current_xy[1] -= split_dim[1] / 2.0 + prev_length / 2.0;
                        prev_length = split_dim[1];
                    },
                    Up   => {
                        current_xy[1] += split_dim[1] / 2.0 + prev_length / 2.0;
                        prev_length = split_dim[1];
                    },
                    Left => {
                        current_xy[0] -= split_dim[0] / 2.0 + prev_length / 2.0;
                        prev_length = split_dim[0];
                    },
                    Right => {
                        current_xy[0] += split_dim[0] / 2.0 + prev_length / 2.0;
                        prev_length = split_dim[0];
                    },
                }

                split.into_ui(split_dim, current_xy, Some(id), ui);
            }
        }

        let mut canvas = canvas::Canvas::new();
        canvas.style = style.clone();
        match maybe_parent {
            Some(parent_id) => canvas.parent(Some(parent_id)),
            None            => canvas.parent(None::<widget::Id>),
        }.point(xy)
            .dim(dim)
            .vertical_scrolling(is_v_scrollable)
            .horizontal_scrolling(is_h_scrollable)
            .set(id, ui);
    }

}


impl<'a> ::color::Colorable for Split<'a> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

impl<'a> ::frame::Frameable for Split<'a> {
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
        self
    }
}


