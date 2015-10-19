
use Scalar;
use color::{Color, Colorable};
use elmesque::Element;
use graphics::character::CharacterCache;
use label::FontSize;
use theme::Theme;
use ui::GlyphCache;
use widget::{self, Widget};


/// Displays some given text centred within a rectangle.
#[derive(Clone, Debug)]
pub struct Label<'a> {
    common: widget::CommonBuilder,
    text: &'a str,
    style: Style,
}

/// The styling for a Label's renderable Element.
#[allow(missing_docs, missing_copy_implementations)]
#[derive(Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    maybe_font_size: Option<FontSize>,
    maybe_color: Option<Color>,
}

/// The state to be stored between updates for the Label.
#[derive(Clone, Debug, PartialEq)]
pub struct State(String);


impl<'a> Label<'a> {

    /// Construct a new Label widget.
    pub fn new(text: &'a str) -> Label<'a> {
        Label {
            common: widget::CommonBuilder::new(),
            text: text,
            style: Style::new(),
        }
    }

    /// Set the font size for the label.
    #[inline]
    pub fn font_size(mut self, size: FontSize) -> Label<'a> {
        self.style.maybe_font_size = Some(size);
        self
    }

}


impl<'a> Widget for Label<'a> {
    type State = State;
    type Style = Style;
    fn common(&self) -> &widget::CommonBuilder { &self.common }
    fn common_mut(&mut self) -> &mut widget::CommonBuilder { &mut self.common }
    fn unique_kind(&self) -> &'static str { "Label" }
    fn init_state(&self) -> State { State(String::new()) }
    fn style(&self) -> Style { self.style.clone() }

    fn default_width<C: CharacterCache>(&self, theme: &Theme, glyph_cache: &GlyphCache<C>) -> Scalar {
        glyph_cache.width(self.style.font_size(theme), self.text)
    }

    fn default_height(&self, theme: &Theme) -> Scalar {
        self.style.font_size(theme) as Scalar
    }

    /// Update the state of the Label.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { state, .. } = args;
        if &state.view().0[..] != self.text {
            state.update(|state| *state = State(self.text.to_owned()));
        }
    }

    /// Construct an Element for the Label.
    fn draw<C: CharacterCache>(args: widget::DrawArgs<Self, C>) -> Element {
        use elmesque::form::{text, collage};
        use elmesque::text::Text;
        let widget::DrawArgs { rect, state: &State(ref string), style, theme, .. } = args;
        let size = style.font_size(theme);
        let color = style.color(theme);
        let (x, y, w, h) = rect.x_y_w_h();
        let form = text(Text::from_string(string.clone())
                            .color(color)
                            .height(size as f64)).shift(x.floor(), y.floor());
        collage(w as i32, h as i32, vec![form])
    }
    
}


impl Style {

    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_color: None,
            maybe_font_size: None,
        }
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_font_size.unwrap_or(theme.font_size_medium)
    }

}


impl<'a> Colorable for Label<'a> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

