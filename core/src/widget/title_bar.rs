//! A simple title bar widget that automatically sizes itself to the top of some other widget.

use {Color, Colorable, FontSize, Borderable, Labelable, Positionable, Sizeable, Ui};
use position::{self, Align, Dimension, Scalar};
use text;
use widget::{self, Widget};


/// A simple title bar widget that automatically sizes itself to the top of some other widget.
#[derive(Clone, WidgetCommon_)]
pub struct TitleBar<'a> {
    /// Data necessary and common for all widget builder types.
    #[conrod(common_builder)]
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

/// Unique styling for the **TitleBar** widget.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// The color of the TitleBar's rectangle surface.
    #[conrod(default = "theme.background_color")]
    pub color: Option<Color>,
    /// The width of the border surrounding the TitleBar's rectangle.
    #[conrod(default = "theme.border_width")]
    pub border: Option<Scalar>,
    /// The color of the TitleBar's border.
    #[conrod(default = "theme.border_color")]
    pub border_color: Option<Color>,
    /// The color of the title bar's text.
    #[conrod(default = "theme.label_color")]
    pub text_color: Option<Color>,
    /// The font size for the title bar's text.
    #[conrod(default = "theme.font_size_medium")]
    pub font_size: Option<FontSize>,
    /// The way in which the title bar's text should wrap.
    #[conrod(default = "Some(widget::text::Wrap::Whitespace)")]
    pub maybe_wrap: Option<Option<widget::text::Wrap>>,
    /// The distance between lines for multi-line title bar text.
    #[conrod(default = "1.0")]
    pub line_spacing: Option<Scalar>,
    /// The horizontal alignment of the title bar text.
    #[conrod(default = "text::Justify::Center")]
    pub justify: Option<text::Justify>,
    /// The position of the title bar's `Label` widget over the *x* axis.
    #[conrod(default = "position::Relative::Align(Align::Middle)")]
    pub label_x: Option<position::Relative>,
    /// The position of the title bar's `Label` widget over the *y* axis.
    #[conrod(default = "position::Relative::Align(Align::Middle)")]
    pub label_y: Option<position::Relative>,
    /// The font used for the `Text`.
    #[conrod(default = "theme.font_id")]
    pub font_id: Option<Option<text::font::Id>>,
}

/// The padding between the edge of the title bar and the title bar's label.
///
/// This is used to determine the size of the TitleBar.
const LABEL_PADDING: f64 = 4.0;


impl<'a> TitleBar<'a> {

    /// Construct a new TitleBar widget and attach it to the widget at the given index.
    pub fn new(label: &'a str, id: widget::Id) -> Self {
        TitleBar {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            label: label,
        }.w_of(id).mid_top_of(id)
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn left_justify_label(mut self) -> Self {
        self.style.justify = Some(text::Justify::Left);
        self
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn center_justify_label(mut self) -> Self {
        self.style.justify = Some(text::Justify::Center);
        self
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn right_justify_label(mut self) -> Self {
        self.style.justify = Some(text::Justify::Right);
        self
    }

    /// Specify the label's position relatively to `Button` along the *x* axis.
    pub fn label_x(mut self, x: position::Relative) -> Self {
        self.style.label_x = Some(x);
        self
    }

    /// Specify the label's position relatively to `Button` along the *y* axis.
    pub fn label_y(mut self, y: position::Relative) -> Self {
        self.style.label_y = Some(y);
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
        let justify = style.justify(ui.theme());
        let font_size = style.font_size(ui.theme());
        let line_spacing = style.line_spacing(ui.theme());
        let maybe_wrap = style.maybe_wrap(ui.theme());
        let font_id = style.font_id(&ui.theme).or(ui.fonts.ids().next());
        let label_x = style.label_x(&ui.theme);
        let label_y = style.label_y(&ui.theme);
        widget::Text::new(label)
            .and_mut(|text| {
                text.style.maybe_wrap = Some(maybe_wrap);
                text.style.justify = Some(justify);
            })
            .and_then(font_id, widget::Text::font_id)
            .padded_w_of(state.ids.rectangle, border)
            .x_position_relative_to(state.ids.rectangle, label_x)
            .y_position_relative_to(state.ids.rectangle, label_y)
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
