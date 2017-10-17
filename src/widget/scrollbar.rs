//! A widget that allows for manually scrolling via dragging the mouse.

use {Color, Colorable, Positionable, Ui};
use graph;
use position::{Dimension, Range, Rect, Scalar};
use std;
use utils;
use widget::{self, Widget};
use widget::scroll::{self, X, Y};


/// A widget that allows for scrolling via dragging the mouse.
#[derive(WidgetCommon_)]
pub struct Scrollbar<A> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    style: Style,
    widget: widget::Id,
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
    fn default_x_dimension(scrollbar: &Scrollbar<Self>, ui: &Ui) -> Dimension;
    /// Determine a default *y* dimension for the scrollbar in the case that no specific height is
    /// given.
    fn default_y_dimension(scrollbar: &Scrollbar<Self>, ui: &Ui) -> Dimension;
    /// Convert a given `Scalar` along the axis into two dimensions.
    fn to_2d(scalar: Scalar) -> [Scalar; 2];
}

/// Styling for the DropDownList, necessary for constructing its renderable Element.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// Color of the widget.
    #[conrod(default = "theme.border_color")]
    pub color: Option<Color>,
    /// The "thickness" of the scrollbar's track and handle `Rect`s.
    #[conrod(default = "10.0")]
    pub thickness: Option<Scalar>,
    /// When true, the `Scrollbar` will only be visible when:
    ///
    /// - The target scrollable widget is being scrolled.
    /// - The mouse is over the scrollbar.
    #[conrod(default = "false")]
    pub auto_hide: Option<bool>,
}

widget_ids! {
    struct Ids {
        track,
        handle,
    }
}

/// The state of the `Scrollbar`.
pub struct State {
    ids: Ids,
}

impl<A> Scrollbar<A> {

    /// Begin building a new scrollbar widget.
    fn new(widget: widget::Id) -> Self {
        Scrollbar {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            widget: widget,
            axis: std::marker::PhantomData,
        }
    }

    /// By default, this is set to `false`.
    ///
    /// When false, the `Scrollbar` will always be visible.
    ///
    /// When true, the `Scrollbar` will only be visible when:
    ///
    /// - The target scrollable widget is actually scrollable and:
    /// - The target scrollable widget is being scrolled.
    /// - The scrollbar is capturing the mouse.
    pub fn auto_hide(mut self, auto_hide: bool) -> Self {
        self.style.auto_hide = Some(auto_hide);
        self
    }

    /// Build the widget with the given `thickness`.
    ///
    /// This value sets the width of vertical scrollbars, or the height of horizontal scrollbars.
    ///
    /// By default, this is `10.0`.
    pub fn thickness(mut self, thickness: Scalar) -> Self {
        self.style.thickness = Some(thickness);
        self
    }

}

impl Scrollbar<X> {

    /// Begin building a new scrollbar widget that scrolls along the *X* axis along the range of
    /// the scrollable widget at the given Id.
    pub fn x_axis(widget: widget::Id) -> Self {
        Scrollbar::new(widget)
            .align_middle_x_of(widget)
            .align_bottom_of(widget)
    }

}

impl Scrollbar<Y> {

    /// Begin building a new scrollbar widget that scrolls along the *Y* axis along the range of
    /// the scrollable widget at the given Id.
    pub fn y_axis(widget: widget::Id) -> Self {
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
    type Event = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn default_x_dimension(&self, ui: &Ui) -> Dimension {
        A::default_x_dimension(self, ui)
    }

    fn default_y_dimension(&self, ui: &Ui) -> Dimension {
        A::default_y_dimension(self, ui)
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, rect, style, ui, .. } = args;
        let Scrollbar { widget, .. } = self;

        // Only continue if the widget that we want to scroll has some scroll state.
        let (offset_bounds, offset, scrollable_range_len, is_scrolling) =
            match ui.widget_graph().widget(widget) {
                Some(widget) => match A::scroll_state(widget) {
                    Some(scroll) =>
                        (scroll.offset_bounds,
                         scroll.offset,
                         scroll.scrollable_range_len,
                         scroll.is_scrolling),
                    None => return,
                },
                None => return,
            };

        // Calculates the `Rect` for a scroll "handle" sitting on the given `track` with an offset
        // and length that represents the given `Axis`' `state`.
        let handle_rect = {
            let perpendicular_track_range = A::perpendicular_range(rect);
            let track_range = A::parallel_range(rect);
            let track_len = track_range.len();
            let len = if scrollable_range_len == 0.0 {
                track_len
            } else {
                utils::clamp(track_len * (track_len / scrollable_range_len), 0.0, track_len)
            };
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
        for widget_event in ui.widget_input(id).events() {
            use event;
            use input;

            match widget_event {

                // If the track is pressed, snap the handle to that part of the track and scroll
                // accordingly with the handle's Range clamped to the track's Range.
                event::Widget::Press(press) => {
                    if let event::Button::Mouse(input::MouseButton::Left, xy) = press.button {
                        let abs_xy = utils::vec2_add(xy, rect.xy());
                        if rect.is_over(abs_xy) && !handle_rect.is_over(abs_xy) {
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
        if additional_offset != 0.0 {
            ui.scroll_widget(widget, A::to_2d(additional_offset));
        }

        // Don't draw the scrollbar if auto_hide is on and there is no interaction.
        let auto_hide = style.auto_hide(ui.theme());
        let not_scrollable = offset_bounds.magnitude().is_sign_positive();
        if auto_hide {
            let no_offset = additional_offset == 0.0;
            let no_mouse_interaction = ui.widget_input(id).mouse().is_none();
            if not_scrollable || (!is_scrolling && no_offset && no_mouse_interaction) {
                return;
            }
        }

        let color = style.color(ui.theme());
        let color = if not_scrollable {
            color
        } else {
            ui.widget_input(id)
                .mouse()
                .map(|m| if m.buttons.left().is_down() {
                    color.clicked()
                } else {
                    color.highlighted()
                })
                .unwrap_or_else(|| color)
        };

        // The `Track` widget along which the handle will slide.
        let track_color = color.alpha(0.25);
        widget::Rectangle::fill(rect.dim())
            .xy(rect.xy())
            .color(track_color)
            .graphics_for(id)
            .parent(id)
            .set(state.ids.track, ui);

        // The `Handle` widget used as a graphical representation of the part of the scrollbar that
        // can be dragged over the track.
        widget::Rectangle::fill(handle_rect.dim())
            .xy(handle_rect.xy())
            .color(color)
            .graphics_for(id)
            .parent(id)
            .set(state.ids.handle, ui);
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

    fn default_x_dimension(scrollbar: &Scrollbar<Self>, _ui: &Ui) -> Dimension {
        Dimension::Of(scrollbar.widget, None)
    }

    fn default_y_dimension(scrollbar: &Scrollbar<Self>, ui: &Ui) -> Dimension {
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

    fn default_x_dimension(scrollbar: &Scrollbar<Self>, ui: &Ui) -> Dimension {
        Dimension::Absolute(scrollbar.style.thickness(&ui.theme))
    }

    fn default_y_dimension(scrollbar: &Scrollbar<Self>, _ui: &Ui) -> Dimension {
        Dimension::Of(scrollbar.widget, None)
    }

    fn to_2d(scalar: Scalar) -> [Scalar; 2] {
        [0.0, scalar]
    }

}

impl<A> Colorable for Scrollbar<A> {
    builder_method!(color { style.color = Some(Color) });
}
