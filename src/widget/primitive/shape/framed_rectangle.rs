use {
    CharacterCache,
    Color,
    Colorable,
    Dimensions,
    Frameable,
    Scalar,
    Sizeable,
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

/// Unique kind for the Widget.
pub const KIND: widget::Kind = "FramedRectangle";

widget_style!{
    KIND;
    /// Unique styling for the **FramedRectangle** widget.
    style Style {
        /// Shape styling for the inner rectangle.
        - color: Color { theme.shape_color },
        /// The thickness of the frame.
        - frame: Scalar { theme.frame_width },
        /// The color of the frame.
        - frame_color: Color { theme.frame_color },
    }
}

impl FramedRectangle {

    /// Build a new **FramedRectangle**.
    pub fn new(dim: Dimensions) -> Self {
        FramedRectangle {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
        }.wh(dim)
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


impl Colorable for FramedRectangle {
    fn color(mut self, color: Color) -> Self {
        self.style.color = Some(color);
        self
    }
}


impl Frameable for FramedRectangle {
    fn frame(mut self, width: Scalar) -> Self {
        self.style.frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.frame_color = Some(color);
        self
    }
}
