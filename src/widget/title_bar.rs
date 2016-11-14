//! A simple title bar widget that automatically sizes itself to the top of some other widget.

use {
    Align,
    Color,
    Colorable,
    Dimension,
    FontSize,
    Borderable,
    Labelable,
    Positionable,
    Scalar,
    Sizeable,
    Ui,
};
use text;
use widget::{self, Widget};


/// A simple title bar widget that automatically sizes itself to the top of some other widget.
pub struct TitleBar<'a> {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// Unique styling for the **BorderedRectangle**.
    pub style: Style,
    /// A label displayed in the middle of the TitleBar.
    pub label: &'a str,
}

/// Unique state for the **TitleBar** widget.
pub struct State {
    ids: Ids,
}

widget_ids! {
    struct Ids {
        rectangle,
        label,
    }
}

widget_style!{
    /// Unique styling for the **TitleBar** widget.
    style Style {
        /// The color of the TitleBar's rectangle surface.
        - color: Color { theme.background_color }
        /// The width of the border surrounding the TitleBar's rectangle.
        - border: Scalar { theme.border_width }
        /// The color of the TitleBar's border.
        - border_color: Color { theme.border_color }

        /// The color of the title bar's text.
        - text_color: Color { theme.label_color }
        /// The font size for the title bar's text.
        - font_size: FontSize { theme.font_size_medium }
        /// The way in which the title bar's text should wrap.
        - maybe_wrap: Option<widget::text::Wrap> { Some(widget::text::Wrap::Whitespace) }
        /// The distance between lines for multi-line title bar text.
        - line_spacing: Scalar { 1.0 }
        /// The horizontal alignment of the title bar text.
        - text_align: Align { Align::Middle }
        /// The font used for the `Text`.
        - font_id: Option<text::font::Id> { theme.font_id }
    }
}

/// The padding between the edge of the title bar and the title bar's label.
///
/// This is used to determine the size of the TitleBar.
const LABEL_PADDING: f64 = 4.0;


impl<'a> TitleBar<'a> {

    /// Construct a new TitleBar widget and attach it to the widget at the given index.
    pub fn new(label: &'a str, id: widget::Id) -> Self {
        TitleBar {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
            label: label,
        }.w_of(id).mid_top_of(id)
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn align_text_left(mut self) -> Self {
        self.style.text_align = Some(Align::Start);
        self
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn align_text_middle(mut self) -> Self {
        self.style.text_align = Some(Align::Middle);
        self
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn align_text_right(mut self) -> Self {
        self.style.text_align = Some(Align::End);
        self
    }

    /// Specify the font used for displaying the text.
    pub fn font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.font_id = Some(Some(font_id));
        self
    }

    builder_methods!{
        pub line_spacing { style.line_spacing = Some(Scalar) }
    }

}


/// Calculate the default height for the **TitleBar**'s rect.
pub fn calc_height(font_size: FontSize) -> Scalar {
    font_size as Scalar + LABEL_PADDING * 2.0
}


impl<'a> Widget for TitleBar<'a> {
    type State = State;
    type Style = Style;
    type Event = ();

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn default_y_dimension(&self, ui: &Ui) -> Dimension {
        let font_size = self.style.font_size(&ui.theme);
        let h = calc_height(font_size);
        Dimension::Absolute(h)
    }

    fn update(self, args: widget::UpdateArgs<Self>) {
        let widget::UpdateArgs { id, state, rect, style, ui, .. } = args;
        let TitleBar { label, .. } = self;

        // BorderedRectangle widget.
        let dim = rect.dim();
        let color = style.color(ui.theme());
        let border = style.border(ui.theme());
        let border_color = style.border_color(ui.theme());
        widget::BorderedRectangle::new(dim)
            .color(color)
            .border(border)
            .border_color(border_color)
            .middle_of(id)
            .graphics_for(id)
            .set(state.ids.rectangle, ui);

        // Label widget.
        let text_color = style.text_color(ui.theme());
        let text_align = style.text_align(ui.theme());
        let font_size = style.font_size(ui.theme());
        let line_spacing = style.line_spacing(ui.theme());
        let maybe_wrap = style.maybe_wrap(ui.theme());
        let font_id = style.font_id(&ui.theme).or(ui.fonts.ids().next());
        widget::Text::new(label)
            .and_mut(|text| {
                text.style.maybe_wrap = Some(maybe_wrap);
                text.style.text_align = Some(text_align);
            })
            .and_then(font_id, widget::Text::font_id)
            .padded_w_of(state.ids.rectangle, border)
            .middle_of(state.ids.rectangle)
            .color(text_color)
            .font_size(font_size)
            .line_spacing(line_spacing)
            .graphics_for(id)
            .set(state.ids.label, ui);
    }

}


impl<'a> Colorable for TitleBar<'a> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a> Borderable for TitleBar<'a> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a> Labelable<'a> for TitleBar<'a> {
    builder_methods!{
        label { label = &'a str }
        label_color { style.text_color = Some(Color) }
        label_font_size { style.font_size = Some(FontSize) }
    }
}
