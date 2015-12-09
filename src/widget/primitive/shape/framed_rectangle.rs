use {
    CharacterCache,
    Color,
    Colorable,
    Dimensions,
    Frameable,
    Scalar,
    Sizeable,
    Theme,
    Widget,
};
use widget;


/// A filled rectangle widget that may or may not have some frame.
#[derive(Copy, Clone, Debug)]
pub struct FramedRectangle {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// Unique styling for the **FramedRectangle**.
    pub style: Style,
}


/// Unique styling for the **FramedRectangle** widget.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    /// Shape styling for the inner rectangle.
    pub maybe_color: Option<Color>,
    /// The thickness of the frame.
    pub maybe_frame: Option<Scalar>,
    /// The color of the frame.
    pub maybe_frame_color: Option<Color>,
}

/// Unique kind for the Widget.
pub const KIND: widget::Kind = "FramedRectangle";

impl FramedRectangle {

    /// Build a new **FramedRectangle**.
    pub fn new(dim: Dimensions) -> Self {
        FramedRectangle {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
        }.dim(dim)
    }

    /// Build the **FramedRectangle** with the given styling.
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

}


impl Widget for FramedRectangle {
    type State = ();
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> &'static str {
        KIND
    }

    fn init_state(&self) -> () {
        ()
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the Rectangle.
    fn update<C: CharacterCache>(self, _args: widget::UpdateArgs<Self, C>) {
        // Nothing to update here!
    }

}


impl Style {

    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
        }
    }

    /// Get the Color for an Element.
    pub fn get_color(&self, theme: &Theme) -> Color {
        self.maybe_color.unwrap_or_else(|| theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn get_frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.unwrap_or_else(|| theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn get_frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.unwrap_or_else(|| theme.frame_color)
    }

}




impl Colorable for FramedRectangle {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}


impl Frameable for FramedRectangle {
    fn frame(mut self, width: Scalar) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
        self
    }
}

