//! A widget for displaying and mutating a one-line field of text.

use {
    Align,
    Color,
    Colorable,
    FontSize,
    Borderable,
    Positionable,
    Range,
    Rect,
    Scalar,
    Sizeable,
    Widget,
};
use event;
use input;
use text;
use widget;

/// A widget for displaying and mutating a small, one-line field of text, given by the user in the
/// form of a `String`.
///
/// It's reaction is triggered upon pressing of the `Enter`/`Return` key.
pub struct TextBox<'a> {
    common: widget::CommonBuilder,
    text: &'a str,
    style: Style,
}

widget_style!{
    /// Unique graphical styling for the TextBox.
    style Style {
        /// The length of the gap between the bounding rectangle's border and the edge of the text.
        - text_padding: Scalar { 5.0 }
        /// Color of the rectangle behind the text.
        ///
        /// If you don't want to see the rectangle, either set the color with a zeroed alpha or use
        /// the `TextEdit` widget directly.
        - color: Color { theme.shape_color }
        /// The width of the bounding `BorderedRectangle` border.
        - border: Scalar { theme.border_width }
        /// The color of the `BorderedRecangle`'s border.
        - border_color: Color { theme.border_color }
        /// The color of the `TextEdit` widget.
        - text_color: Color { theme.label_color }
        /// The font size for the text.
        - font_size: FontSize { theme.font_size_medium }
        /// The horizontal alignment of the text.
        - x_align: Align { Align::Start }
        /// The font used for the `Text`.
        - font_id: Option<text::font::Id> { theme.font_id }
    }
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
            common: widget::CommonBuilder::new(),
            text: text,
            style: Style::new(),
        }
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn align_text_left(self) -> Self {
        self.x_align_text(Align::Start)
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn align_text_middle(self) -> Self {
        self.x_align_text(Align::Middle)
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn align_text_right(self) -> Self {
        self.x_align_text(Align::End)
    }

    /// Specify the font used for displaying the text.
    pub fn font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.font_id = Some(Some(font_id));
        self
    }

    builder_methods!{
        pub text_color { style.text_color = Some(Color) }
        pub font_size { style.font_size = Some(FontSize) }
        pub x_align_text { style.x_align = Some(Align) }
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

    /// Update the state of the TextEdit.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, rect, style, mut ui, .. } = args;
        let TextBox { text, .. } = self;

        let font_size = style.font_size(ui.theme());
        let border = style.border(ui.theme());
        let text_padding = style.text_padding(ui.theme());
        let x_align = style.x_align(ui.theme());

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
            .x_align_text(x_align)
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
