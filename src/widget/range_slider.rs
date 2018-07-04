//! A widget for specifying start and end values for some linear range.

use {Color, Colorable, FontSize, Borderable, Labelable, Positionable, Widget};
use num::Float;
use position::{Padding, Range, Rect, Scalar};
use text;
use utils;
use widget;

/// Linear range selection.
#[derive(WidgetCommon_)]
pub struct RangeSlider<'a, T> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    start: T,
    end: T,
    min: T,
    max: T,
    maybe_label: Option<&'a str>,
    style: Style,
    /// The amount in which the slider's display should be skewed.
    ///
    /// Higher skew amounts (above 1.0) will weight lower values.
    ///
    /// Lower skew amounts (below 1.0) will weight heigher values.
    ///
    /// All skew amounts should be greater than 0.0.
    ///
    /// By default, this value is `1.0` (no skew).
    pub skew: f32,
}

/// Graphical styling unique to the RangeSlider widget.
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
        border,
        slider,
        label,
    }
}

/// Represents the state of the Slider widget.
pub struct State {
    drag: Option<Drag>,
    ids: Ids,
}

/// The part of the `RangeSlider` that is in the process of being dragged.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Drag {
    /// One of the edges is being dragged.
    Edge(Edge),
    /// The whole range is being dragged.
    Handle,
}

/// Either the `Start` or `End` `Edge` of the `RangeSlider`'s bar.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Edge {
    /// The start edge of the scrollbar handle.
    Start,
    /// The end edge of the scrollbar handle.
    End,
}

/// The `Event` type produced by the `RangeSlider`.
///
/// This can be used as an `Iterator` that only yields changed `Edge`s.
#[derive(Clone)]
pub struct Event<T> {
    start: Option<T>,
    end: Option<T>,
}


impl<T> Iterator for Event<T> {
    type Item = (Edge, T);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(new_start) = self.start.take() {
            return Some((Edge::Start, new_start));
        }
        if let Some(new_end) = self.end.take() {
            return Some((Edge::End, new_end));
        }
        None
    }
}


impl<'a, T> RangeSlider<'a, T> {
    /// Construct a new RangeSlider widget.
    pub fn new(start: T, end: T, min: T, max: T) -> Self {
        RangeSlider {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            start: start,
            end: end,
            min: min,
            max: max,
            maybe_label: None,
            skew: 1.0,
        }
    }

    /// Specify the font used for displaying the label.
    pub fn label_font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.label_font_id = Some(Some(font_id));
        self
    }

    /// The amount in which the slider's display should be skewed.
    ///
    /// Higher skew amounts (above 1.0) will weight lower values.
    ///
    /// Lower skew amounts (below 1.0) will weight heigher values.
    ///
    /// All skew amounts should be greater than 0.0.
    ///
    /// By default, this value is `1.0` (no skew).
    pub fn skew(mut self, skew: f32) -> Self {
        self.skew = skew;
        self
    }
}

