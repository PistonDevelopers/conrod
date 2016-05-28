use {
    Backend,
    Color,
    Colorable,
    Dimension,
    IndexSlot,
    Positionable,
    Range,
    Rect,
    Rectangle,
    Scalar,
    Sizeable,
    Ui,
};
use graph;
use std;
use utils;
use widget::{self, Widget};
use widget::scroll::{self, X, Y};


/// A widget that allows for scrolling via dragging the mouse.
pub struct Scrollbar<A> {
    common: widget::CommonBuilder,
    style: Style,
    widget: widget::Index,
    axis: std::marker::PhantomData<A>,
}

/// The axis that is scrolled by the `Scrollbar`.
pub trait Axis: scroll::Axis + Sized {
    /// The `Rect` for a scroll "track" with the given `thickness` for a container with the given
    /// `Rect`.
    fn track_rect(container: Rect, thickness: Scalar) -> Rect;
    /// The `Rect` for a scroll handle given both `Range`s.
    fn handle_rect(perpendicular_track_range: Range, handle_range: Range) -> Rect;
    /// Retrieve the related `scroll::State` for the axis from a given widget container.
    fn scroll_state(widget: &graph::Container) -> Option<&scroll::State<Self>>;
    /// Determine a default *x* dimension for the scrollbar in the case that no specific width is
    /// given.
    fn default_x_dimension<B: Backend>(scrollbar: &Scrollbar<Self>, ui: &Ui<B>) -> Dimension;
    /// Determine a default *y* dimension for the scrollbar in the case that no specific height is
    /// given.
    fn default_y_dimension<B: Backend>(scrollbar: &Scrollbar<Self>, ui: &Ui<B>) -> Dimension;
    /// Convert a given `Scalar` along the axis into two dimensions.
    fn to_2d(scalar: Scalar) -> [Scalar; 2];
}

/// Unique kind for the widget.
pub const KIND: widget::Kind = "Scrollbar";


widget_style!{
    KIND;
    /// Styling for the DropDownList, necessary for constructing its renderable Element.
    style Style {
        /// Color of the widget.
        - color: Color { theme.shape_color }
        /// The "thickness" of the scrollbar's track and handle `Rect`s.
        - thickness: Scalar { 10.0 }
    }
}

/// The state of the `Scrollbar`.
#[derive(PartialEq, Clone, Debug)]
pub struct State {
    track_idx: IndexSlot,
    handle_idx: IndexSlot,
}

impl<A> Scrollbar<A> {
    /// Begin building a new scrollbar widget.
    fn new(widget: widget::Index) -> Self {
        Scrollbar {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
            widget: widget,
            axis: std::marker::PhantomData,
        }
    }
}

impl Scrollbar<X> {

    /// Begin building a new scrollbar widget that scrolls along the *X* axis along the range of
    /// the scrollable widget at the given index.
    pub fn x_axis<I: Into<widget::Index> + Copy>(widget: I) -> Self {
        let widget = widget.into();
        Scrollbar::new(widget)
            .align_middle_x_of(widget)
            .align_bottom_of(widget)
    }

}

impl Scrollbar<Y> {

    /// Begin building a new scrollbar widget that scrolls along the *Y* axis along the range of
    /// the scrollable widget at the given index.
    pub fn y_axis<I: Into<widget::Index> + Copy>(widget: I) -> Self {
        let widget = widget.into();
        Scrollbar::new(widget)
            .align_middle_y_of(widget)
            .align_right_of(widget)
    }

}

