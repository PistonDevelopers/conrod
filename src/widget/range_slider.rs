use {
    Color,
    Colorable,
    FontSize,
    Frameable,
    Labelable,
    IndexSlot,
    KidArea,
    Padding,
    Positionable,
    Range,
    Rect,
    Rectangle,
    Scalar,
    Text,
    Widget,
};
use num::{Float, NumCast, ToPrimitive};
use utils;
use widget;

/// Linear range selection.
pub struct RangeSlider<'a, T, F> {
    common: widget::CommonBuilder,
    start: T,
    end: T,
    min: T,
    max: T,
    /// Set the reaction for the Slider.
    ///
    /// It will be triggered if the value is updated or if the mouse button is released while the
    /// cursor is above the rectangle.
    pub maybe_react: Option<F>,
    maybe_label: Option<&'a str>,
    style: Style,
}

/// Unique kind for the widget type.
pub const KIND: widget::Kind = "RangeSlider";

widget_style!{
    KIND;
    /// Graphical styling unique to the RangeSlider widget.
    style Style {
        /// The color of the slidable rectangle.
        - color: Color { theme.shape_color }
        /// The length of the frame around the edges of the slidable rectangle.
        - frame: Scalar { theme.frame_width }
        /// The color of the Slider's frame.
        - frame_color: Color { theme.frame_color }
        /// The color of the Slider's label.
        - label_color: Color { theme.label_color }
        /// The font-size for the Slider's label.
        - label_font_size: FontSize { theme.font_size_medium }
    }
}

/// Represents the state of the Slider widget.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    drag: Option<Drag>,
    frame_idx: IndexSlot,
    slider_idx: IndexSlot,
    label_idx: IndexSlot,
}

/// The part of the `RangeSlider` that is in the process of being dragged.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Drag {
    Edge(Edge),
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

impl<'a, T, F> RangeSlider<'a, T, F> {

    /// Construct a new RangeSlider widget.
    pub fn new(start: T, end: T, min: T, max: T) -> Self {
        RangeSlider {
            common: widget::CommonBuilder::new(),
            start: start,
            end: end,
            min: min,
            max: max,
            maybe_react: None,
            maybe_label: None,
            style: Style::new(),
        }
    }

    builder_methods!{
        pub react { maybe_react = Some(F) }
    }

}

impl<'a, T, F> Widget for RangeSlider<'a, T, F>
    where F: FnMut(Edge, T),
          T: Float + NumCast + ToPrimitive,
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

    fn init_state(&self) -> Self::State {
        State {
            drag: None,
            frame_idx: IndexSlot::new(),
            slider_idx: IndexSlot::new(),
            label_idx: IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn kid_area(&self, args: widget::KidAreaArgs<Self>) -> KidArea {
        const LABEL_PADDING: Scalar = 10.0;
        KidArea {
            rect: args.rect,
            pad: Padding {
                x: Range::new(LABEL_PADDING, LABEL_PADDING),
                y: Range::new(LABEL_PADDING, LABEL_PADDING),
            },
        }
    }

    /// Update the state of the Slider.
    fn update(self, args: widget::UpdateArgs<Self>) {
        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let RangeSlider { start, end, min, max, maybe_label, maybe_react, .. } = self;

        let frame = style.frame(ui.theme());
        let inner_rect = rect.pad(frame);

        let value_to_x = |v| utils::map_range(v, min, max, inner_rect.left(), inner_rect.right());
        let x_to_value = |x| utils::map_range(x, inner_rect.left(), inner_rect.right(), min, max);

        let mut maybe_drag = state.drag;
        let mut new_start = start;
        let mut new_end = utils::clamp(end, start, max);
        for widget_event in ui.widget_input(idx).events() {
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

        // If the value has just changed, or if the slider has been clicked/released, call the
        // reaction function.
        if let Some(mut react) = maybe_react {
            if start != new_start {
                react(Edge::Start, new_start);
            }
            if end != new_end {
                react(Edge::End, new_end);
            }
        }

        if maybe_drag != state.drag {
            state.update(|state| state.drag = maybe_drag);
        }

        // The **Rectangle** for the frame.
        let frame_idx = state.frame_idx.get(&mut ui);

        let interaction_color = |ui: &::ui::UiCell, color: Color|
            ui.widget_input(idx).mouse()
                .map(|mouse| if mouse.buttons.left().is_down() {
                    color.clicked()
                } else {
                    color.highlighted()
                })
                .unwrap_or(color);

        let frame_color = interaction_color(&ui, style.frame_color(ui.theme()));
        Rectangle::fill(rect.dim())
            .middle_of(idx)
            .graphics_for(idx)
            .color(frame_color)
            .set(frame_idx, &mut ui);

        // The **Rectangle** for the adjustable slider.
        let start_x = value_to_x(new_start);
        let end_x = value_to_x(new_end);
        let slider_rect = Rect { x: Range::new(start_x, end_x), y: inner_rect.y };
        let color = interaction_color(&ui, style.color(ui.theme()));
        let slider_idx = state.slider_idx.get(&mut ui);
        let slider_xy_offset = [slider_rect.x() - rect.x(), slider_rect.y() - rect.y()];
        Rectangle::fill(slider_rect.dim())
            .xy_relative_to(idx, slider_xy_offset)
            .graphics_for(idx)
            .parent(idx)
            .color(color)
            .set(slider_idx, &mut ui);

        // The **Text** for the slider's label (if it has one).
        if let Some(label) = maybe_label {
            let label_color = style.label_color(ui.theme());
            let font_size = style.label_font_size(ui.theme());
            //const TEXT_PADDING: f64 = 10.0;
            let label_idx = state.label_idx.get(&mut ui);
            Text::new(label)
                .mid_left_of(idx)
                .graphics_for(idx)
                .color(label_color)
                .font_size(font_size)
                .set(label_idx, &mut ui);
        }
    }

}


impl<'a, T, F> Colorable for RangeSlider<'a, T, F> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, T, F> Frameable for RangeSlider<'a, T, F> {
    builder_methods!{
        frame { style.frame = Some(Scalar) }
        frame_color { style.frame_color = Some(Color) }
    }
}

impl<'a, T, F> Labelable<'a> for RangeSlider<'a, T, F> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
