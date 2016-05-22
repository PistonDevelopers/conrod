

/// A widget for displaying and mutating a given one-line text `String`. It's reaction is
/// triggered upon pressing of the `Enter`/`Return` key.
pub struct TextBox<'a, F> {
    common: widget::CommonBuilder,
    text: &'a mut String,
    /// The reaction for the TextEdit.
    ///
    /// If `Some`, this will be triggered upon pressing of the `Enter`/`Return` key.
    pub maybe_react: Option<F>,
    style: Style,
}


/// Unique kind for the widget type.
pub const KIND: widget::Kind = "TextBox";

widget_style!{
    KIND;
    /// Unique graphical styling for the TextEdit.
    style Style {
        /// Color of the rectangle behind the text.
        ///
        /// If you don't want to see the rectangle, set the color with a zeroed alpha.
        - color: Color { theme.shape_color }
        /// The color
        - text_color: Color { theme.text_color }
        /// The font size for the text.
        - font_size: FontSize { 24 }
        /// The horizontal alignment of the text.
        - x_align: Align { Align::Start }
        /// The way in which text is wrapped at the end of a line.
        - line_wrap: Wrap { Wrap::Whitespace }
    }
}

impl<'a, F> TextBox<'a, F> {

    /// Construct a TextEdit widget.
    pub fn new(text: &'a mut String) -> Self {
        TextEdit {
            common: widget::CommonBuilder::new(),
            text: text,
            maybe_react: None,
            style: Style::new(),
        }
    }

    builder_methods!{
        pub font_size { style.font_size = Some(FontSize) }
        pub react { maybe_react = Some(F) }
    }

}
