use {
    CharacterCache,
    Color,
    Colorable,
    Dimension,
    Dimensions,
    FontSize,
    Frameable,
    FramedRectangle,
    FramedRectangleStyle,
    IndexSlot,
    Labelable,
    Padding,
    Place,
    Position,
    Positionable,
    Range,
    Rect,
    Scalar,
    Sizeable,
    TextStyle,
    Theme,
    TitleBar,
    Ui,
    UiCell,
    Widget,
};
use position;
use position::Direction::{Forwards, Backwards};
use widget::{self, title_bar};


/// **Canvas** is designed to be a "container"-like "parent" widget that simplifies placement of
/// "children" widgets.
///
/// Widgets can be placed on a **Canvas** in a variety of ways using methods from the
/// [**Positionable**](../position/trait.Positionable) trait.
///
/// **Canvas** provides methods for padding the kid widget area which can make using the
/// **Place**-related **Positionable** methods a little easier.
///
/// A **Canvas** can also be divided into a sequence of smaller **Canvas**ses using the `.flow_*`
/// methods. This creates a kind of **Canvas** tree, where each "split" can be sized using the
/// `.length` or `.length_weight` methods.
///
/// See the `canvas.rs` example for a demonstration of the **Canvas** type.
#[derive(Copy, Clone, Debug)]
pub struct Canvas<'a> {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// The builder data related to the style of the Canvas.
    pub style: Style,
    /// The label for the **Canvas**' **TitleBar** if there is one.
    pub maybe_title_bar_label: Option<&'a str>, 
    /// A list of child **Canvas**ses as splits of this **Canvas** flowing in the given direction.
    pub maybe_splits: Option<FlowOfSplits<'a>>,
}

/// **Canvas** state to be cached.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    rectangle_idx: IndexSlot,
    title_bar_idx: IndexSlot,
}

/// Describes the style of a Canvas.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    /// Styling for the Canvas' rectangle.
    pub framed_rectangle: FramedRectangleStyle,
    /// The label and styling for the Canvas' title bar if it has one.
    pub text: TextStyle,
    /// Padding of the kid area.
    pub padding: PaddingBuilder,
    /// If this **Canvas** is a split of some parent **Canvas**, this is the length of the split.
    pub maybe_length: Option<Length>,
}

/// Unique kind for the widget type.
pub const KIND: widget::Kind = "Canvas";

/// A series of **Canvas** splits along with their unique identifiers.
pub type ListOfSplits<'a> = &'a [(widget::Id, Canvas<'a>)];

/// A series of **Canvas** splits flowing in the specified direction.
pub type FlowOfSplits<'a> = (Direction, ListOfSplits<'a>);

/// A builder for the padding of the area where child widgets will be placed.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PaddingBuilder {
    /// The padding for the left of the area where child widgets will be placed.
    pub maybe_left: Option<Scalar>,
    /// The padding for the right of the area where child widgets will be placed.
    pub maybe_right: Option<Scalar>,
    /// The padding for the top of the area where child widgets will be placed.
    pub maybe_top: Option<Scalar>,
    /// The padding for the bottom of the area where child widgets will be placed.
    pub maybe_bottom: Option<Scalar>,
}

/// The length of a `Split` given as a weight.
///
/// The length is determined by determining what percentage each `Split`'s weight contributes to
/// the total weight of all `Split`s in a flow list.
pub type Weight = Scalar;

/// Used to describe the desired length for a `Split`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Length {
    /// The length as an absolute scalar.
    Absolute(Scalar),
    /// The length as a weight of the non-absolute length of the parent **Canvas**.
    Weight(Weight),
}

/// The direction in which a sequence of canvas splits will be laid out.
#[derive(Copy, Clone, Debug)]
enum Direction {
    X(position::Direction),
    Y(position::Direction),
}


impl<'a> Canvas<'a> {

