//! A widget for selecting a single value along some linear range.

use {
    Color,
    Colorable,
    FontSize,
    Borderable,
    Labelable,
    Padding,
    Positionable,
    Range,
    Rect,
    Scalar,
    Widget,
};
use num::{Float, NumCast, ToPrimitive};
use text;
use widget;


/// Linear value selection.
///
/// If the slider's width is greater than it's height, it will automatically become a horizontal
/// slider, otherwise it will be a vertical slider.
///
/// Its reaction is triggered if the value is updated or if the mouse button is released while
/// the cursor is above the rectangle.
pub struct Slider<'a, T> {
    common: widget::CommonBuilder,
    value: T,
    min: T,
    max: T,
    /// The amount in which the slider's display should be skewed.
    ///
    /// Higher skew amounts (above 1.0) will weight lower values.
    ///
    /// Lower skew amounts (below 1.0) will weight heigher values.
    ///
    /// All skew amounts should be greater than 0.0.
    pub skew: f32,
    maybe_label: Option<&'a str>,
    style: Style,
    /// Whether or not user input is enabled for the Slider.
    pub enabled: bool,
}

widget_style! {
    /// Graphical styling unique to the Slider widget.
    style Style {
        /// The color of the slidable rectangle.
        - color: Color { theme.shape_color }
        /// The length of the border around the edges of the slidable rectangle.
        - border: Scalar { theme.border_width }
        /// The color of the Slider's border.
        - border_color: Color { theme.border_color }
        /// The color of the Slider's label.
        - label_color: Color { theme.label_color }
        /// The font-size for the Slider's label.
        - label_font_size: FontSize { theme.font_size_medium }
        /// The ID of the font used to display the label.
        - label_font_id: Option<text::font::Id> { theme.font_id }
    }
}

widget_ids! {
    struct Ids {
        border,
        slider,
        label,
    }
}

/// Represents the state of the Slider widget.
pub struct State {
    ids: Ids,
}

impl<'a, T> Slider<'a, T> {

    /// Construct a new Slider widget.
    pub fn new(value: T, min: T, max: T) -> Self {
        Slider {
            common: widget::CommonBuilder::new(),
            value: value,
            min: min,
            max: max,
            skew: 1.0,
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
        pub skew { skew = f32 }
        pub enabled { enabled = bool }
    }

}

impl<'a, T> Widget for Slider<'a, T>
    where T: Float + NumCast + ToPrimitive,
{
    type State = State;
    type Style = Style;
    type Event = Option<T>;

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

    fn kid_area(&self, args: widget::KidAreaArgs<Self>) -> widget::KidArea {
        const LABEL_PADDING: Scalar = 10.0;
        widget::KidArea {
            rect: args.rect,
            pad: Padding {
                x: Range::new(LABEL_PADDING, LABEL_PADDING),
                y: Range::new(LABEL_PADDING, LABEL_PADDING),
            },
        }
    }

    /// Update the state of the Slider.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        use utils::{clamp, map_range, value_from_perc};

        let widget::UpdateArgs { id, state, rect, style, ui, .. } = args;
        let Slider { value, min, max, skew, maybe_label, .. } = self;

        let is_horizontal = rect.w() > rect.h();
        let border = style.border(ui.theme());
        let inner_rect = rect.pad(border);

        let new_value = if let Some(mouse) = ui.widget_input(id).mouse() {
            if mouse.buttons.left().is_down() {
                let mouse_abs_xy = mouse.abs_xy();
                if is_horizontal {
                    // Horizontal.
                    let inner_w = inner_rect.w();
                    let slider_w = mouse_abs_xy[0] - inner_rect.x.start;
                    let perc = clamp(slider_w, 0.0, inner_w) / inner_w;
                    let skewed_perc = (perc).powf(skew as f64);
                    let w_perc = skewed_perc;
                    value_from_perc(w_perc as f32, min, max)
                } else {
                    // Vertical.
                    let inner_h = inner_rect.h();
                    let slider_h = mouse_abs_xy[1] - inner_rect.y.start;
                    let perc = clamp(slider_h, 0.0, inner_h) / inner_h;
                    let skewed_perc = (perc).powf(skew as f64);
                    let h_perc = skewed_perc;
                    value_from_perc(h_perc as f32, min, max)
                }
            } else {
                value
            }
        } else {
            value
        };

        // The **Rectangle** for the border.
        let interaction_color = |ui: &::ui::UiCell, color: Color|
            ui.widget_input(id).mouse()
                .map(|mouse| if mouse.buttons.left().is_down() {
                    color.clicked()
                } else {
                    color.highlighted()
                })
                .unwrap_or(color);

        let border_color = interaction_color(ui, style.border_color(ui.theme()));
        widget::Rectangle::fill(rect.dim())
            .middle_of(id)
            .graphics_for(id)
            .color(border_color)
            .set(state.ids.border, ui);

        // The **Rectangle** for the adjustable slider.
        let slider_rect = if is_horizontal {
            let left = inner_rect.x.start;
            let right = map_range(new_value, min, max, left, inner_rect.x.end);
            let x = Range::new(left, right);
            let y = inner_rect.y;
            Rect { x: x, y: y }
        } else {
            let bottom = inner_rect.y.start;
            let top = map_range(new_value, min, max, bottom, inner_rect.y.end);
            let x = inner_rect.x;
            let y = Range::new(bottom, top);
            Rect { x: x, y: y }
        };
        let color = interaction_color(ui, style.color(ui.theme()));
        let slider_xy_offset = [slider_rect.x() - rect.x(), slider_rect.y() - rect.y()];
        widget::Rectangle::fill(slider_rect.dim())
            .xy_relative_to(id, slider_xy_offset)
            .graphics_for(id)
            .parent(id)
            .color(color)
            .set(state.ids.slider, ui);

        // The **Text** for the slider's label (if it has one).
        if let Some(label) = maybe_label {
            let label_color = style.label_color(ui.theme());
            let font_size = style.label_font_size(ui.theme());
            let font_id = style.label_font_id(&ui.theme).or(ui.fonts.ids().next());
            //const TEXT_PADDING: f64 = 10.0;
            widget::Text::new(label)
                .and_then(font_id, widget::Text::font_id)
                .and(|text| if is_horizontal { text.mid_left_of(id) }
                            else { text.mid_bottom_of(id) })
                .graphics_for(id)
                .color(label_color)
                .font_size(font_size)
                .set(state.ids.label, ui);
        }

        // If the value has just changed, return the new value.
        if value != new_value { Some(new_value) } else { None }
    }

}


impl<'a, T> Colorable for Slider<'a, T> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, T> Borderable for Slider<'a, T> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a, T> Labelable<'a> for Slider<'a, T> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
