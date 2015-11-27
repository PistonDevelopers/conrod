
use {CharacterCache, Dimension, Scalar};
use color::{Color, Colorable};
use label::FontSize;
use theme::Theme;
use ui::Ui;
use widget::{self, Widget};


/// Displays some given text centred within a rectangular area.
///
/// By default, the rectangular dimensions are fit to the area occuppied by the text.
///
/// If some horizontal dimension is given, the text will automatically wrap to the width and align
/// in accordance with the produced **HorizontalAlign**.
pub struct Text<'a> {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// The text to be drawn by the **Text**.
    pub text: &'a str,
    /// Unique styling for the **Text**.
    pub style: Style,
}


/// The styling for a **Text**'s graphics.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    /// The font size for the **Text**.
    pub maybe_font_size: Option<FontSize>,
    /// The color of the **Text**.
    pub maybe_color: Option<Color>,
    /// Whether or not the text should wrap around the width.
    pub maybe_wrap: Option<Option<Wrap>>,
    /// The spacing between consecutive lines.
    pub maybe_line_spacing: Option<Scalar>,
    // /// The typeface with which the Text is rendered.
    // pub maybe_typeface: Option<Path>,
    // /// The line styling for the text.
    // pub maybe_line: Option<Option<Line>>,
}

/// The unique kind for the widget.
pub const KIND: widget::Kind = "Text";

/// The way in which text should wrap around the width.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum Wrap {
    /// Wrap at the first character that exceeds the width.
    Character,
    /// Wrap at the first word that exceeds the width.
    Whitespace,
}

// /// Line styling for the **Text**.
// pub enum Line {
//     /// Underline the text.
//     Under,
//     /// Overline the text.
//     Over,
//     /// Strikethrough the text.
//     Through,
// }

/// The state to be stored between updates for the **Text**.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// An owned version of the string.
    pub string: String,
}


impl<'a> Text<'a> {

    /// Build a new **Text** widget.
    pub fn new(text: &'a str) -> Self {
        Text {
            common: widget::CommonBuilder::new(),
            text: text,
            style: Style::new(),
        }
    }

    /// Build the **Text** with the given font size.
    pub fn font_size(mut self, size: FontSize) -> Self {
        self.style.maybe_font_size = Some(size);
        self
    }

    /// Specify that the **Text** should not wrap lines around the width.
    pub fn no_line_wrap(mut self) -> Self {
        self.style.maybe_wrap = Some(None);
        self
    }

    /// Line wrap the **Text** at the beginning of the first word that exceeds the width.
    pub fn wrap_by_word(mut self) -> Self {
        self.style.maybe_wrap = Some(Some(Wrap::Whitespace));
        self
    }

    /// Line wrap the **Text** at the beginning of the first character that exceeds the width.
    pub fn wrap_by_character(mut self) -> Self {
        self.style.maybe_wrap = Some(Some(Wrap::Character));
        self
    }

    /// Build the **Text** with the given **Style**.
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

}

impl Style {

    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_color: None,
            maybe_font_size: None,
            maybe_wrap: None,
            maybe_line_spacing: None,
        }
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.unwrap_or(theme.label_color)
    }

    /// Get the text font size for an Element.
    pub fn font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_font_size.unwrap_or(theme.font_size_medium)
    }

    /// Get whether or not the text should wrap around if the length exceeds the width.
    pub fn wrap(&self, theme: &Theme) -> Option<Wrap> {
        const DEFAULT_WRAP: Option<Wrap> = Some(Wrap::Whitespace);
        self.maybe_wrap.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_wrap.unwrap_or(DEFAULT_WRAP)
        })).unwrap_or(DEFAULT_WRAP)
    }

    /// Get the length of the separating space between consecutive lines of text.
    pub fn line_spacing(&self, theme: &Theme) -> Scalar {
        const DEFAULT_LINE_SPACING: Scalar = 1.0;
        self.maybe_line_spacing.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_line_spacing.unwrap_or(DEFAULT_LINE_SPACING)
        })).unwrap_or(DEFAULT_LINE_SPACING)
    }

}


impl<'a> Widget for Text<'a> {
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

    fn init_state(&self) -> State {
        State {
            string: String::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn default_x_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        Dimension::Absolute(ui.glyph_cache.width(self.style.font_size(&ui.theme), self.text))
    }

    fn default_y_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        Dimension::Absolute(self.style.font_size(&ui.theme) as Scalar)
    }

    /// Update the state of the Text.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { rect, state, style, ui, .. } = args;
        let Text { text, .. } = self;

        let maybe_wrap = style.wrap(ui.theme());
        match maybe_wrap {

            // If there is no wrapping, we'll just check to see if the strings match.
            None => {
                if &state.view().string[..] != text {
                    state.update(|state| state.string = text.to_owned());
                }
            },

            // If we have some wrapping, we'll need to iterate over the wrapped lines of both
            // the string and str to check for differences.
            Some(wrap) => {
                use std::iter::once;

                let max_w = rect.w();
                let font_size = style.font_size(ui.theme());

                fn are_line_iters_different<'a, 'b, A, B>(mut a: A, mut b: B) -> bool
                    where A: Iterator<Item=&'a str>,
                          B: Iterator<Item=&'b str>,
                {
                    loop {
                        match (a.next(), b.next()) {
                            (None, None) => return false,
                            (line, new_line) if line != new_line => return true,
                            _ => (),
                        }
                    }
                }

                fn collect_wrapped_string<'a, I: Iterator<Item=&'a str>>(strs: I) -> String {
                    strs.flat_map(|s| once(s).chain(once("\n"))).collect()
                }

                match wrap {

                    Wrap::Character => {
                        let new_lines =
                            || ui.glyph_cache().wrap_by_character(font_size, text, max_w);
                        let is_different = {
                            let lines = state.view().string.lines();
                            are_line_iters_different(lines, new_lines())
                        };
                        if is_different {
                            state.update(|state| state.string = collect_wrapped_string(new_lines()))
                        }
                    },

                    Wrap::Whitespace => {
                        let new_lines =
                            || ui.glyph_cache().wrap_by_whitespace(font_size, text, max_w);
                        let is_different = {
                            let lines = state.view().string.lines();
                            are_line_iters_different(lines, new_lines())
                        };
                        if is_different {
                            state.update(|state| state.string = collect_wrapped_string(new_lines()))
                        }
                    },
                }
            },

        }
    }

}



impl<'a> Colorable for Text<'a> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