    /// Construct a new Canvas builder.
    pub fn new() -> Canvas<'a> {
        Canvas {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
            maybe_title_bar_label: None,
            maybe_splits: None,
        }
    }

    /// Show or hide the title bar.
    pub fn title_bar(mut self, label: &'a str) -> Self {
        self.maybe_title_bar_label = Some(label);
        self
    }

    /// Set the length of the Split as an absolute scalar.
    pub fn length(mut self, length: Scalar) -> Self {
        self.style.maybe_length = Some(Length::Absolute(length));
        self
    }

    /// Set the length of the Split as a weight.
    ///
    /// The default length weight for each widget is `1.0`.
    pub fn length_weight(mut self, weight: Weight) -> Self {
        self.style.maybe_length = Some(Length::Weight(weight));
        self
    }

    /// Set the child Canvas Splits of the current Canvas flowing in a given direction.
    fn flow(mut self, direction: Direction, splits: ListOfSplits<'a>) -> Self {
        self.maybe_splits = Some((direction, splits));
        self
    }

    /// Set the child Canvasses flowing to the right.
    pub fn flow_right(self, splits: ListOfSplits<'a>) -> Self {
        self.flow(Direction::X(Forwards), splits)
    }

    /// Set the child Canvasses flowing to the left.
    pub fn flow_left(self, splits: ListOfSplits<'a>) -> Self {
        self.flow(Direction::Y(Backwards), splits)
    }

    /// Set the child Canvasses flowing upwards.
    pub fn flow_up(self, splits: ListOfSplits<'a>) -> Self {
        self.flow(Direction::Y(Forwards), splits)
    }

    /// Set the child Canvasses flowing downwards.
    pub fn flow_down(self, splits: ListOfSplits<'a>) -> Self {
        self.flow(Direction::Y(Backwards), splits)
    }

    /// Set the padding of the left of the area where child widgets will be placed.
    #[inline]
    pub fn pad_left(mut self, pad: Scalar) -> Self {
        self.style.padding.maybe_left = Some(pad);
        self
    }

    /// Set the padding of the right of the area where child widgets will be placed.
    #[inline]
    pub fn pad_right(mut self, pad: Scalar) -> Self {
        self.style.padding.maybe_right = Some(pad);
        self
    }

    /// Set the padding of the bottom of the area where child widgets will be placed.
    #[inline]
    pub fn pad_bottom(mut self, pad: Scalar) -> Self {
        self.style.padding.maybe_bottom = Some(pad);
        self
    }

    /// Set the padding of the top of the area where child widgets will be placed.
    #[inline]
    pub fn pad_top(mut self, pad: Scalar) -> Self {
        self.style.padding.maybe_top = Some(pad);
        self
    }

    /// Set the padding for all edges of the area where child widgets will be placed.
    #[inline]
    pub fn pad(self, pad: Scalar) -> Self {
        self.pad_left(pad).pad_right(pad).pad_bottom(pad).pad_top(pad)
    }

    /// Set the padding of the area where child widgets will be placed.
    #[inline]
    pub fn padding(self, pad: Padding) -> Self {
        self.pad_left(pad.x.start)
            .pad_right(pad.x.end)
            .pad_bottom(pad.y.start)
            .pad_top(pad.y.end)
    }

    /// Build the **Canvas** with the given **Style**.
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

}


impl<'a> Widget for Canvas<'a> {
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> widget::Kind {
        KIND
    }

    fn init_state(&self) -> State {
        State {
            rectangle_idx: IndexSlot::new(),
            title_bar_idx: IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn default_x_position<C: CharacterCache>(&self, _ui: &Ui<C>) -> Position {
        Position::Place(Place::Middle, None)
    }

    fn default_y_position<C: CharacterCache>(&self, _ui: &Ui<C>) -> Position {
        Position::Place(Place::Middle, None)
    }

    fn default_x_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        widget::default_x_dimension(self, ui)
            .or_else(|| ui.w_of(ui.window).map(|w| Dimension::Absolute(w)))
            .expect("`Ui.window` should always have some width")
    }

    fn default_y_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        widget::default_y_dimension(self, ui)
            .or_else(|| ui.h_of(ui.window).map(|h| Dimension::Absolute(h)))
            .expect("`Ui.window` should always have some height")
    }

    /// The title bar area at which the Canvas can be clicked and dragged.
    ///
    /// Note: the position of the returned **Rect** should be relative to the center of the widget.
    fn drag_area(&self, dim: Dimensions, style: &Style, theme: &Theme) -> Option<Rect> {
        self.maybe_title_bar_label.map(|_| {
            let font_size = style.text.font_size(theme);
            let (h, rel_y) = title_bar_h_rel_y(dim[1], font_size);
            let rel_xy = [0.0, rel_y];
            let dim = [dim[0], h];
            Rect::from_xy_dim(rel_xy, dim)
        })
    }

    /// The area of the widget below the title bar, upon which child widgets will be placed.
    fn kid_area<C: CharacterCache>(&self, args: widget::KidAreaArgs<Self, C>) -> widget::KidArea {
        let widget::KidAreaArgs { rect, style, theme, .. } = args;
        if self.maybe_title_bar_label.is_some() {
            let font_size = style.text.font_size(theme);
            let title_bar = title_bar(rect, font_size);
            widget::KidArea {
                rect: rect.pad_top(title_bar.h()),
                pad: style.padding(theme),
            }
        } else {
            widget::KidArea {
                rect: rect,
                pad: style.padding(theme),
            }
        }
    }

    /// Update the state of the Canvas.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { idx, state, rect, mut ui, .. } = args;
        let Canvas { style, maybe_title_bar_label, maybe_splits, .. } = self;

