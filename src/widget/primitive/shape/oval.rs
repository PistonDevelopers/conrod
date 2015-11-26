use {CharacterCache, LineStyle, Scalar};
use color::{Color, Colorable};
use elmesque::Element;
use position::{Dimensions, Sizeable};
use super::Style as Style;
use widget::{self, Widget};
use widget::primitive::point_path;



/// A simple, non-interactive widget for drawing a single **Oval**.
#[derive(Copy, Clone, Debug)]
pub struct Oval {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// Unique styling.
    pub style: Style,
}

/// Unique state for the **Oval**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State {
    kind: Kind,
}

/// Whether the **Oval** is drawn as an **Outline** or **Fill**ed with a color.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Kind {
    /// Only the **Outline** of the **Oval** is drawn.
    Outline,
    /// The **Oval**'s area is **Fill**ed with some color.
    Fill,
}

/// Unique Kind for the Widget.
pub const KIND: widget::Kind = "Oval";


impl Oval {

    /// Build an **Oval** with the given dimensions and style.
    pub fn styled(dim: Dimensions, style: Style) -> Self {
        Oval {
            common: widget::CommonBuilder::new(),
            style: style,
        }.dim(dim)
    }

    /// Build a new **Fill**ed **Oval**.
    pub fn fill(dim: Dimensions) -> Self {
        Oval::styled(dim, Style::fill())
    }

    /// Build a new **Oval** **Fill**ed with the given color.
    pub fn fill_with(dim: Dimensions, color: Color) -> Self {
        Oval::styled(dim, Style::fill_with(color))
    }

    /// Build a new **Outline**d **Oval** widget.
    pub fn outline(dim: Dimensions) -> Self {
        Oval::styled(dim, Style::outline())
    }

    /// Build a new **Oval** **Outline**d with the given style.
    pub fn outline_styled(dim: Dimensions, line_style: LineStyle) -> Self {
        Oval::styled(dim, Style::outline_styled(line_style))
    }

}


impl Widget for Oval {
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
            kind: Kind::Fill,
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the Oval.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { state, style, .. } = args;

        let kind = match *style {
            Style::Fill(_) => Kind::Fill,
            Style::Outline(_) => Kind::Outline,
        };

        if state.view().kind != kind {
            state.update(|state| state.kind = kind);
        }
    }

    /// Construct an Element for the Oval.
    fn draw<C: CharacterCache>(args: widget::DrawArgs<Self, C>) -> Element {
        use elmesque::form::{collage, polygon};
        use std::f64::consts::PI;

        let widget::DrawArgs { rect, style, theme, .. } = args;
        let (x, y, w, h) = rect.x_y_w_h();
        const CIRCLE_RESOLUTION: usize = 50;
        let t = 2.0 * PI / CIRCLE_RESOLUTION as Scalar;
        let hw = w / 2.0;
        let hh = h / 2.0;
        let f = |i: Scalar| [x + hw * (t*i).cos(), y + hh * (t*i).sin()];
        let points = (0..CIRCLE_RESOLUTION+1).map(|i| f(i as f64));

        match *style {

            // Draw a filled oval.
            Style::Fill(_) => {
                let color = style.get_color(theme);
                // FIXME: This allocation is unnecessary and could be replaced with an iterator.
                let points = points.map(|p| (p[0], p[1])).collect();
                let form = polygon(points).filled(color);
                collage(w as i32, h as i32, vec![form])
            },

            // Draw only the outline of the oval.
            Style::Outline(line_style) => {
                // FIXME: This allocation is unnecessary and could be replaced with an iterator.
                let points: Vec<_> = points.collect();
                point_path::draw_lines(points.iter().cloned(), rect, line_style, theme)
            },
        }
    }

}


impl Colorable for Oval {
    fn color(mut self, color: Color) -> Self {
        self.style.set_color(color);
        self
    }
}

