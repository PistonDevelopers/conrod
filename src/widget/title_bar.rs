use {
    Align,
    Backend,
    Color,
    Colorable,
    Dimension,
    FontSize,
    Frameable,
    FramedRectangle,
    IndexSlot,
    Labelable,
    Mouse,
    Positionable,
    Scalar,
    Sizeable,
    Text,
    TextWrap,
    Ui,
};
use widget::{self, Widget};


/// A simple title bar widget that automatically sizes itself to the top of some other widget.
pub struct TitleBar<'a, F> {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// Unique styling for the **FramedRectangle**.
    pub style: Style,
    /// A label displayed in the middle of the TitleBar.
    pub label: &'a str,
    /// Some function used to react to interactions with the TitleBar.
    pub maybe_react: Option<F>,
}

/// Unique state for the **TitleBar** widget.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    interaction: Interaction,
    rectangle_idx: IndexSlot,
    label_idx: IndexSlot,
}

widget_style!{
    KIND;
    /// Unique styling for the **TitleBar** widget.
    style Style {
        /// The color of the TitleBar's rectangle surface.
        - color: Color { theme.background_color }
        /// The width of the frame surrounding the TitleBar's rectangle.
        - frame: Scalar { theme.frame_width }
        /// The color of the TitleBar's frame.
        - frame_color: Color { theme.frame_color }

        /// The color of the title bar's text.
        - text_color: Color { theme.label_color }
        /// The font size for the title bar's text.
        - font_size: FontSize { theme.font_size_medium }
        /// The way in which the title bar's text should wrap.
        - maybe_wrap: Option<TextWrap> { Some(TextWrap::Whitespace) }
        /// The distance between lines for multi-line title bar text.
        - line_spacing: Scalar { 1.0 }
        /// The horizontal alignment of the title bar text.
        - text_align: Align { Align::Middle }
    }
}

/// Some interaction with the **TitleBar**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Interaction {
    Normal,
    Highlighted,
    Clicked,
}

/// Unique kind for the widget type.
pub const KIND: widget::Kind = "TitleBar";

/// The padding between the edge of the title bar and the title bar's label.
///
/// This is used to determine the size of the TitleBar.
const LABEL_PADDING: f64 = 4.0;


/// Check the current state of the button.
fn get_new_interaction(is_over: bool, prev: Interaction, mouse: Mouse) -> Interaction {
    use mouse::ButtonPosition::{Down, Up};
    use self::Interaction::{Normal, Highlighted, Clicked};
    match (is_over, prev, mouse.left.position) {
        (true,  Normal,  Down) => Normal,
        (true,  _,       Down) => Clicked,
        (true,  _,       Up)   => Highlighted,
        (false, Clicked, Down) => Clicked,
        _                      => Normal,
    }
}


impl<'a, F> TitleBar<'a, F>
    where F: FnOnce(Interaction),
{

    /// Construct a new TitleBar widget and attach it to the widget at the given index.
    pub fn new<I>(label: &'a str, idx: I) -> Self
        where I: Into<widget::Index> + Copy,
    {
        TitleBar {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
            label: label,
            maybe_react: None,
        }.w_of(idx).mid_top_of(idx)
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

    builder_methods!{
        pub line_spacing { style.line_spacing = Some(Scalar) }
        pub react { maybe_react = Some(F) }
    }

}


/// Calculate the default height for the **TitleBar**'s rect.
pub fn calc_height(font_size: FontSize) -> Scalar {
    font_size as Scalar + LABEL_PADDING * 2.0
}


impl<'a, F> Widget for TitleBar<'a, F>
    where F: FnOnce(Interaction),
{
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
            rectangle_idx: IndexSlot::new(),
            label_idx: IndexSlot::new(),
            interaction: Interaction::Normal,
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn default_y_dimension<B: Backend>(&self, ui: &Ui<B>) -> Dimension {
        let font_size = self.style.font_size(&ui.theme);
        let h = calc_height(font_size);
        Dimension::Absolute(h)
    }

    fn update<B: Backend>(self, args: widget::UpdateArgs<Self, B>) {
        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let TitleBar { label, maybe_react, .. } = self;

        // Check whether or not a new interaction has occurred.
        let new_interaction = match ui.input(idx).maybe_mouse {
            None => Interaction::Normal,
            Some(mouse) => {
                let is_over = rect.is_over(mouse.xy);
                get_new_interaction(is_over, state.view().interaction, mouse)
            },
        };

        // FramedRectangle widget.
        let rectangle_idx = state.view().rectangle_idx.get(&mut ui);
        let dim = rect.dim();
        let color = style.color(ui.theme());
        let frame = style.frame(ui.theme());
        let frame_color = style.frame_color(ui.theme());
        FramedRectangle::new(dim)
            .color(color)
            .frame(frame)
            .frame_color(frame_color)
            .middle_of(idx)
            .graphics_for(idx)
            .set(rectangle_idx, &mut ui);

        // Label widget.
        let label_idx = state.view().label_idx.get(&mut ui);
        let text_color = style.text_color(ui.theme());
        let text_align = style.text_align(ui.theme());
        let font_size = style.font_size(ui.theme());
        let line_spacing = style.line_spacing(ui.theme());
        let maybe_wrap = style.maybe_wrap(ui.theme());
        Text::new(label)
            .and_mut(|text| {
                text.style.maybe_wrap = Some(maybe_wrap);
                text.style.text_align = Some(text_align);
            })
            .padded_w_of(rectangle_idx, frame)
            .middle_of(rectangle_idx)
            .color(text_color)
            .font_size(font_size)
            .line_spacing(line_spacing)
            .graphics_for(idx)
            .set(label_idx, &mut ui);

        if state.view().interaction != new_interaction {
            if let Some(react) = maybe_react {
                // If there's been some change in interaction and we have some react function, call
                // the react function with our new interaction.
                react(new_interaction);
            }
            state.update(|state| state.interaction = new_interaction);
        }
    }

}


impl<'a, F> Colorable for TitleBar<'a, F> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, F> Frameable for TitleBar<'a, F> {
    builder_methods!{
        frame { style.frame = Some(Scalar) }
        frame_color { style.frame_color = Some(Color) }
    }
}

impl<'a, F> Labelable<'a> for TitleBar<'a, F> {
    builder_methods!{
        label { label = &'a str }
        label_color { style.text_color = Some(Color) }
        label_font_size { style.font_size = Some(FontSize) }
    }
}