        // FramedRectangle widget as the rectangle backdrop.
        let rectangle_idx = state.view().rectangle_idx.get(&mut ui);
        let dim = rect.dim();
        let color = style.color(ui.theme());
        let frame = style.frame(ui.theme());
        let frame_color = style.frame_color(ui.theme());
        FramedRectangle::new(dim)
            .color(color)
            .frame(frame)
            .frame_color(frame_color)
            .middle_of(idx)
            .graphics_for(idx)
            .place_on_kid_area(false)
            .set(rectangle_idx, &mut ui);

        // TitleBar widget if we were given some label.
        if let Some(label) = maybe_title_bar_label {
            let title_bar_idx = state.view().title_bar_idx.get(&mut ui);
            let font_size = style.title_bar_font_size(ui.theme());
            let label_color = style.title_bar_label_color(ui.theme());
            TitleBar::new(label, rectangle_idx)
                .color(color)
                .frame(frame)
                .frame_color(frame_color)
                .label_font_size(font_size)
                .label_color(label_color)
                .graphics_for(idx)
                .place_on_kid_area(false)
                .react(|_interaction| ())
                .set(title_bar_idx, &mut ui);
        }

        // If we were given some child canvas splits, we should instantiate them.
        if let Some((direction, splits)) = maybe_splits {

            let (total_abs, total_weight) =
                splits.iter().fold((0.0, 0.0), |(abs, weight), &(_, split)| {
                    match split.style.length(ui.theme()) {
                        Length::Absolute(a) => (abs + a, weight),
                        Length::Weight(w) => (abs, weight + w),
                    }
                });

            // No need to calculate kid_area again, we'll just get it from the graph.
            let kid_area = ui.kid_area_of(idx).expect("No KidArea found");
            let kid_area_range = match direction {
                Direction::X(_) => kid_area.x,
                Direction::Y(_) => kid_area.y,
            };

            let total_length = kid_area_range.len();
            let non_abs_length = (total_length - total_abs).max(0.0);
            let weight_normaliser = 1.0 / total_weight;

            let length = |split: &Self, ui: &UiCell<C>| -> Scalar {
                match split.style.length(ui.theme()) {
                    Length::Absolute(length) => length,
                    Length::Weight(weight) => weight * weight_normaliser * non_abs_length,
                }
            };

            let set_split = |split_id: widget::Id, split: Canvas<'a>, ui: &mut UiCell<C>| {
                split.parent(idx).set(split_id, ui);
            };

            // Instantiate each of the splits, matching on the direction first for efficiency.
            match direction {

                Direction::X(direction) => match direction {
                    Forwards => for (i, &(split_id, split)) in splits.iter().enumerate() {
                        let w = length(&split, &ui);
                        let split = match i {
                            0 => split.h(kid_area.h()).mid_left_of(idx),
                            _ => split.right(0.0),
                        }.w(w);
                        set_split(split_id, split, &mut ui);
                    },
                    Backwards => for (i, &(split_id, split)) in splits.iter().enumerate() {
                        let w = length(&split, &ui);
                        let split = match i {
                            0 => split.h(kid_area.h()).mid_right_of(idx),
                            _ => split.left(0.0),
                        }.w(w);
                        set_split(split_id, split, &mut ui);
                    },
                },

                Direction::Y(direction) => match direction {
                    Forwards => for (i, &(split_id, split)) in splits.iter().enumerate() {
                        let h = length(&split, &ui);
                        let split = match i {
                            0 => split.w(kid_area.w()).mid_bottom_of(idx),
                            _ => split.up(0.0),
                        }.h(h);
                        set_split(split_id, split, &mut ui);
                    },
                    Backwards => for (i, &(split_id, split)) in splits.iter().enumerate() {
                        let h = length(&split, &ui);
                        let split = match i {
                            0 => split.w(kid_area.w()).mid_top_of(idx),
                            _ => split.down(0.0),
                        }.h(h);
                        set_split(split_id, split, &mut ui);
                    },
                },
            }

        }
    }

}


/// The height and relative y coordinate of a Canvas' title bar given some canvas height and font
/// size for the title bar.
fn title_bar_h_rel_y(canvas_h: Scalar, font_size: FontSize) -> (Scalar, Scalar) {
    let h = title_bar::calc_height(font_size);
    let rel_y = canvas_h / 2.0 - h / 2.0;
    (h, rel_y)
}

/// The Rect for the Canvas' title bar.
fn title_bar(canvas: Rect, font_size: FontSize) -> Rect {
    let (c_w, c_h) = canvas.w_h();
    let (h, rel_y) = title_bar_h_rel_y(c_h, font_size);
    let xy = [0.0, rel_y];
    let dim = [c_w, h];
    Rect::from_xy_dim(xy, dim)
}


impl Style {

