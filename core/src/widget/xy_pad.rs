//! Used for displaying and controlling a 2D point on a cartesian plane within a given range.

use {
    Color,
    Colorable,
    Borderable,
    FontSize,
    Labelable,
    Positionable,
    Scalar,
    Widget,
};
use num::Float;
use text;
use utils::{map_range, val_to_string};
use widget;


/// Used for displaying and controlling a 2D point on a cartesian plane within a given range.
///
/// Its reaction is triggered when the value is updated or if the mouse button is released while
/// the cursor is above the rectangle.
#[derive(WidgetCommon_)]
pub struct XYPad<'a, X, Y> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    x: X, min_x: X, max_x: X,
    y: Y, min_y: Y, max_y: Y,
    maybe_label: Option<&'a str>,
    style: Style,
    /// Indicates whether the XYPad will respond to user input.
    pub enabled: bool,
}

/// Unique graphical styling for the XYPad.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// The color of the XYPad's rectangle.
    #[conrod(default = "theme.shape_color")]
    pub color: Option<Color>,
    /// The width of the border surrounding the rectangle.
    #[conrod(default = "theme.border_width")]
    pub border: Option<Scalar>,
    /// The color of the surrounding rectangle border.
    #[conrod(default = "theme.border_color")]
    pub border_color: Option<Color>,
    /// The color of the XYPad's label and value label text.
    #[conrod(default = "theme.label_color")]
    pub label_color: Option<Color>,
    /// The font size for the XYPad's label.
    #[conrod(default = "theme.font_size_medium")]
    pub label_font_size: Option<FontSize>,
    /// The ID of the font used to display the label.
    #[conrod(default = "theme.font_id")]
    pub label_font_id: Option<Option<text::font::Id>>,
    /// The font size for the XYPad's *value* label.
    #[conrod(default = "14")]
    pub value_font_size: Option<FontSize>,
    /// The thickness of the XYPad's crosshair lines.
    #[conrod(default = "2.0")]
    pub line_thickness: Option<Scalar>,
}

widget_ids! {
    struct Ids {
        rectangle,
        label,
        h_line,
        v_line,
        value_label,
    }
}

/// The state of the XYPad.
pub struct State {
    ids: Ids,
}


impl<'a, X, Y> XYPad<'a, X, Y> {

    /// Build a new XYPad widget.
    pub fn new(x_val: X, min_x: X, max_x: X, y_val: Y, min_y: Y, max_y: Y) -> Self {
        XYPad {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            x: x_val, min_x: min_x, max_x: max_x,
            y: y_val, min_y: min_y, max_y: max_y,
            maybe_label: None,
            enabled: true,
        }
    }

    /// Specify the font used for displaying the label.
    pub fn label_font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.label_font_id = Some(Some(font_id));
        self
    }

    builder_methods!{
        pub line_thickness { style.line_thickness = Some(Scalar) }
        pub value_font_size { style.value_font_size = Some(FontSize) }
        pub enabled { enabled = bool }
    }

}

