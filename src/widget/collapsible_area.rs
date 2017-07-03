//! The `CollapsibleArea` widget and related items.

use {Borderable, Colorable, Labelable, Positionable, Sizeable, Widget};
use {Color, FontSize, Scalar, UiCell};
use position;
use std;
use text;
use widget;

/// A vertically collapsible area.
///
/// When "open" this widget returns a canvas upon which other widgets can be placed.
#[derive(Copy, Clone, Debug, WidgetCommon_)]
pub struct CollapsibleArea<'a> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    style: Style,
    is_open: bool,
    text: &'a str,
}

widget_ids! {
    /// The unique identifiers for the `CollapsibleArea`'s child widgets.
    #[allow(missing_docs, missing_copy_implementations)]
    pub struct Ids {
        button,
        triangle,
        area,
    }
}

/// The unique state cached within the widget graph for the `CollapsibleArea`.
pub struct State {
    ids: Ids,
}

/// Unique styling for the CollapsibleArea.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// Color of the Button's pressable area.
    #[conrod(default = "theme.shape_color")]
    pub color: Option<Color>,
    /// Width of the border surrounding the button
    #[conrod(default = "theme.border_width")]
    pub border: Option<Scalar>,
    /// The color of the border.
    #[conrod(default = "theme.border_color")]
    pub border_color: Option<Color>,
    /// The color of the Button's label.
    #[conrod(default = "theme.label_color")]
    pub label_color: Option<Color>,
    /// The font size of the Button's label.
    #[conrod(default = "None")]
    pub label_font_size: Option<Option<FontSize>>,
    /// The ID of the font used to display the label.
    #[conrod(default = "theme.font_id")]
    pub label_font_id: Option<Option<text::font::Id>>,
}

/// The event returned when the text bar or triangle is pressed.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Event {
    /// The collapsible area was opened.
    Open,
    /// The collapsible area was closed.
    Close,
}

/// The area returned by the widget when the `CollapsibleArea` is open.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Area {
    /// A unique identifier for the user's widget.
    pub id: widget::Id,
    /// The widget::Id for the collapsible area that produced this `Area`.
    pub collapsible_area_id: widget::Id,
    /// The width of the `CollapsibleArea` that produced this `Area`.
    pub width: Scalar
}


impl<'a> CollapsibleArea<'a> {

    /// Begin building the `CollapsibleArea` widget.
    pub fn new(is_open: bool, text: &'a str) -> Self {
        CollapsibleArea {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            is_open: is_open,
            text: text,
        }
    }

    /// Specify the color of the `CollapsibleArea`'s label.
    pub fn label_color(mut self, color: Color) -> Self {
        self.style.label_color = Some(color);
        self
    }

    /// Specify the font size for the `CollapsibleArea`'s label.
    pub fn label_font_size(mut self, font_size: FontSize) -> Self {
        self.style.label_font_size = Some(Some(font_size));
        self
    }

    /// Specify the font for the `CollapsibleArea`'s label.
    pub fn label_font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.label_font_id = Some(Some(font_id));
        self
    }

}

impl<'a> Widget for CollapsibleArea<'a> {
    type State = State;
    type Style = Style;
    type Event = (Option<Area>, Option<Event>);

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, style, rect, ui, .. } = args;
        let CollapsibleArea { text, mut is_open, .. } = self;

        let (_, _, w, h) = rect.x_y_w_h();
        let color = style.color(&ui.theme);
        let border = style.border(&ui.theme);
        let border_color = style.border_color(&ui.theme);
        let label_color = style.label_color(&ui.theme);
        let label_font_id = style.label_font_id(&ui.theme).or(ui.fonts.ids().next());
        let label_font_size = match style.label_font_size(&ui.theme) {
            Some(font_size) => font_size,
            None => std::cmp::max((h / 2.5) as FontSize, 10),
        };

        // The rectangle in which the triangle is set.
        let triangle_rect = position::Rect {
            x: position::Range {
                start: rect.x.start,
                end: rect.x.start + rect.y.len(),
            },
            y: rect.y,
        };

        // When the button is pressed, toggle whether the area is open or closed.
        let event = widget::Button::new()
            .w_h(w, h)
            .middle_of(id)
            .color(color)
            .border(border)
            .border_color(border_color)
            .label(text)
            .label_color(label_color)
            .label_font_size(label_font_size)
            .label_x(position::Relative::Place(position::Place::Start(Some(triangle_rect.w()))))
            .and_then(label_font_id, |b, font_id| b.label_font_id(font_id))
            .set(state.ids.button, ui)
            .next()
            .map(|_| {
                is_open = !is_open;
                if is_open { Event::Open } else { Event::Close }
            });

        // The points for the triangle.
        let side_offset = triangle_rect.w() / 10.0;
        let point_offset = triangle_rect.h() / 6.0;
        let triangle_x = triangle_rect.x();
        let triangle_y = triangle_rect.y();
        let points = if is_open {
            let a = [triangle_x, triangle_y - point_offset];
            let b = [triangle_x + side_offset, triangle_y + point_offset];
            let c = [triangle_x - side_offset, triangle_y + point_offset];
            [a, b, c]
        } else {
            let a = [triangle_x + point_offset, triangle_y];
            let b = [triangle_x - point_offset, triangle_y + side_offset];
            let c = [triangle_x - point_offset, triangle_y - side_offset];
            [a, b, c]
        };

        // The triangle widget.
        widget::Polygon::fill(points.iter().cloned())
            .align_middle_y_of(state.ids.button)
            .align_left_of(state.ids.button)
            .wh(triangle_rect.dim())
            .parent(state.ids.button)
            .graphics_for(state.ids.button)
            .color(label_color)
            .set(state.ids.triangle, ui);

        // The area on which the user can place their widgets if it is open.
        let area = if is_open {
            Some(Area {
                id: state.ids.area,
                collapsible_area_id: id,
                width: w,
            })
        } else {
            None
        };

        (area, event)
    }
}

impl<'a> Colorable for CollapsibleArea<'a> {
    fn color(mut self, color: Color) -> Self {
        self.style.color = Some(color);
        self
    }
}

impl<'a> Borderable for CollapsibleArea<'a> {
    fn border(mut self, border: Scalar) -> Self {
        self.style.border = Some(border);
        self
    }
    fn border_color(mut self, color: Color) -> Self {
        self.style.border_color = Some(color);
        self
    }
}

impl Event {
    /// Returns whether or not the `Event` results in an open collapsible area.
    pub fn is_open(&self) -> bool {
        match *self {
            Event::Open => true,
            Event::Close => false,
        }
    }
}

impl Area {
    /// Set the user's given widget directly under the `CollapsibleArea`.
    ///
    /// Returns any events produced by the given widget.
    pub fn set<W>(self, widget: W, ui: &mut UiCell) -> W::Event
        where W: Widget,
    {
        let Area { id, collapsible_area_id, width } = self;
        widget
            .w(width)
            .parent(collapsible_area_id)
            .align_middle_x_of(collapsible_area_id)
            .down_from(collapsible_area_id, 0.0)
            .set(id, ui)
    }
}
