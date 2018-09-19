//! A widget for displaying and mutating a one-line field of text.

use {Color, Colorable, FontSize, Borderable, Positionable, Sizeable, Widget};
use event;
use input;
use position::{Range, Rect, Scalar};
use text;
use widget;

/// A widget for displaying and mutating a small, one-line field of text, given by the user in the
/// form of a `String`.
///
/// It's reaction is triggered upon pressing of the `Enter`/`Return` key.
#[derive(WidgetCommon_)]
pub struct TextBox<'a> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    text: &'a str,
    style: Style,
}

/// Unique graphical styling for the TextBox.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// The length of the gap between the bounding rectangle's border and the edge of the text.
    #[conrod(default = "5.0")]
    pub text_padding: Option<Scalar>,
    /// Color of the rectangle behind the text.
    ///
    /// If you don't want to see the rectangle, either set the color with a zeroed alpha or use
    /// the `TextEdit` widget directly.
    #[conrod(default = "theme.shape_color")]
    pub color: Option<Color>,
    /// The width of the bounding `BorderedRectangle` border.
    #[conrod(default = "theme.border_width")]
    pub border: Option<Scalar>,
    /// The color of the `BorderedRecangle`'s border.
    #[conrod(default = "theme.border_color")]
    pub border_color: Option<Color>,
    /// The color of the `TextEdit` widget.
    #[conrod(default = "theme.label_color")]
    pub text_color: Option<Color>,
    /// The font size for the text.
    #[conrod(default = "theme.font_size_medium")]
    pub font_size: Option<FontSize>,
    /// The typographic alignment of the text.
    #[conrod(default = "text::Justify::Left")]
    pub justify: Option<text::Justify>,
    /// The font used for the `Text`.
    #[conrod(default = "theme.font_id")]
    pub font_id: Option<Option<text::font::Id>>,
}

widget_ids! {
    struct Ids {
        text_edit,
        rectangle,
    }
}

/// The `State` of the `TextBox` widget that will be cached within the `Ui`.
pub struct State {
    ids: Ids,
}

impl<'a> TextBox<'a> {

    /// Construct a TextBox widget.
    pub fn new(text: &'a str) -> Self {
        TextBox {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            text: text,
        }
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn left_justify(self) -> Self {
        self.justify(text::Justify::Left)
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn center_justify(self) -> Self {
        self.justify(text::Justify::Center)
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn right_justify(self) -> Self {
        self.justify(text::Justify::Right)
    }

    /// Specify the font used for displaying the text.
    pub fn font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.font_id = Some(Some(font_id));
        self
    }

    builder_methods!{
        pub text_color { style.text_color = Some(Color) }
        pub font_size { style.font_size = Some(FontSize) }
        pub justify { style.justify = Some(text::Justify) }
        pub pad_text { style.text_padding = Some(Scalar) }
    }

}

/// Events produced by the `TextBox`.
#[derive(Clone, Debug)]
pub enum Event {
    /// The `String` was updated.
    Update(String),
    /// The `Return` or `Enter` key was pressed.
    Enter,
}

impl<'a> Widget for TextBox<'a> {
    type State = State;
    type Style = Style;
    type Event = Vec<Event>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the TextEdit.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, rect, style, ui, .. } = args;
        let TextBox { text, .. } = self;

        let font_size = style.font_size(ui.theme());
        let border = style.border(ui.theme());
        let text_padding = style.text_padding(ui.theme());
        let justify = style.justify(ui.theme());

        let text_rect = {
            let w = rect.x.pad(border + text_padding).len();
            let h = font_size as Scalar + 1.0;
            let x = Range::new(0.0, w).align_middle_of(rect.x);
            let y = Range::new(0.0, h).align_middle_of(rect.y);
            Rect { x: x, y: y }
        };

        let color = style.color(ui.theme());
        let border_color = style.border_color(ui.theme());
        widget::BorderedRectangle::new(rect.dim())
            .xy(rect.xy())
            .graphics_for(id)
            .parent(id)
            .border(border)
            .color(color)
            .border_color(border_color)
            .set(state.ids.rectangle, ui);

        let mut events = Vec::new();

        let text_color = style.text_color(ui.theme());
        let font_id = style.font_id(&ui.theme).or(ui.fonts.ids().next());
        if let Some(new_string) = widget::TextEdit::new(text)
            .and_then(font_id, widget::TextEdit::font_id)
            .wh(text_rect.dim())
            .xy(text_rect.xy())
            .font_size(font_size)
            .color(text_color)
            .justify(justify)
            .parent(id)
            .set(state.ids.text_edit, ui)
        {
            events.push(Event::Update(new_string));
        }

        // Produce an event for any `Enter`/`Return` presses.
        //
        // TODO: We should probably be doing this via the `TextEdit` widget.
        for widget_event in ui.widget_input(state.ids.text_edit).events() {
            match widget_event {
                event::Widget::Press(press) => match press.button {
                    event::Button::Keyboard(key) => match key {
                        input::Key::Return => events.push(Event::Enter),
                        _ => (),
                    },
                    _ => (),
                },
                _ => (),
            }
        }

        events
    }

}

impl<'a> Borderable for TextBox<'a> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a> Colorable for TextBox<'a> {
    builder_method!(color { style.color = Some(Color) });
}
