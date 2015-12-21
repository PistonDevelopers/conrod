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
    TextStyle,
    Theme,
    TitleBar,
    Ui,
    Widget,
};
use position;
use widget::{self, title_bar};


/// A widget designed to be a parent for other widgets.
pub struct Canvas<'a> {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// The builder data related to the style of the Canvas.
    pub style: Style,
    /// The label for the **Canvas**' **TitleBar** if there is one.
    pub maybe_title_bar_label: Option<&'a str>, 
}

/// **Canvas** state to be cached.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    rectangle_idx: IndexSlot,
    title_bar_idx: IndexSlot,
}

/// Unique kind for the widget type.
pub const KIND: widget::Kind = "Canvas";


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

/// Describes the style of a Canvas.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    /// Styling for the Canvas' rectangle.
    pub framed_rectangle: FramedRectangleStyle,
    /// The label and styling for the Canvas' title bar if it has one.
    pub text: TextStyle,
    /// Padding of the kid area.
    pub padding: PaddingBuilder,
}


impl<'a> Canvas<'a> {

    /// Construct a new Canvas builder.
    pub fn new() -> Canvas<'a> {
        Canvas {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
            maybe_title_bar_label: None,
        }
    }

    /// Show or hide the title bar.
    pub fn title_bar(mut self, label: &'a str) -> Self {
        self.maybe_title_bar_label = Some(label);
        self
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
        widget::default_x_dimension(self, ui).unwrap_or(Dimension::Absolute(64.0))
    }

    fn default_y_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        widget::default_y_dimension(self, ui).unwrap_or(Dimension::Absolute(80.0))
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
        let Canvas { style, maybe_title_bar_label, .. } = self;

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

