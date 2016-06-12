use {
    Color,
    Dimension,
    Rect,
    Widget,
    Ui,
};
use widget;

/// A primitive and basic widget for drawing an `Image`.
#[derive(Copy, Clone)]
pub struct Image {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// The rectangle area of the original source image that should be used.
    pub src_rect: Option<Rect>,
    /// Unique styling.
    pub style: Style,
    /// A unique index representing a widget.
    ///
    /// It is up to the user to ensure that the `texture_index` is unique and mapped to the correct
    /// texture.
    pub texture_index: usize,
}

/// Unique `State` to be stored between updates for the `Image`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State {
    /// A unique index into the `Texture`.
    pub texture_index: usize,
    /// The rectangular area of the texture to use as the image.
    pub src_rect: Option<Rect>,
}

/// Unique kind for the widget.
pub const KIND: widget::Kind = "Image";

widget_style!{
    KIND;
    /// Unique styling for the `Image` widget.
    style Style {
        /// Optionally specify a single colour to use for the image.
        - maybe_color: Option<Color> { None }
    }
}


impl Image {

    /// Construct a new `Image`.
    pub fn new(texture_index: usize) -> Self {
        Image {
            common: widget::CommonBuilder::new(),
            src_rect: None,
            style: Style::new(),
            texture_index: texture_index,
        }
    }

    builder_methods!{
        pub source_rectangle { src_rect = Some(Rect) }
        pub color { style.maybe_color = Some(Option<Color>) }
    }

}


impl Widget for Image {
    type State = State;
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

    fn init_state(&self) -> Self::State {
        State {
            texture_index: self.texture_index,
            src_rect: None,
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn default_x_dimension(&self, ui: &Ui) -> Dimension {
        match self.src_rect.as_ref() {
            Some(rect) => Dimension::Absolute(rect.w()),
            None => widget::default_x_dimension(self, ui),
        }
    }

    fn default_y_dimension(&self, ui: &Ui) -> Dimension {
        match self.src_rect.as_ref() {
            Some(rect) => Dimension::Absolute(rect.h()),
            None => widget::default_y_dimension(self, ui),
        }
    }

    fn update(self, args: widget::UpdateArgs<Self>) {
        let widget::UpdateArgs { state, .. } = args;
        let Image { src_rect, texture_index, .. } = self;

        if state.texture_index != texture_index {
            state.update(|state| state.texture_index = texture_index);
        }

        if state.src_rect != src_rect {
            state.update(|state| state.src_rect = src_rect);
        }
    }

}
