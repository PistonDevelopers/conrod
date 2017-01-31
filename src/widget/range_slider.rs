//! A widget for specifying start and end values for some linear range.

use {Color, Colorable, FontSize, Borderable, Labelable, Positionable, Widget};
use num::{Float, NumCast, ToPrimitive};
use position::{Padding, Range, Rect, Scalar};
use text;
use utils;
use widget;

/// Linear range selection.
pub struct RangeSlider<'a, T> {
    common: widget::CommonBuilder,
    start: T,
    end: T,
    min: T,
    max: T,
    maybe_label: Option<&'a str>,
    style: Style,
}

widget_style!{
    /// Graphical styling unique to the RangeSlider widget.
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
            common: widget::CommonBuilder::new(),
            start: start,
            end: end,
            min: min,
            max: max,
            maybe_label: None,
            style: Style::new(),
        }
    }

    /// Specify the font used for displaying the label.
    pub fn label_font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.label_font_id = Some(Some(font_id));
        self
    }

}

impl<'a, T> Widget for RangeSlider<'a, T>
    where T: Float + NumCast + ToPrimitive,
{
    type State = State;
    type Style = Style;
    type Event = Event<T>;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

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

    /// Update the state of the Slider.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, rect, style, mut ui, .. } = args;
        let RangeSlider { start, end, min, max, maybe_label, .. } = self;

        let border = style.border(ui.theme());
        let inner_rect = rect.pad(border);

        let value_to_x = |v| utils::map_range(v, min, max, inner_rect.left(), inner_rect.right());
        let x_to_value = |x| utils::map_range(x, inner_rect.left(), inner_rect.right(), min, max);

        let mut maybe_drag = state.drag;
        let mut new_start = start;
        let mut new_end = utils::clamp(end, start, max);
        for widget_event in ui.widget_input(id).events() {
            use event;
            use input;

            match widget_event {

                /// - If over the range and within the range_end_grab_threshold, snap the end to
                /// the cursor.
                /// - Else if over the range, begin dragging the range.
                /// - Else if not over the range, snap the end closest to the mouse to the mouse.
                event::Widget::Press(press) => {
                    let press_xy = match press.button {
                        event::Button::Mouse(input::MouseButton::Left, press_xy) => press_xy,
                        _ => continue,
                    };
                    let abs_press_xy = utils::vec2_add(inner_rect.xy(), press_xy);
                    if inner_rect.is_over(abs_press_xy) {
                        let start_x = value_to_x(new_start);
                        let end_x = value_to_x(new_end);
                        let length_x = end_x - start_x;
                        let grab_edge_threshold = length_x / 10.0;
                        let handle_rect = Rect { x: Range::new(start_x, end_x), y: inner_rect.y };
                        if handle_rect.is_over(abs_press_xy) {
                            let distance_from_start = (abs_press_xy[0] - start_x).abs();
                            if distance_from_start < grab_edge_threshold {
                                maybe_drag = Some(Drag::Edge(Edge::Start));
                                new_start = x_to_value(abs_press_xy[0]);
                                continue;
                            }
                            let distance_from_end = (end_x - abs_press_xy[0]).abs();
                            if distance_from_end < grab_edge_threshold {
                                maybe_drag = Some(Drag::Edge(Edge::End));
                                new_end = x_to_value(abs_press_xy[0]);
                                continue;
                            }
                            maybe_drag = Some(Drag::Handle);
                        } else {
                            let distance_from_start = (abs_press_xy[0] - start_x).abs();
                            let distance_from_end = (end_x - abs_press_xy[0]).abs();
                            if distance_from_start < distance_from_end {
                                maybe_drag = Some(Drag::Edge(Edge::Start));
                                new_start = x_to_value(abs_press_xy[0]);
                            } else {
                                maybe_drag = Some(Drag::Edge(Edge::End));
                                new_end = x_to_value(abs_press_xy[0]);
                            }
                        }
                    }
                },

                /// Drags either the Start, End or the whole Bar depending on where it was pressed.
                event::Widget::Drag(drag_event) if drag_event.button == input::MouseButton::Left => {
                    match maybe_drag {
                        Some(Drag::Edge(Edge::Start)) => {
                            let abs_drag_to = inner_rect.x() + drag_event.to[0];
                            new_start = utils::clamp(x_to_value(abs_drag_to), min, new_end);
                        },
                        Some(Drag::Edge(Edge::End)) => {
                            let abs_drag_to = inner_rect.x() + drag_event.to[0];
                            new_end = utils::clamp(x_to_value(abs_drag_to), new_start, max);
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
        let start_x = value_to_x(new_start);
        let end_x = value_to_x(new_end);
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
