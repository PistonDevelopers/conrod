//! A button that allows for toggling boolean state.

use {
    Color,
    Colorable,
    FontSize,
    Borderable,
    Labelable,
    Positionable,
    Scalar,
    Widget,
};
use text;
use widget;


/// A pressable widget for toggling the state of a bool.
///
/// Like the Button widget, it's reaction is triggered upon release and will return the new bool
/// state.
///
/// Note that the Toggle will not mutate the bool for you, you should do this yourself within the
/// react function.
#[derive(Clone)]
pub struct Toggle<'a> {
    common: widget::CommonBuilder,
    value: bool,
    maybe_label: Option<&'a str>,
    style: Style,
    /// If true, will allow user inputs. If false, will disallow user inputs.
    pub enabled: bool,
}

widget_style! {
    /// Styling for the Toggle including coloring, bordering and labelling.
    style Style {
        /// Color of the Toggle's pressable area.
        - color: Color { theme.shape_color }
        /// The width of the rectangular border surrounding the Toggle.
        - border: Scalar { theme.border_width }
        /// The color of the Toggle's border.
        - border_color: Color { theme.border_color }
        /// The color of the Toggle's Text label.
        - label_color: Color { theme.label_color }
        /// The font size for the Toggle's Text label.
        - label_font_size: FontSize { theme.font_size_medium }
        /// The ID of the font used to display the label.
        - label_font_id: Option<text::font::Id> { theme.font_id }
    }
}

widget_ids! {
    struct Ids {
        rectangle,
        label,
    }
}

/// The state of the Toggle.
pub struct State {
    ids: Ids,
}

/// The `Event` type yielded by the `Toggle` widget.
///
/// Implements `Iterator` yielding a `bool` indicating the new state for each time the `Toggle` was
/// clicked with the left mouse button since the last update.
#[derive(Clone, Debug)]
#[allow(missing_copy_implementations)]
pub struct TimesClicked {
    state: bool,
    count: u16,
}


impl Iterator for TimesClicked {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            self.count -= 1;
            self.state = !self.state;
            Some(self.state)
        } else {
            None
        }
    }
}


impl<'a> Toggle<'a> {

    /// Construct a new Toggle widget.
    pub fn new(value: bool) -> Toggle<'a> {
        Toggle {
            common: widget::CommonBuilder::new(),
            maybe_label: None,
            value: value,
            style: Style::new(),
            enabled: true,
        }
    }

    /// Specify the font used for displaying the label.
    pub fn label_font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.label_font_id = Some(Some(font_id));
        self
    }

    builder_methods!{
        pub enabled { enabled = bool }
    }

}

impl<'a> Widget for Toggle<'a> {
    type State = State;
    type Style = Style;
    type Event = TimesClicked;

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

    /// Update the state of the Toggle.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, style, rect, mut ui, .. } = args;
        let Toggle { value, enabled, maybe_label, .. } = self;

        let times_clicked = TimesClicked {
            state: value,
            count: if enabled { ui.widget_input(id).clicks().left().count() as u16 } else { 0 },
        };

        // BorderedRectangle widget.
        let dim = rect.dim();
        let border = style.border(ui.theme());
        let color = {
            let color = style.color(ui.theme());
            let new_value = times_clicked.clone().last().unwrap_or(value);
            let color = if new_value { color } else { color.with_luminance(0.1) };
            match ui.widget_input(id).mouse() {
                Some(mouse) =>
                    if mouse.buttons.left().is_down() { color.clicked() }
                    else { color.highlighted() },
                None => color,
            }
        };
        let border_color = style.border_color(ui.theme());
        widget::BorderedRectangle::new(dim)
            .middle_of(id)
            .graphics_for(id)
            .color(color)
            .border(border)
            .border_color(border_color)
            .set(state.ids.rectangle, ui);

        // Label widget.
        if let Some(label) = maybe_label {
            let color = style.label_color(ui.theme());
            let font_size = style.label_font_size(ui.theme());
            let font_id = style.label_font_id(&ui.theme).or(ui.fonts.ids().next());
            widget::Text::new(label)
                .and_then(font_id, widget::Text::font_id)
                .middle_of(state.ids.rectangle)
                .graphics_for(id)
                .color(color)
                .font_size(font_size)
                .set(state.ids.label, ui);
        }

        times_clicked
    }
}


impl<'a> Colorable for Toggle<'a> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a> Borderable for Toggle<'a> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a> Labelable<'a> for Toggle<'a> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
