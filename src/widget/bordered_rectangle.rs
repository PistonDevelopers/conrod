//! The `BorderedRectangle` widget and related items.

use {
    Color,
    Colorable,
    Dimensions,
    Borderable,
    Positionable,
    Scalar,
    Sizeable,
    Widget,
};
use widget;


/// A filled rectangle widget that may or may not have some border.
///
/// NOTE: BorderedRectangle is currently implemented as two filled rectangles:
///
/// 1. A `Rectangle` for the border.
/// 2. A `Rectangle` for the non-border area.
///
/// This is flawed in that, if a user specifies an alpha lower than 1.0, the front `Rectangle` will
/// blend with the border `Rectangle`, which is likely unexpected behaviour. This should be changed
/// so that the border is drawn using a outlined `Rectangle`.
#[derive(Copy, Clone, Debug)]
pub struct BorderedRectangle {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// Unique styling for the **BorderedRectangle**.
    pub style: Style,
}

widget_ids! {
    struct Ids {
        border,
        rectangle,
    }
}

widget_style!{
    /// Unique styling for the **BorderedRectangle** widget.
    style Style {
        /// Shape styling for the inner rectangle.
        - color: Color { theme.shape_color }
        /// The thickness of the border.
        - border: Scalar { theme.border_width }
        /// The color of the border.
        - border_color: Color { theme.border_color }
    }
}

/// Unique state for the `BorderedRectangle`.
pub struct State {
    ids: Ids,
}

impl BorderedRectangle {

    /// Build a new **BorderedRectangle**.
    pub fn new(dim: Dimensions) -> Self {
        BorderedRectangle {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
        }.wh(dim)
    }

    builder_method!(pub with_style { style = Style });

}


impl Widget for BorderedRectangle {
    type State = State;
    type Style = Style;
    type Event = ();

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

    /// Update the state of the Rectangle.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, style, rect, ui, .. } = args;

        let border = style.border(&ui.theme);
        if border > 0.0 {
            let border_color = style.border_color(&ui.theme);
            widget::Rectangle::fill(rect.dim())
                .xy(rect.xy())
                .color(border_color)
                .parent(id)
                .graphics_for(id)
                .set(state.ids.border, ui);
        }

        let color = style.color(&ui.theme);
        widget::Rectangle::fill(rect.pad(border).dim())
            .xy(rect.xy())
            .color(color)
            .parent(id)
            .graphics_for(id)
            .set(state.ids.rectangle, ui);
    }

}


impl Colorable for BorderedRectangle {
    builder_method!(color { style.color = Some(Color) });
}


impl Borderable for BorderedRectangle {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}