    /// Construct a default Canvas Style.
    pub fn new() -> Style {
        Style {
            framed_rectangle: FramedRectangleStyle::new(),
            text: TextStyle::new(),
            padding: PaddingBuilder {
                maybe_left: None,
                maybe_right: None,
                maybe_top: None,
                maybe_bottom: None,
            },
            maybe_length: None,
        }
    }

    /// Get the color for the Canvas' Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.framed_rectangle.maybe_color
            .or_else(|| theme.widget_style::<Style>(KIND).map(|default| {
                default.style.framed_rectangle.maybe_color.unwrap_or(theme.background_color)
            }))
            .unwrap_or(theme.background_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.framed_rectangle.maybe_frame
            .or_else(|| theme.widget_style::<Style>(KIND).map(|default| {
                default.style.framed_rectangle.maybe_frame.unwrap_or(theme.frame_width)
            }))
            .unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.framed_rectangle.maybe_frame_color
            .or_else(|| theme.widget_style::<Style>(KIND).map(|default| {
                default.style.framed_rectangle.maybe_frame_color.unwrap_or(theme.frame_color)
            }))
            .unwrap_or(theme.frame_color)
    }

    /// Get the font size of the title bar.
    pub fn title_bar_font_size(&self, theme: &Theme) -> FontSize {
        self.text.maybe_font_size
            .or_else(|| theme.widget_style::<Style>(KIND).map(|default| {
                default.style.text.maybe_font_size.unwrap_or(theme.font_size_medium)
            }))
            .unwrap_or(theme.font_size_medium)
    }

    // /// Get the alignment of the title bar label.
    // pub fn title_bar_label_align(&self, theme: &Theme) -> Horizontal {
    //     const DEFAULT_ALIGN: Horizontal = Horizontal::Middle;
    //     self.maybe_title_bar_label_align.or_else(|| theme.widget_style::<Style>(KIND).map(|default| {
    //         default.style.maybe_title_bar_label_align.unwrap_or(DEFAULT_ALIGN)
    //     })).unwrap_or(DEFAULT_ALIGN)
    // }

    /// The length of the Canvas when instantiated as a split of some parent canvas.
    pub fn length(&self, theme: &Theme) -> Length {
        const DEFAULT_LENGTH: Length = Length::Weight(1.0);
        self.maybe_length.or_else(|| theme.widget_style::<Self>(KIND).and_then(|default| {
            default.style.maybe_length
        })).unwrap_or(DEFAULT_LENGTH)
    }

    /// Get the color of the title bar label.
    pub fn title_bar_label_color(&self, theme: &Theme) -> Color {
        self.text.maybe_color
            .or_else(|| theme.widget_style::<Style>(KIND).map(|default| {
                default.style.text.maybe_color.unwrap_or(theme.label_color)
            }))
            .unwrap_or(theme.label_color)
    }

    /// Get the Padding for the Canvas' kid area.
    pub fn padding(&self, theme: &Theme) -> position::Padding {
        let default_style = theme.widget_style::<Style>(KIND);
        position::Padding {
            x: Range {
                start: self.padding.maybe_left.or_else(|| default_style.as_ref().map(|default| {
                    default.style.padding.maybe_left.unwrap_or(theme.padding.x.start)
                })).unwrap_or(theme.padding.x.start),
                end: self.padding.maybe_right.or_else(|| default_style.as_ref().map(|default| {
                    default.style.padding.maybe_right.unwrap_or(theme.padding.x.end)
                })).unwrap_or(theme.padding.x.end),
            },
            y: Range {
                start: self.padding.maybe_bottom.or_else(|| default_style.as_ref().map(|default| {
                    default.style.padding.maybe_bottom.unwrap_or(theme.padding.y.start)
                })).unwrap_or(theme.padding.y.start),
                end: self.padding.maybe_top.or_else(|| default_style.as_ref().map(|default| {
                    default.style.padding.maybe_top.unwrap_or(theme.padding.y.end)
                })).unwrap_or(theme.padding.y.end),
            },
        }
    }

}


impl<'a> ::color::Colorable for Canvas<'a> {
    fn color(mut self, color: Color) -> Self {
        self.style.framed_rectangle.maybe_color = Some(color);
        self
    }
}

impl<'a> ::frame::Frameable for Canvas<'a> {
    fn frame(mut self, width: f64) -> Self {
        self.style.framed_rectangle.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.framed_rectangle.maybe_frame_color = Some(color);
        self
    }
}

impl<'a> ::label::Labelable<'a> for Canvas<'a> {
    fn label(self, text: &'a str) -> Self {
        self.title_bar(text)
    }
    fn label_color(mut self, color: Color) -> Self {
        self.style.text.maybe_color = Some(color);
        self
    }
    fn label_font_size(mut self, size: FontSize) -> Self {
        self.style.text.maybe_font_size = Some(size);
        self
    }
}

