//! The `Button` widget and related items.

use {
    Color,
    Colorable,
    FontSize,
    Borderable,
    Labelable,
    Rect,
    Positionable,
    Scalar,
    UiCell,
    Widget,
};
use widget;


/// A pressable button widget whose reaction is triggered upon release.
#[derive(Clone)]
pub struct Button<'a, S> {
    common: widget::CommonBuilder,
    maybe_label: Option<&'a str>,
    /// Whether the `Button` is a `Flat` color or an `Image`.
    pub show: S,
    /// Unique styling parameters for the Button.
    pub style: Style,
    /// Whether or not user input is enabled.
    enabled: bool,
}

widget_style!{
    /// Unique styling for the Button.
    style Style {
        /// Color of the Button's pressable area.
        - color: Color { theme.shape_color }
        /// Width of the border surrounding the button
        - border: Scalar { theme.border_width }
        /// The color of the border.
        - border_color: Color { theme.border_color }
        /// The color of the Button's label.
        - label_color: Color { theme.label_color }
        /// The font size of the Button's label.
        - label_font_size: FontSize { theme.font_size_medium }
    }
}

/// The style of the `Button`, either `Flat` or `Image`.
pub trait Show: Sized {
    /// Display the unique styling of the `Button`.
    fn show(self, _button_idx: widget::Index, _ui: &mut UiCell) {}
}

/// The `Button` simply displays a flat color.
#[derive(Copy, Clone)]
pub struct Flat;

/// The `Button` displays an `Image` on top.
#[derive(Copy, Clone)]
pub struct Image {
    /// The index of the `Image` to be used.
    pub index: widget::Index,
    /// If `Some`, maps the image's luminance to this `Color`.
    pub color: ImageColor,
    /// The rectangular area of the original source image that should be displayed.
    pub src_rect: Option<Rect>,
}

/// The coloring of the `Image`.
#[derive(Copy, Clone, Debug)]
pub enum ImageColor {
    /// The image's luminance will be mapped to this color.
    Normal(Color),
    /// The image's luminance will be mapped to this color.
    ///
    /// The color will change slightly upon interaction to provide visual feedback.
    WithFeedback(Color),
    /// The image's regular color will be used.
    None,
}


/// Represents the state of the Button widget.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    rectangle_idx: widget::IndexSlot,
    label_idx: widget::IndexSlot,
}

/// The `Event` type yielded by the `Button` widget.
///
/// Represents the number of times that the `Button` has been clicked with the left mouse button
/// since the last update.
#[derive(Clone, Debug)]
#[allow(missing_copy_implementations)]
pub struct TimesClicked(pub u16);


impl TimesClicked {
    /// `true` if the `Button` was clicked one or more times.
    pub fn was_clicked(self) -> bool { self.0 > 0 }
}

impl Iterator for TimesClicked {
    type Item = ();
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 > 0 {
            self.0 -= 1;
            Some(())
        } else {
            None
        }
    }
}


impl<'a> Button<'a, Image> {

    /// Begin building a button displaying the given `Image` on top.
    pub fn image<I>(image_idx: I) -> Self
        where I: Into<widget::Index>,
    {
        let image = Image {
            index: image_idx.into(),
            src_rect: None,
            color: ImageColor::None,
        };
        Self::new_internal(image)
    }

    /// The rectangular area of the image that we wish to display.
    ///
    /// If this method is not called, the entire image will be used.
    pub fn source_rectangle(mut self, rect: Rect) -> Self {
        self.show.src_rect = Some(rect);
        self
    }

    /// Map the `Image`'s luminance to the given color.
    pub fn image_color(mut self, color: Color) -> Self {
        self.show.color = ImageColor::Normal(color);
        self
    }

    /// Map the `Image`'s luminance to the given color.
    ///
    /// The color will change slightly when the button is highlighted or clicked to give the user
    /// some visual feedback.
    pub fn image_color_with_feedback(mut self, color: Color) -> Self {
        self.show.color = ImageColor::WithFeedback(color);
        self
    }

}

impl<'a> Button<'a, Flat> {

    /// Begin building a flat-colored `Button` widget.
    pub fn new() -> Self {
        Self::new_internal(Flat)
    }

}


impl<'a, S> Button<'a, S> {

    /// Create a button context to be built upon.
    fn new_internal(show: S) -> Self {
        Button {
            common: widget::CommonBuilder::new(),
            show: show,
            maybe_label: None,
            style: Style::new(),
            enabled: true,
        }
    }

    builder_methods!{
        pub enabled { enabled = bool }
    }
}


impl<'a, S> Widget for Button<'a, S>
    where S: Show,
{
    type State = State;
    type Style = Style;
    type Event = TimesClicked;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self) -> State {
        State {
            rectangle_idx: widget::IndexSlot::new(),
            label_idx: widget::IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the Button.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { idx, state, style, rect, mut ui, .. } = args;
        let Button { show, maybe_label, .. } = self;

        let (color, times_clicked) = {
            let input = ui.widget_input(idx);
            let color = style.color(ui.theme());
            let color = input.mouse().map_or(color, |mouse| {
                if mouse.buttons.left().is_down() {
                    color.clicked()
                } else {
                    color.highlighted()
                }
            });
            let times_clicked = input.clicks().left().count() as u16;
            (color, times_clicked)
        };

        // BorderedRectangle widget.
        let rectangle_idx = state.rectangle_idx.get(&mut ui);
        let dim = rect.dim();
        let border = style.border(ui.theme());
        let border_color = style.border_color(ui.theme());
        widget::BorderedRectangle::new(dim)
            .middle_of(idx)
            .graphics_for(idx)
            .color(color)
            .border(border)
            .border_color(border_color)
            .set(rectangle_idx, &mut ui);

        // Label widget.
        if let Some(label) = maybe_label {
            let label_idx = state.label_idx.get(&mut ui);
            let color = style.label_color(ui.theme());
            let font_size = style.label_font_size(ui.theme());
            widget::Text::new(label)
                .middle_of(rectangle_idx)
                .graphics_for(idx)
                .color(color)
                .font_size(font_size)
                .set(label_idx, &mut ui);
        }

		// This instantiates the image widget if necessary.
        show.show(idx, &mut ui);

        TimesClicked(times_clicked)
    }

}


impl<'a, S> Colorable for Button<'a, S> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, S> Borderable for Button<'a, S> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a, S> Labelable<'a> for Button<'a, S> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}


impl Show for Flat {}

impl Show for Image {
    fn show(self, button_idx: widget::Index, ui: &mut UiCell) {
        let Image { index, src_rect, color } = self;
        let mut image = widget::Image::new().middle_of(button_idx).graphics_for(button_idx);
        image.src_rect = src_rect;
        image.style.maybe_color = match color {
            ImageColor::Normal(color) => Some(Some(color)),
            ImageColor::WithFeedback(color) =>
                ui.widget_input(button_idx).mouse()
                    .map(|mouse| if mouse.buttons.left().is_down() {
                        Some(color.clicked())
                    } else {
                        Some(color.highlighted())
                    })
                    .or(Some(Some(color))),
            ImageColor::None => None,
        };
        image.set(index, ui);
    }
}
