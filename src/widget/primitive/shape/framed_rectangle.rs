use {CharacterCache, Scalar, Theme};
use color::{Color, Colorable};
use elmesque::Element;
use frame::Frameable;
use position::{Dimensions, Sizeable};
use widget::{self, Widget};


/// A filled rectangle widget that may or may not have some frame.
#[derive(Copy, Clone, Debug)]
pub struct FramedRectangle {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// Unique styling for the **FramedRectangle**.
    pub style: Style,
}


/// Unique styling for the **FramedRectangle** widget.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    /// Shape styling for the inner rectangle.
    pub maybe_color: Option<Color>,
    /// The thickness of the frame.
    pub maybe_frame: Option<Scalar>,
    /// The color of the frame.
    pub maybe_frame_color: Option<Color>,
}


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
        "FramedRectangle"
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

    /// Construct an Element for the Rectangle.
    fn draw<C: CharacterCache>(args: widget::DrawArgs<Self, C>) -> Element {
        use elmesque::form::{self, collage};
        let widget::DrawArgs { rect, style, theme, .. } = args;

        let (x, y, w, h) = rect.x_y_w_h();
        let frame = style.get_frame(theme);

        let maybe_frame_form = if frame > 0.0 {
            let frame_color = style.get_frame_color(theme);
            Some(form::rect(w, h).filled(frame_color).shift(x, y))
        } else {
            None
        };

        let (inner_w, inner_h) = rect.pad(frame).w_h();
        let color = style.get_color(theme);
        let inner_form = form::rect(inner_w, inner_h).filled(color).shift(x, y);

        let forms = maybe_frame_form.into_iter().chain(::std::iter::once(inner_form));
        collage(w as i32, h as i32, forms.collect())
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

