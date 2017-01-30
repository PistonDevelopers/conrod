//! The `Button` widget and related items.

use {Color, Colorable, FontSize, Borderable, Labelable, Positionable, Sizeable, UiCell, Widget};
use image;
use position::{Align, Rect, Scalar};
use text;
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
        /// The label's alignment over the *x* axis.
        - label_x_align: Align { Align::Middle }
        /// The ID of the font used to display the label.
        - label_font_id: Option<text::font::Id> { theme.font_id }
    }
}

widget_ids! {
    /// Identifiers for a "flat" button.
    #[allow(missing_docs, missing_copy_implementations)]
    pub struct FlatIds {
        rectangle,
        label,
    }
}

widget_ids! {
    /// Identifiers for an image button.
    #[allow(missing_docs, missing_copy_implementations)]
    pub struct ImageIds {
        rectangle,
        label,
        image,
    }
}

/// The `Button` simply displays a flat color.
#[derive(Copy, Clone)]
pub struct Flat;

/// The `Button` displays an `Image` on top.
#[derive(Copy, Clone)]
pub struct Image {
    /// The id of the `Image` to be used.
    pub image_id: image::Id,
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
    pub fn image(image_id: image::Id) -> Self {
        let image = Image {
            image_id: image_id,
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

    /// Align the label to the mid-left of the `Button`'s surface.
    pub fn align_label_left(mut self) -> Self {
        self.style.label_x_align = Some(Align::Start);
        self
    }

    /// Align the label to the mid-left of the `Button`'s surface.
    ///
    /// This is the default label alignment.
    pub fn align_label_middle(mut self) -> Self {
        self.style.label_x_align = Some(Align::Middle);
        self
    }

    /// Align the label to the mid-left of the `Button`'s surface.
    pub fn align_label_right(mut self) -> Self {
        self.style.label_x_align = Some(Align::End);
        self
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

    /// Specify the font used for displaying the label.
    pub fn label_font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.label_font_id = Some(Some(font_id));
        self
    }

    builder_methods!{
        pub enabled { enabled = bool }
    }
}


impl<'a> Widget for Button<'a, Flat> {
    type State = FlatIds;
    type Style = Style;
    type Event = TimesClicked;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        FlatIds::new(id_gen)
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the Button.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, style, rect, ui, .. } = args;
        let Button { maybe_label, .. } = self;

        let (color, times_clicked) = color_and_times_clicked(id, &style, ui);

        bordered_rectangle(id, state.rectangle, rect, color, style, ui);

        // Label widget.
        if let Some(l) = maybe_label {
            label(id, state.label, state.rectangle, l, style, ui);
        }

        TimesClicked(times_clicked)
    }

}

impl<'a> Widget for Button<'a, Image> {
    type State = ImageIds;
    type Style = Style;
    type Event = TimesClicked;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        ImageIds::new(id_gen)
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the Button.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, style, rect, ui, .. } = args;
        let Button { show, maybe_label, .. } = self;

        let (color, times_clicked) = color_and_times_clicked(id, &style, ui);

        bordered_rectangle(id, state.rectangle, rect, color, style, ui);

        // Instantiate the image.
        let Image { image_id, src_rect, color } = show;
        let mut image = widget::Image::new(image_id)
            .middle_of(id)
            .wh_of(id)
            .parent(id)
            .graphics_for(id);
        image.src_rect = src_rect;
        image.style.maybe_color = match color {
            ImageColor::Normal(color) => Some(Some(color)),
            ImageColor::WithFeedback(color) =>
                ui.widget_input(id).mouse()
                    .map(|mouse| if mouse.buttons.left().is_down() {
                        Some(color.clicked())
                    } else {
                        Some(color.highlighted())
                    })
                    .or(Some(Some(color))),
            ImageColor::None => None,
        };
        image.set(state.image, ui);

        if let Some(s) = maybe_label {
            label(id, state.label, state.rectangle, s, style, ui);
        }

        TimesClicked(times_clicked)
    }

}


fn color_and_times_clicked(button_id: widget::Id, style: &Style, ui: &UiCell) -> (Color, u16) {
    let input = ui.widget_input(button_id);
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
}

fn bordered_rectangle(button_id: widget::Id, rectangle_id: widget::Id,
                      rect: Rect, color: Color, style: &Style, ui: &mut UiCell)
{
    // BorderedRectangle widget.
    let dim = rect.dim();
    let border = style.border(&ui.theme);
    let border_color = style.border_color(&ui.theme);
    widget::BorderedRectangle::new(dim)
        .middle_of(button_id)
        .graphics_for(button_id)
        .color(color)
        .border(border)
        .border_color(border_color)
        .set(rectangle_id, ui);
}

fn label(button_id: widget::Id, label_id: widget::Id, rectangle_id: widget::Id,
         label: &str, style: &Style, ui: &mut UiCell)
{
    let color = style.label_color(&ui.theme);
    let font_size = style.label_font_size(&ui.theme);
    let align = style.label_x_align(&ui.theme);
    let font_id = style.label_font_id(&ui.theme).or(ui.fonts.ids().next());
    widget::Text::new(label)
        .and_then(font_id, widget::Text::font_id)
        .and(|b| match align {
            Align::Start =>
                b.mid_left_with_margin_on(rectangle_id, font_size as Scalar),
            Align::Middle =>
                b.middle_of(rectangle_id),
            Align::End =>
                b.mid_right_with_margin_on(rectangle_id, font_size as Scalar),
        })
        .parent(button_id)
        .graphics_for(button_id)
        .color(color)
        .font_size(font_size)
        .set(label_id, ui);
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
