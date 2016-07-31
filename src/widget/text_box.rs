use {
    Align,
    Color,
    Colorable,
    FontSize,
    Frameable,
    FramedRectangle,
    IndexSlot,
    Positionable,
    Range,
    Rect,
    Scalar,
    Sizeable,
    TextEdit,
    Widget,
};
use event;
use input;
use widget;

/// A widget for displaying and mutating a small, one-line field of text, given by the user in the
/// form of a `String`.
///
/// It's reaction is triggered upon pressing of the `Enter`/`Return` key.
pub struct TextBox<'a, F> {
    common: widget::CommonBuilder,
    text: &'a mut String,
    /// The reaction for the TextBox.
    ///
    /// If `Some`, this will be triggered upon pressing of the `Enter`/`Return` key.
    pub maybe_react: Option<F>,
    style: Style,
}

widget_style!{
    /// Unique graphical styling for the TextBox.
    style Style {
        /// The length of the gap between the bounding rectangle's frame and the edge of the text.
        - text_padding: Scalar { 5.0 }
        /// Color of the rectangle behind the text.
        ///
        /// If you don't want to see the rectangle, either set the color with a zeroed alpha or use
        /// the `TextEdit` widget directly.
        - color: Color { theme.shape_color }
        /// The width of the bounding `FramedRectangle` frame.
        - frame: Scalar { theme.frame_width }
        /// The color of the `FramedRecangle`'s frame.
        - frame_color: Color { theme.frame_color }
        /// The color of the `TextEdit` widget.
        - text_color: Color { theme.label_color }
        /// The font size for the text.
        - font_size: FontSize { theme.font_size_medium }
        /// The horizontal alignment of the text.
        - x_align: Align { Align::Start }
    }
}

/// The `State` of the `TextBox` widget that will be cached within the `Ui`.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    text_edit_idx: IndexSlot,
    rectangle_idx: IndexSlot,
}

impl<'a, F> TextBox<'a, F> {

    /// Construct a TextBox widget.
    pub fn new(text: &'a mut String) -> Self {
        TextBox {
            common: widget::CommonBuilder::new(),
            text: text,
            maybe_react: None,
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

    builder_methods!{
        pub text_color { style.text_color = Some(Color) }
        pub font_size { style.font_size = Some(FontSize) }
        pub react { maybe_react = Some(F) }
        pub x_align_text { style.x_align = Some(Align) }
        pub pad_text { style.text_padding = Some(Scalar) }
    }

}

impl<'a, F> Widget for TextBox<'a, F>
    where F: FnMut(&mut String),
{
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self) -> State {
        State {
            text_edit_idx: IndexSlot::new(),
            rectangle_idx: IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the TextEdit.
    fn update(self, args: widget::UpdateArgs<Self>) {
        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let TextBox { text, mut maybe_react, .. } = self;

        let font_size = style.font_size(ui.theme());
        let frame = style.frame(ui.theme());
        let text_padding = style.text_padding(ui.theme());
        let x_align = style.x_align(ui.theme());

        let text_rect = {
            let w = rect.x.pad(frame + text_padding).len();
            let h = font_size as Scalar + 1.0;
            let x = Range::new(0.0, w).align_middle_of(rect.x);
            let y = Range::new(0.0, h).align_middle_of(rect.y);
            Rect { x: x, y: y }
        };

        let rectangle_idx = state.rectangle_idx.get(&mut ui);
        let color = style.color(ui.theme());
        let frame_color = style.frame_color(ui.theme());
        FramedRectangle::new(rect.dim())
            .xy(rect.xy())
            .graphics_for(idx)
            .parent(idx)
            .frame(frame)
            .color(color)
            .frame_color(frame_color)
            .set(rectangle_idx, &mut ui);

        let text_edit_idx = state.text_edit_idx.get(&mut ui);
        let text_color = style.text_color(ui.theme());
        TextEdit::new(text)
            .wh(text_rect.dim())
            .xy(text_rect.xy())
            .font_size(font_size)
            .color(text_color)
            .x_align_text(x_align)
            .parent(idx)
            .react(|_text: &mut String| {})
            .set(text_edit_idx, &mut ui);

        // React to any `Enter`/`Return` presses.
        //
        // TODO: We should be doing this via the `TextEdit` widget.
        for widget_event in ui.widget_input(text_edit_idx).events() {
            match widget_event {
                event::Widget::Press(press) => match press.button {
                    event::Button::Keyboard(key) => match key {
                        input::Key::Return => {
                            if let Some(mut react) = maybe_react.take() {
                                react(text);
                            }
                        },
                        _ => ()
                    },
                    _ => (),
                },
                _ => (),
            }
        }
    }

}

impl<'a, F> Frameable for TextBox<'a, F> {
    builder_methods!{
        frame { style.frame = Some(Scalar) }
        frame_color { style.frame_color = Some(Color) }
    }
}

impl<'a, F> Colorable for TextBox<'a, F> {
    builder_method!(color { style.color = Some(Color) });
}
