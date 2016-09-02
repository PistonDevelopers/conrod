//! A simple, non-interactive widget for drawing an `Image`.

use {Color, Dimension, Rect, Widget, Ui};
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
}

/// Unique `State` to be stored between updates for the `Image`.
#[derive(Copy, Clone)]
pub struct State {
    /// The rectangular area of the image that we wish to display.
    ///
    /// If `None`, the entire image will be used.
    pub src_rect: Option<Rect>,
}

widget_style!{
    /// Unique styling for the `Image` widget.
    style Style {
        /// Optionally specify a single colour to use for the image.
        - maybe_color: Option<Color> { None }
    }
}


impl Image {

    /// Construct a new `Image`.
    ///
    /// Note that the `Image` widget does not require borrowing or owning any image data directly.
    /// Instead, image data is stored within a `conrod::image::Map` where widget indices are mapped
    /// to their associated data.
    ///
    /// This is done for a few reasons:
    ///
    /// - To avoid requiring that the widget graph owns an instance of each image
    /// - To avoid requiring that the user passes the image data to the `Image` every update
    /// unnecessarily
    /// - To make it easier for users to borrow and mutate their images without needing to index
    /// into the `Ui`'s widget graph (which also requires casting types).
    ///
    /// During rendering, conrod will take the `image::Map`, retrieve the data associated with each
    /// image and yield it via the `render::Primitive::Image` variant.
    ///
    /// Note: this implies that the type must be the same for all `Image` widgets instantiated via
    /// the same `Ui`. In the case that you require multiple different types of images, we
    /// recommend that you either:
    ///
    /// 1. use an enum with a variant for each type
    /// 2. use a trait object, where the trait is implemented for each of your image types or
    /// 3. use an index type which may be mapped to your various image types.
    pub fn new() -> Self {
        Image {
            common: widget::CommonBuilder::new(),
            src_rect: None,
            style: Style::new(),
        }
    }

    /// The rectangular area of the image that we wish to display.
    ///
    /// If this method is not called, the entire image will be used.
    pub fn source_rectangle(mut self, rect: Rect) -> Self {
        self.src_rect = Some(rect);
        self
    }

    builder_methods!{
        pub color { style.maybe_color = Some(Option<Color>) }
    }

}


impl Widget for Image {
    type State = State;
    type Style = Style;
    type Event = ();

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State {
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

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { state, .. } = args;
        let Image { src_rect, .. } = self;

        if state.src_rect != src_rect {
            state.update(|state| state.src_rect = src_rect);
        }
    }

}