impl<'a, X, Y> Widget for XYPad<'a, X, Y>
    where X: Float + ToString + ::std::fmt::Debug + ::std::any::Any,
          Y: Float + ToString + ::std::fmt::Debug + ::std::any::Any,
{
    type State = State;
    type Style = Style;
    type Event = Option<(X, Y)>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the XYPad's cached state.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        use position::{Direction, Edge};

        let widget::UpdateArgs { id, state, rect, style, ui, .. } = args;
        let XYPad {
            x, min_x, max_x,
            y, min_y, max_y,
            maybe_label,
            ..
        } = self;

        let border = style.border(ui.theme());
        let inner_rect = rect.pad(border);

        let mut new_x = x;
        let mut new_y = y;
        if let Some(mouse) = ui.widget_input(id).mouse() {
            if mouse.buttons.left().is_down() {
                let mouse_abs_xy = mouse.abs_xy();
                let clamped_x = inner_rect.x.clamp_value(mouse_abs_xy[0]);
                let clamped_y = inner_rect.y.clamp_value(mouse_abs_xy[1]);
                let (l, r, b, t) = inner_rect.l_r_b_t();
                new_x = map_range(clamped_x, l, r, min_x, max_x);
                new_y = map_range(clamped_y, b, t, min_y, max_y);
            }
        }

        // If the value across either axis has changed, produce an event.
        let event = if x != new_x || y != new_y {
            Some((new_x, new_y))
        } else {
            None
        };

        let interaction_color = |ui: &::ui::UiCell, color: Color|
            ui.widget_input(id).mouse()
                .map(|mouse| if mouse.buttons.left().is_down() {
                    color.clicked()
                } else {
                    color.highlighted()
                })
                .unwrap_or(color);

        // The backdrop **BorderedRectangle** widget.
        let dim = rect.dim();
        let color = interaction_color(&ui, style.color(ui.theme()));
        let border = style.border(ui.theme());
        let border_color = style.border_color(ui.theme());
        widget::BorderedRectangle::new(dim)
            .middle_of(id)
            .graphics_for(id)
            .color(color)
            .border(border)
            .border_color(border_color)
            .set(state.ids.rectangle, ui);

        // Label **Text** widget.
        let label_color = style.label_color(ui.theme());
        let font_id = style.label_font_id(&ui.theme).or(ui.fonts.ids().next());
        if let Some(label) = maybe_label {
            let label_font_size = style.label_font_size(ui.theme());
            widget::Text::new(label)
                .and_then(font_id, widget::Text::font_id)
                .middle_of(state.ids.rectangle)
                .graphics_for(id)
                .color(label_color)
                .font_size(label_font_size)
                .set(state.ids.label, ui);
        }

        // Crosshair **Line** widgets.
        let (w, h) = inner_rect.w_h();
        let half_w = w / 2.0;
        let half_h = h / 2.0;
        let v_line_x = map_range(new_x, min_x, max_x, -half_w, half_w);
        let h_line_y = map_range(new_y, min_y, max_y, -half_h, half_h);
        let thickness = style.line_thickness(ui.theme());
        let line_color = label_color.with_alpha(1.0);

        let v_line_start = [0.0, -half_h];
        let v_line_end = [0.0, half_h];
        widget::Line::centred(v_line_start, v_line_end)
            .color(line_color)
            .thickness(thickness)
            .x_y_relative_to(id, v_line_x, 0.0)
            .graphics_for(id)
            .parent(id)
            .set(state.ids.v_line, ui);

        let h_line_start = [-half_w, 0.0];
        let h_line_end = [half_w, 0.0];
        widget::Line::centred(h_line_start, h_line_end)
            .color(line_color)
            .thickness(thickness)
            .x_y_relative_to(id, 0.0, h_line_y)
            .graphics_for(id)
            .parent(id)
            .set(state.ids.h_line, ui);

        // Crosshair value label **Text** widget.
        let x_string = val_to_string(new_x, max_x, max_x - min_x, rect.w() as usize);
        let y_string = val_to_string(new_y, max_y, max_y - min_y, rect.h() as usize);
        let value_string = format!("{}, {}", x_string, y_string);
        let cross_hair_xy = [inner_rect.x() + v_line_x, inner_rect.y() + h_line_y];
        const VALUE_TEXT_PAD: f64 = 5.0;
        let x_direction = match inner_rect.x.closest_edge(cross_hair_xy[0]) {
            Edge::End => Direction::Backwards,
            Edge::Start => Direction::Forwards,
        };
        let y_direction = match inner_rect.y.closest_edge(cross_hair_xy[1]) {
            Edge::End => Direction::Backwards,
            Edge::Start => Direction::Forwards,
        };
        let value_font_size = style.value_font_size(ui.theme());
        widget::Text::new(&value_string)
            .and_then(font_id, widget::Text::font_id)
            .x_direction_from(state.ids.v_line, x_direction, VALUE_TEXT_PAD)
            .y_direction_from(state.ids.h_line, y_direction, VALUE_TEXT_PAD)
            .color(line_color)
            .graphics_for(id)
            .parent(id)
            .font_size(value_font_size)
            .set(state.ids.value_label, ui);

        event
    }

}


impl<'a, X, Y> Colorable for XYPad<'a, X, Y> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, X, Y> Borderable for XYPad<'a, X, Y> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a, X, Y> Labelable<'a> for XYPad<'a, X, Y> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
