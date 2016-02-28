use {
    Backend,
    Color,
    Dimension,
    Rect,
    Scalar,
    Widget,
    Ui,
};
use backend::graphics::ImageSize;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use widget;

/// A primitive and basic widget for drawing an `Image`.
pub struct Image<T> {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// The rectangle area of the original source image that should be used.
    pub src_rect: Option<Rect>,
    /// Unique styling.
    pub style: Style,
    /// Where the `Image` data is stored.
    pub src: Source<T>,
}

/// Where the `Image` data is stored.
pub enum Source<T> {
    Texture(Arc<T>),
}

/// Unique `State` to be stored between updates for the `Image`.
#[derive(Clone)]
pub struct State<T>
    where T: ImageSize
{
    /// The `Texture` used by the `Image` along with its source rectangle.
    pub texture: Option<Texture<T>>,
}

/// The `Texture` used by the `Image` along with its source rectangle.
#[derive(Clone)]
pub struct Texture<T> {
    /// A pointer to the backend texture type.
    pub arc: Arc<T>,
    /// The rectangular area of the texture to use as the image.
    pub src_rect: Rect,
}

impl<T> PartialEq for State<T>
    where T: ImageSize,
{
    fn eq(&self, other: &Self) -> bool {
        self.texture == other.texture
    }
}

impl<T> Debug for State<T>
    where T: ImageSize,
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{:?}", &self.texture)
    }
}

impl<T> PartialEq for Texture<T>
    where T: ImageSize,
{
    fn eq(&self, other: &Self) -> bool {
        self.arc.get_size() == other.arc.get_size() && self.src_rect == other.src_rect
    }
}

impl<T> Debug for Texture<T>
    where T: ImageSize,
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let (w, h) = self.arc.get_size();
        write!(f, "size: [{:?}, {:?}], src_rect: {:?}", w, h, self.src_rect)
    }
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


impl<T> Image<T> {

    /// Construct a new `Image`.
    fn new(src: Source<T>) -> Self {
        Image {
            common: widget::CommonBuilder::new(),
            src_rect: None,
            style: Style::new(),
            src: src,
        }
    }

    /// Construct a new `Image` from a texture.
    pub fn from_texture(texture: Arc<T>) -> Self {
        Self::new(Source::Texture(texture))
    }

    builder_methods!{
        pub source_rectangle { src_rect = Some(Rect) }
        pub color { style.maybe_color = Some(Option<Color>) }
    }

}


impl<T> Widget for Image<T>
    where T: Any + ImageSize,
{
    type State = State<T>;
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
            texture: None,
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn default_x_dimension<B: Backend>(&self, _ui: &Ui<B>) -> Dimension {
        match self.src_rect.as_ref() {
            Some(rect) => Dimension::Absolute(rect.w()),
            None => match self.src {
                Source::Texture(ref texture) => {
                    let (w, _) = texture.get_size();
                    Dimension::Absolute(w as Scalar)
                },
            },
        }
    }

    fn default_y_dimension<B: Backend>(&self, _ui: &Ui<B>) -> Dimension {
        match self.src_rect.as_ref() {
            Some(rect) => Dimension::Absolute(rect.h()),
            None => match self.src {
                Source::Texture(ref texture) => {
                    let (_, h) = texture.get_size();
                    Dimension::Absolute(h as Scalar)
                },
            },
        }
    }

    fn update<B: Backend>(self, args: widget::UpdateArgs<Self, B>) {
        let widget::UpdateArgs { state, .. } = args;
        let Image { src_rect, src, .. } = self;

        match src {
            Source::Texture(texture) => {
                let src_rect = src_rect.unwrap_or_else(|| {
                    let (w, h) = texture.get_size();
                    Rect::from_xy_dim([0.0, 0.0], [w as Scalar, h as Scalar])
                });

                let src_rect_has_changed = 
                    state.view().texture.as_ref().map(|t| &t.src_rect) != Some(&src_rect);
                let texture_size_changed =
                    state.view().texture.as_ref().map(|t| t.arc.get_size())
                    != Some(texture.get_size());
                if src_rect_has_changed || texture_size_changed {
                    state.update(|state| state.texture = Some(Texture {
                        arc: texture,
                        src_rect: src_rect,
                    }));
                }
            },
        }
    }

}