impl<'a, T> Widget for RangeSlider<'a, T>
where
    T: Float,
{
    type State = State;
    type Style = Style;
    type Event = Event<T>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            drag: None,
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

    /// Update the state of the range slider.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, rect, style, ui, .. } = args;
        let RangeSlider { start, end, min, max, maybe_label, skew, .. } = self;

        let border = style.border(ui.theme());
        let inner_rect = rect.pad(border);

        // Functions for converting between ranges.
        let normalise_value = |v: T| utils::map_range(v, min, max, 0.0, 1.0);
        let normalise_and_skew_value = |v: T| {
            let f = utils::clamp(normalise_value(v), 0.0, 1.0);
            f.powf(skew as f64)
        };
        let unskew_and_unnormalise_value = |f: f64| {
            let f = utils::clamp(f, 0.0, 1.0);
            utils::map_range(f.powf(1.0 / skew as f64), 0.0, 1.0, min, max)
        };
        let value_to_x = |v: T| {
            let f = normalise_and_skew_value(v);
            utils::map_range(f, 0.0, 1.0, inner_rect.left(), inner_rect.right())
        };
        let x_to_value = |x: Scalar| {
            let f = utils::map_range(x, inner_rect.left(), inner_rect.right(), 0.0, 1.0);
            let unskewed_and_unnormalised = unskew_and_unnormalise_value(f);
            unskewed_and_unnormalised
        };

        let mut maybe_drag = state.drag;
        let mut new_start = utils::clamp(start, min, max);
        let mut new_end = utils::clamp(end, start, max);
        for widget_event in ui.widget_input(id).events() {
            use event;
            use input;

            match widget_event {

                // - If over the range and within the range_end_grab_threshold, snap the end to
                // the cursor.
                // - Else if over the range, begin dragging the range.
                // - Else if not over the range, snap the end closest to the mouse to the mouse.
                event::Widget::Press(press) => {
                    let press_xy = match press.button {
                        event::Button::Mouse(input::MouseButton::Left, press_xy) => press_xy,
                        _ => continue,
                    };
                    let abs_press_xy = utils::vec2_add(inner_rect.xy(), press_xy);
                    if inner_rect.is_over(abs_press_xy) {
                        let start_x = value_to_x(new_start);
                        let end_x = value_to_x(new_end);
                        let handle_rect = Rect { x: Range::new(start_x, end_x), y: inner_rect.y };
                        let length_x = end_x - start_x;
                        let grab_edge_threshold = length_x / 10.0;
                        if handle_rect.is_over(abs_press_xy) {
                            let distance_from_start = (abs_press_xy[0] - start_x).abs();
                            if distance_from_start <= grab_edge_threshold {
                                maybe_drag = Some(Drag::Edge(Edge::Start));
                                new_start = x_to_value(abs_press_xy[0]);
                                continue;
                            }
                            let distance_from_end = (end_x - abs_press_xy[0]).abs();
                            if distance_from_end <= grab_edge_threshold {
                                maybe_drag = Some(Drag::Edge(Edge::End));
                                new_end = x_to_value(abs_press_xy[0]);
                                continue;
                            }
                            maybe_drag = Some(Drag::Handle);
                        } else {
                            // If the mouse is not over the handle, grab the closest edge.
                            let distance_from_start = start_x - abs_press_xy[0];
                            let distance_from_end = end_x - abs_press_xy[0];
                            if distance_from_start == distance_from_end {
                                if distance_from_start > 0.0 {
                                    maybe_drag = Some(Drag::Edge(Edge::Start));
                                    new_start = x_to_value(abs_press_xy[0]);
                                } else {
                                    maybe_drag = Some(Drag::Edge(Edge::End));
                                    new_end = x_to_value(abs_press_xy[0]);
                                }
                            } else if distance_from_start.abs() < distance_from_end.abs() {
                                maybe_drag = Some(Drag::Edge(Edge::Start));
                                new_start = x_to_value(abs_press_xy[0]);
                            } else {
                                maybe_drag = Some(Drag::Edge(Edge::End));
                                new_end = x_to_value(abs_press_xy[0]);
                            }
                        }
                    }
                },

                // Drags either the Start, End or the whole handle depending on where it was pressed.
                event::Widget::Drag(drag_event) if drag_event.button == input::MouseButton::Left => {
                    match maybe_drag {
                        Some(Drag::Edge(Edge::Start)) => {
                            let abs_drag_to = inner_rect.x() + drag_event.to[0];
                            let v = x_to_value(abs_drag_to);
                            new_start = utils::clamp(v, min, new_end);
                        },
                        Some(Drag::Edge(Edge::End)) => {
                            let abs_drag_to = inner_rect.x() + drag_event.to[0];
                            let v = x_to_value(abs_drag_to);
                            new_end = utils::clamp(v, new_start, max);
                        },
                        Some(Drag::Handle) => {
                            let drag_amt = drag_event.delta_xy[0];
                            let end_x = value_to_x(new_end);
                            let start_x = value_to_x(new_start);
                            if drag_amt.is_sign_positive() {
                                let max_x = inner_rect.right();
                                let dragged_end = utils::clamp(end_x + drag_amt, end_x, max_x);
                                let distance_dragged = dragged_end - end_x;
                                let dragged_start = start_x + distance_dragged;
                                new_start = x_to_value(dragged_start);
                                new_end = x_to_value(dragged_end);
                            } else {
                                let min_x = inner_rect.left();
                                let dragged_start = utils::clamp(start_x + drag_amt, min_x, start_x);
                                let distance_dragged = dragged_start - start_x;
                                let dragged_end = end_x + distance_dragged;
                                new_start = x_to_value(dragged_start);
                                new_end = x_to_value(dragged_end);
                            }
                        },
                        None => (),
                    }
                },

                event::Widget::Release(release) => {
                    if let event::Button::Mouse(input::MouseButton::Left, _) = release.button {
                        maybe_drag = None;
                    }
                },

                _ => (),
            }

        }

        // If the value has just changed, or if the slider has been clicked/released, produce an
        // event.
        let event = Event {
            start: if start != new_start { Some(new_start) } else { None },
            end: if end != new_end { Some(new_end) } else { None },
        };

        if maybe_drag != state.drag {
            state.update(|state| state.drag = maybe_drag);
        }

        // The **Rectangle** for the border.
        let interaction_color = |ui: &::ui::UiCell, color: Color|
            ui.widget_input(id).mouse()
                .map(|mouse| if mouse.buttons.left().is_down() {
                    color.clicked()
                } else {
                    color.highlighted()
                })
                .unwrap_or(color);

        let border_color = interaction_color(&ui, style.border_color(ui.theme()));
        widget::Rectangle::fill(rect.dim())
            .middle_of(id)
            .graphics_for(id)
            .color(border_color)
            .set(state.ids.border, ui);

        // The **Rectangle** for the adjustable slider.
        let mut start_x = value_to_x(new_start);
        let mut end_x = value_to_x(new_end);

        // Always show at least a line for the handle.
        let min_visible_len = 2.0;
        if (start_x - rect.left()) < (rect.right() - end_x) {
            start_x = start_x.min(end_x - min_visible_len);
        } else {
            end_x = end_x.max(start_x + min_visible_len);
        }

        let slider_rect = Rect { x: Range::new(start_x, end_x), y: inner_rect.y };
        let color = interaction_color(&ui, style.color(ui.theme()));
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
                .mid_left_of(id)
                .graphics_for(id)
                .color(label_color)
                .font_size(font_size)
                .set(state.ids.label, ui);
        }

        event
    }

}


impl<'a, T> Colorable for RangeSlider<'a, T> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, T> Borderable for RangeSlider<'a, T> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a, T> Labelable<'a> for RangeSlider<'a, T> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
