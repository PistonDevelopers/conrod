use {
    Color,
    Colorable,
    Dimensions,
    Frameable,
    IndexSlot,
    Rectangle,
    Positionable,
    Scalar,
    Sizeable,
    Widget,
};
use widget;


/// A filled rectangle widget that may or may not have some frame.
///
/// NOTE: FramedRectangle is currently implemented as two filled rectangles:
///
/// 1. A `Rectangle` for the frame.
/// 2. A `Rectangle` for the non-frame area.
///
/// This is flawed in that, if a user specifies an alpha lower than 1.0, the front `Rectangle` will
/// blend with the frame `Rectangle`, which is likely unexpected behaviour. This should be changed
/// so that the frame is drawn using a outlined `Rectangle`.
#[derive(Copy, Clone, Debug)]
pub struct FramedRectangle {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// Unique styling for the **FramedRectangle**.
    pub style: Style,
}

/// Unique kind for the Widget.
pub const KIND: widget::Kind = "FramedRectangle";

/// Unique state for the `FramedRectangle`.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    frame_idx: IndexSlot,
    rectangle_idx: IndexSlot,
}

widget_style!{
    KIND;
    /// Unique styling for the **FramedRectangle** widget.
    style Style {
        /// Shape styling for the inner rectangle.
        - color: Color { theme.shape_color }
        /// The thickness of the frame.
        - frame: Scalar { theme.frame_width }
        /// The color of the frame.
        - frame_color: Color { theme.frame_color }
    }
}

impl FramedRectangle {

    /// Build a new **FramedRectangle**.
    pub fn new(dim: Dimensions) -> Self {
        FramedRectangle {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
        }.wh(dim)
    }

    builder_method!(pub with_style { style = Style });

}


impl Widget for FramedRectangle {
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
            frame_idx: IndexSlot::new(),
            rectangle_idx: IndexSlot::new(),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the Rectangle.
    fn update(self, args: widget::UpdateArgs<Self>) {
        let widget::UpdateArgs { idx, state, style, rect, mut ui, .. } = args;

        let frame = style.frame(&ui.theme);
        if frame > 0.0 {
            let frame_color = style.frame_color(&ui.theme);
            let frame_idx = state.frame_idx.get(&mut ui);
            Rectangle::fill(rect.dim())
                .xy(rect.xy())
                .color(frame_color)
                .parent(idx)
                .graphics_for(idx)
                .set(frame_idx, &mut ui);
        }

        let color = style.color(&ui.theme);
        let rectangle_idx = state.rectangle_idx.get(&mut ui);
        Rectangle::fill(rect.pad(frame).dim())
            .xy(rect.xy())
            .color(color)
            .parent(idx)
            .graphics_for(idx)
            .set(rectangle_idx, &mut ui);
    }

}


impl Colorable for FramedRectangle {
    builder_method!(color { style.color = Some(Color) });
}


impl Frameable for FramedRectangle {
    builder_methods!{
        frame { style.frame = Some(Scalar) }
        frame_color { style.frame_color = Some(Color) }
    }
}
