use {CharacterCache, LabelStyle, FramedRectangleStyle, Scalar, Theme};
use color::{Color, Colorable};
use elmesque::Element;
use frame::Frameable;
use position::{Dimensions, Sizeable};
use widget::{self, Widget};


/// A simple title bar widget that automatically sizes itself to the top of some other widget.
pub struct TitleBar<'a> {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// Unique styling for the **FramedRectangle**.
    pub style: Style,
    /// A label displayed in the middle of the TitleBar.
    pub label: &'a str,
}

/// Unique state for the **TitleBar** widget.
#[derive(Copy, Clone, Debug)]
pub struct State {
    maybe_rectangle_idx: Option<NodeIndex>,
    maybe_label_idx: Option<NodeIndex>,
}

/// Unique styling for the **TitleBar** widget.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    /// Shape styling for the rectangle.
    pub framed_rectangle_style: FramedRectangleStyle,
    /// Styling for the label.
    pub label_style: LabelStyle,
}

/// The padding between the edge of the title bar and the title bar's label.
///
/// This is used to determine the size of the TitleBar.
const LABEL_PADDING: f64 = 4.0;


impl Style {

    /// A new default Style.
    pub fn new() -> Self {
        Style {
            framed_rectangle_style: FramedRectangleStyle::new(),
            label_style: LabelStyle::new(),
        }
    }

}


impl<'a> TitleBar<'a> {

    /// Construct a new TitleBar widget and attach it to the widget at the given index.
    pub fn new<I>(label: &'a str, idx: I) -> Self
        where I: Into<widget::Index>
    {
        TitleBar {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
            label: label,
        }
    }

    /// Get the font size for the **TitleBar**'s label.
    pub fn font_size(&self, theme: &Theme) -> FontSize {
        self.label_style.font_size(theme)
    }

}


impl<'a> Widget for TitleBar<'a> {
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> &'static str {
        "Canvas"
    }

    fn init_state(&self) -> State {
        State {
            interaction: Interaction::Normal,
            time_last_clicked: precise_time_ns(),
            maybe_title_bar: None,
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn default_height(&self, theme: &Theme) -> Scalar {
        let font_size = self.style.font_size(theme);
        font_size as Scalar + LABEL_PADDING * 2.0;
    }

    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { idx, state, rect, style, ui, .. } = args;
        let TitleBar { label, .. } = self;

        // FramedRectangle widget.
        let rectangle_idx = state.view().maybe_rectangle_idx
            .unwrap_or_else(|| ui.new_unique_node_index());
        let dim = rect.dim();
        let mut rectangle = FramedRectangle::new(dim)
            .middle_of(idx)
            .picking_passthrough(true);
        rectangle.style = style.framed_rectangle_style;
        rectangle.set(rectangle_idx, &mut ui);

        // Label widget.
        let label_idx = state.view().maybe_label_idx
            .unwrap_or_else(|| ui.new_unique_node_index());
        let mut label = Label::new(label)
            .middle_of(rectangle_idx)
            .picking_passthrough(true);
        label.style = style.label_style;
        label.set(label_idx, &mut ui);

        // If the rectangle index has changed, update it.
        if state.view().maybe_rectangle_idx != Some(rectangle_idx) {
            state.update(|state| state.maybe_rectangle_idx = Some(rectangle_idx));
        }

        // If the label index has changed, update it.
        if state.view().maybe_label_idx != maybe_label_idx {
            state.update(|state| state.maybe_label_idx = maybe_label_idx);
        }
    }

}