impl<A> Widget for Scrollbar<A>
    where A: Axis,
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

    fn init_state(&self) -> State {
        State {
            track_idx: IndexSlot::new(),
            handle_idx: IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn default_x_dimension<B: Backend>(&self, ui: &Ui<B>) -> Dimension {
        A::default_x_dimension(self, ui)
    }

    fn default_y_dimension<B: Backend>(&self, ui: &Ui<B>) -> Dimension {
        A::default_y_dimension(self, ui)
    }

    fn update<B: Backend>(self, args: widget::UpdateArgs<Self, B>) {
        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let Scrollbar { widget, .. } = self;

        let color = style.color(ui.theme());

        // Only continue if the widget that we want to scroll has some scroll state.
        let (offset_bounds, offset, scrollable_range_len) = match ui.widget_graph().widget(widget) {
            Some(widget) => match A::scroll_state(widget) {
                Some(scroll) => (scroll.offset_bounds, scroll.offset, scroll.scrollable_range_len),
                None => return,
            },
            None => return,
        };

        // The `Track` widget along which the handle will slide.
        let track_idx = state.view().track_idx.get(&mut ui);
        let track_color = color.alpha(0.25);
        Rectangle::fill(rect.dim())
            .xy(rect.xy())
            .color(track_color)
            .graphics_for(idx)
            .parent(idx)
            .set(track_idx, &mut ui);

        // Calculates the `Rect` for a scroll "handle" sitting on the given `track` with an offset
        // and length that represents the given `Axis`' `state`.
        let handle_rect = {
            let perpendicular_track_range = A::perpendicular_range(rect);
            let track_range = A::parallel_range(rect);
            let track_len = track_range.len();
            let len = track_len * (track_len / scrollable_range_len);
            let handle_range = Range::from_pos_and_len(0.0, len);
            let pos = {
                let pos_min = handle_range.align_start_of(track_range).middle();
                let pos_max = handle_range.align_end_of(track_range).middle();
                let pos_bounds = Range::new(pos_min, pos_max);
                offset_bounds.map_value_to(offset, &pos_bounds)
            };
            let range = Range::from_pos_and_len(pos, len);
            A::handle_rect(perpendicular_track_range, range)
        };

        let handle_range = A::parallel_range(handle_rect);
        let handle_pos_range_len = || {
            let track_range = A::parallel_range(rect);
            let handle_pos_at_start = handle_range.align_start_of(track_range).middle();
            let handle_pos_at_end = handle_range.align_end_of(track_range).middle();
            let handle_pos_range = Range::new(handle_pos_at_start, handle_pos_at_end);
            handle_pos_range.len()
        };

        // Sum all offset yielded by `Press` and `Drag` events.
        let mut additional_offset = 0.0;
        for widget_event in ui.widget_input(idx).events() {
            use event;
            use input;

            match widget_event {

                // If the track is pressed, snap the handle to that part of the track and scroll
                // accordingly with the handle's Range clamped to the track's Range.
                event::Widget::Press(press) => {
                    if let event::Button::Mouse(input::MouseButton::Left, xy) = press.button {
                        let abs_xy = utils::vec2_add(xy, rect.xy());
                        if rect.is_over(abs_xy) && !handle_rect.is_over(abs_xy) {
                            println!("click!");
                            let handle_pos_range_len = handle_pos_range_len();
                            let offset_range_len = offset_bounds.len();
                            let mouse_scalar = A::mouse_scalar(xy);
                            let pos_offset = mouse_scalar - handle_range.middle();
                            let offset = utils::map_range(pos_offset,
                                                          0.0, handle_pos_range_len,
                                                          0.0, offset_range_len);
                            additional_offset += -offset;
                        }
                    }
                },

                // Check for the handle being dragged across the track.
                event::Widget::Drag(drag) if drag.button == input::MouseButton::Left => {
                    let handle_pos_range_len = handle_pos_range_len();
                    let offset_range_len = offset_bounds.len();
                    let from_scalar = A::mouse_scalar(drag.from);
                    let to_scalar = A::mouse_scalar(drag.to);
                    let pos_offset = to_scalar - from_scalar;
                    let offset = utils::map_range(pos_offset,
                                                  0.0, handle_pos_range_len,
                                                  0.0, offset_range_len);
                    additional_offset += -offset;
                },

                _ => (),

            }
        }

        // Scroll the given widget by the accumulated additional offset.
        ui.scroll_widget(widget, A::to_2d(additional_offset));

        // The `Handle` widget used as a graphical representation of the part of the scrollbar that
        // can be dragged over the track.
        let handle_idx = state.view().handle_idx.get(&mut ui);
        Rectangle::fill(handle_rect.dim())
            .xy(handle_rect.xy())
            .color(color)
            .graphics_for(idx)
            .parent(idx)
            .set(handle_idx, &mut ui);

    }
}

impl Axis for X {

    fn track_rect(container: Rect, thickness: Scalar) -> Rect {
        let h = thickness;
        let w = container.w();
        let x = container.x();
        Rect::from_xy_dim([x, 0.0], [w, h]).align_bottom_of(container)
    }

    fn handle_rect(perpendicular_track_range: Range, handle_range: Range) -> Rect {
        Rect {
            x: handle_range,
            y: perpendicular_track_range,
        }
    }

    fn scroll_state(widget: &graph::Container) -> Option<&scroll::State<Self>> {
        widget.maybe_x_scroll_state.as_ref()
    }

    fn default_x_dimension<B: Backend>(scrollbar: &Scrollbar<Self>, _ui: &Ui<B>) -> Dimension {
        Dimension::Of(scrollbar.widget, None)
    }

    fn default_y_dimension<B: Backend>(scrollbar: &Scrollbar<Self>, ui: &Ui<B>) -> Dimension {
        Dimension::Absolute(scrollbar.style.thickness(&ui.theme))
    }

    fn to_2d(scalar: Scalar) -> [Scalar; 2] {
        [scalar, 0.0]
    }

}

impl Axis for Y {

    fn track_rect(container: Rect, thickness: Scalar) -> Rect {
        let w = thickness;
        let h = container.h();
        let y = container.y();
        Rect::from_xy_dim([0.0, y], [w, h]).align_right_of(container)
    }

    fn handle_rect(perpendicular_track_range: Range, handle_range: Range) -> Rect {
        Rect {
            x: perpendicular_track_range,
            y: handle_range,
        }
    }

    fn scroll_state(widget: &graph::Container) -> Option<&scroll::State<Self>> {
        widget.maybe_y_scroll_state.as_ref()
    }

    fn default_x_dimension<B: Backend>(scrollbar: &Scrollbar<Self>, ui: &Ui<B>) -> Dimension {
        Dimension::Absolute(scrollbar.style.thickness(&ui.theme))
    }

    fn default_y_dimension<B: Backend>(scrollbar: &Scrollbar<Self>, _ui: &Ui<B>) -> Dimension {
        Dimension::Of(scrollbar.widget, None)
    }

    fn to_2d(scalar: Scalar) -> [Scalar; 2] {
        [0.0, scalar]
    }

}

impl<A> Colorable for Scrollbar<A> {
    builder_method!(color { style.color = Some(Color) });
}
