//! A widget for selecting a single value along some linear range.

use {Color, Colorable, FontSize, Borderable, Labelable, Positionable, Widget};
use num::{Float, NumCast, ToPrimitive};
use position::{Padding, Range, Rect, Scalar};
use text;
use widget;
use widget::triangles::Triangle;


/// Linear value selection.
///
/// If the slider's width is greater than it's height, it will automatically become a horizontal
/// slider, otherwise it will be a vertical slider.
///
/// Its reaction is triggered if the value is updated or if the mouse button is released while
/// the cursor is above the rectangle.
#[derive(WidgetCommon_)]
pub struct Slider<'a, T> {
    #[conrod(common_builder)]
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

/// Graphical styling unique to the Slider widget.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// The color of the slidable rectangle.
    #[conrod(default = "theme.shape_color")]
    pub color: Option<Color>,
    /// The length of the border around the edges of the slidable rectangle.
    #[conrod(default = "theme.border_width")]
    pub border: Option<Scalar>,
    /// The color of the Slider's border.
    #[conrod(default = "theme.border_color")]
    pub border_color: Option<Color>,
    /// The color of the Slider's label.
    #[conrod(default = "theme.label_color")]
    pub label_color: Option<Color>,
    /// The font-size for the Slider's label.
    #[conrod(default = "theme.font_size_medium")]
    pub label_font_size: Option<FontSize>,
    /// The ID of the font used to display the label.
    #[conrod(default = "theme.font_id")]
    pub label_font_id: Option<Option<text::font::Id>>,
}

widget_ids! {
    struct Ids {
        triangles,
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
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            value: value,
            min: min,
            max: max,
            skew: 1.0,
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

        // The **Rectangle** for the adjustable slider.
        let value_perc = map_range(new_value, min, max, 0.0, 1.0);
        let unskewed_perc = value_perc.powf(1.0 / skew as f64);
        let (slider_rect, blank_rect) = if is_horizontal {
            let left = inner_rect.x.start;
            let slider = map_range(unskewed_perc, 0.0, 1.0, left, inner_rect.x.end);
            let right = inner_rect.x.end;
            let y = inner_rect.y;
            let slider_rect = Rect { x: Range::new(left, slider), y: y };
            let blank_rect = Rect { x: Range::new(slider, right), y: y };
            (slider_rect, blank_rect)
        } else {
            let bottom = inner_rect.y.start;
            let slider = map_range(unskewed_perc, 0.0, 1.0, bottom, inner_rect.y.end);
            let top = inner_rect.y.end;
            let x = inner_rect.x;
            let slider_rect = Rect { x: x, y: Range::new(bottom, slider) };
            let blank_rect = Rect { x: x, y: Range::new(slider, top) };
            (slider_rect, blank_rect)
        };

        let border_triangles = widget::bordered_rectangle::border_triangles(rect, border);
        let (a, b) = widget::rectangle::triangles(slider_rect);
        let slider_triangles = [a, b];
        let (a, b) = widget::rectangle::triangles(blank_rect);
        let blank_triangles = [a, b];

        let border_color = interaction_color(ui, style.border_color(ui.theme())).to_rgb();
        let color = interaction_color(ui, style.color(ui.theme())).to_rgb();

        // The border and blank triangles are the same color.
        let border_colored_triangles = border_triangles
            .as_ref()
            .into_iter()
            .flat_map(|tris| tris.iter().cloned())
            .chain(blank_triangles.iter().cloned())
            .map(|Triangle(ps)| Triangle([
                 (ps[0], border_color),
                 (ps[1], border_color),
                 (ps[2], border_color)
            ]));

        // Color the slider triangles.
        let slider_colored_triangles = slider_triangles
            .iter()
            .cloned()
            .map(|Triangle(ps)| Triangle([(ps[0], color), (ps[1], color), (ps[2], color)]));

        // Chain all triangles together into a single iterator.
        let triangles = border_colored_triangles.chain(slider_colored_triangles);

        widget::Triangles::multi_color(triangles)
            .with_bounding_rect(rect)
            .graphics_for(id)
            .parent(id)
            .set(state.ids.triangles, ui);

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
