use {
    CharacterCache,
    Color,
    Colorable,
    Dimension,
    FontSize,
    Frameable,
    FramedRectangle,
    FramedRectangleStyle,
    Labelable,
    Mouse,
    Positionable,
    NodeIndex,
    Scalar,
    Sizeable,
    Text,
    TextStyle,
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
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State {
    interaction: Interaction,
    maybe_rectangle_idx: Option<NodeIndex>,
    maybe_label_idx: Option<NodeIndex>,
}

/// Unique styling for the **TitleBar** widget.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    /// Shape styling for the rectangle.
    pub framed_rectangle: FramedRectangleStyle,
    /// Styling for the label.
    pub text: TextStyle,
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


impl Style {

    /// A new default Style.
    pub fn new() -> Self {
        Style {
            framed_rectangle: FramedRectangleStyle::new(),
            text: TextStyle::new(),
        }
    }

}


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
        }.width_of(idx).mid_top_of(idx)
    }

    /// Pass some styling for the **TitleBar**'s **Label**.
    pub fn label_style(mut self, style: TextStyle) -> Self {
        self.style.text = style;
        self
    }

    /// Pass some styling for the **TitleBar**'s **FramedRectangle**.
    pub fn rect_style(mut self, style: FramedRectangleStyle) -> Self {
        self.style.framed_rectangle = style;
        self
    }

    /// Pass the title bar some function to call upon interaction changes.
    pub fn react(mut self, f: F) -> Self {
        self.maybe_react = Some(f);
        self
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
            maybe_rectangle_idx: None,
            maybe_label_idx: None,
            interaction: Interaction::Normal,
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn default_y_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        let font_size = self.style.text.font_size(&ui.theme);
        let h = calc_height(font_size);
        Dimension::Absolute(h)
    }

    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let TitleBar { label, maybe_react, .. } = self;

        // Check whether or not a new interaction has occurred.
        let new_interaction = match ui.input().maybe_mouse {
            None => Interaction::Normal,
            Some(mouse) => {
                let is_over = rect.is_over(mouse.xy);
                get_new_interaction(is_over, state.view().interaction, mouse)
            },
        };

        // FramedRectangle widget.
        let rectangle_idx = state.view().maybe_rectangle_idx
            .unwrap_or_else(|| ui.new_unique_node_index());
        let dim = rect.dim();
        FramedRectangle::new(dim)
            .with_style(style.framed_rectangle)
            .middle_of(idx)
            .graphics_for(idx)
            .set(rectangle_idx, &mut ui);

        // Label widget.
        let label_idx = state.view().maybe_label_idx
            .unwrap_or_else(|| ui.new_unique_node_index());
        Text::new(label)
            .with_style(style.text)
            .middle_of(rectangle_idx)
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

        if state.view().maybe_rectangle_idx != Some(rectangle_idx) {
            state.update(|state| state.maybe_rectangle_idx = Some(rectangle_idx));
        }

        if state.view().maybe_label_idx != Some(label_idx) {
            state.update(|state| state.maybe_label_idx = Some(label_idx));
        }
    }

}


impl<'a, F> Colorable for TitleBar<'a, F> {
    fn color(mut self, color: Color) -> Self {
        self.style.framed_rectangle.maybe_color = Some(color);
        self
    }
}

impl<'a, F> Frameable for TitleBar<'a, F> {
    fn frame(mut self, width: f64) -> Self {
        self.style.framed_rectangle.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.framed_rectangle.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, F> Labelable<'a> for TitleBar<'a, F> {
    fn label(mut self, text: &'a str) -> Self {
        self.label = text;
        self
    }

    fn label_color(mut self, color: Color) -> Self {
        self.style.text.maybe_color = Some(color);
        self
    }

    fn label_font_size(mut self, size: FontSize) -> Self {
        self.style.text.maybe_font_size = Some(size);
        self
    }
}

