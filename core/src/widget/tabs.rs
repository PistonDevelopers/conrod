//! A wrapper around a list of `Canvas`ses that displays them as a list of selectable tabs.

use {
    Color,
    Dimensions,
    FontSize,
    Point,
    Rect,
    Scalar,
    Widget,
};
use std;
use text;
use utils;
use widget;


/// A wrapper around a list of `Canvas`ses that displays them as a list of selectable tabs.
#[derive(WidgetCommon_)]
pub struct Tabs<'a> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    tabs: &'a [(widget::Id, &'a str)],
    style: Style,
    maybe_starting_tab_idx: Option<usize>,
}

/// The state to be cached within the Canvas.
pub struct State {
    /// An owned, ordered list of the **Tab**s and their associated indices.
    tabs: Vec<Tab>,
    /// An index into the `tabs` slice that represents the currently selected Canvas.
    maybe_selected_tab_idx: Option<usize>,
}

/// A single **Tab** in the list owned by the **Tabs** **State**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tab {
    /// The public identifier, given by the user.
    id: widget::Id,
    /// The **Tab**'s selectable **Button**.
    button_id: widget::Id,
}

/// The padding between the edge of the title bar and the title bar's label.
const TAB_BAR_LABEL_PADDING: f64 = 4.0;

/// Unique styling for the `Tabs` widget.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// Layout for the tab selection bar.
    #[conrod(default = "Layout::Horizontal")]
    pub layout: Option<Layout>,
    /// The thickness of the tab selection bar (width for vertical, height for horizontal).
    #[conrod(default = "None")]
    pub bar_thickness: Option<Option<Scalar>>,
    /// Color of the number dialer's label.
    #[conrod(default = "theme.label_color")]
    pub label_color: Option<Color>,
    /// Font size of the number dialer's label.
    #[conrod(default = "theme.font_size_medium")]
    pub label_font_size: Option<FontSize>,
    /// The `font::Id` of the number dialer's font.
    #[conrod(default = "None")]
    pub font_id: Option<Option<text::font::Id>>,
    /// The styling for each `Canvas`.
    #[conrod(default = "widget::canvas::Style::default()")]
    pub canvas: Option<widget::canvas::Style>,
}

/// The direction in which the tabs will be laid out.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Layout {
    /// Tabs will be laid out horizontally (left to right).
    Horizontal,
    /// Tabs will be laid out vertically (top to bottom).
    Vertical,
}


impl<'a> Tabs<'a> {

    /// Construct some new Canvas Tabs.
    pub fn new(tabs: &'a [(widget::Id, &'a str)]) -> Tabs<'a> {
        Tabs {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            tabs: tabs,
            maybe_starting_tab_idx: None,
        }
    }

    /// Set the initially selected tab with a Canvas via its widget::Id.
    pub fn starting_canvas(mut self, canvas_id: widget::Id) -> Self {
        let maybe_idx = self.tabs.iter().enumerate()
            .find(|&(_, &(id, _))| canvas_id == id)
            .map(|(idx, &(_, _))| idx);
        self.maybe_starting_tab_idx = maybe_idx;
        self
    }

    /// Set the padding for all edges.
    pub fn pad(self, pad: Scalar) -> Tabs<'a> {
        self.pad_left(pad).pad_right(pad).pad_top(pad).pad_bottom(pad)
    }

    /// Layout the tabs horizontally.
    pub fn layout_horizontally(mut self) -> Self {
        self.style.layout = Some(Layout::Horizontal);
        self
    }

    /// Layout the tabs vertically.
    pub fn layout_vertically(mut self) -> Self {
        self.style.layout = Some(Layout::Vertical);
        self
    }

    /// Build the `Tabs` widget with the given styling for its `Canvas`ses.
    pub fn canvas_style(mut self, style: widget::canvas::Style) -> Self {
        self.style.canvas = Some(style);
        self
    }

    /// Map the `NumberDialer`'s `widget::canvas::Style` to a new `widget::canvas::Style`.
    fn map_canvas_style<F>(mut self, map: F) -> Self
        where F: FnOnce(widget::canvas::Style) -> widget::canvas::Style,
    {
        self.style.canvas = Some(map({
            self.style.canvas.clone()
                .unwrap_or_else(widget::canvas::Style::default)
        }));
        self
    }

    /// If the `Tabs` has some `widget::canvas::Style`, assign the left padding.
    pub fn pad_left(self, pad: Scalar) -> Self {
        self.map_canvas_style(|mut style| { style.pad_left = Some(pad); style })
    }

    /// If the `Tabs` has some `widget::canvas::Style`, assign the left padding.
    pub fn pad_right(self, pad: Scalar) -> Self {
        self.map_canvas_style(|mut style| { style.pad_right = Some(pad); style })
    }

    /// If the `Tabs` has some `widget::canvas::Style`, assign the left padding.
    pub fn pad_bottom(self, pad: Scalar) -> Self {
        self.map_canvas_style(|mut style| { style.pad_bottom = Some(pad); style })
    }

    /// If the `Tabs` has some `widget::canvas::Style`, assign the left padding.
    pub fn pad_top(self, pad: Scalar) -> Self {
        self.map_canvas_style(|mut style| { style.pad_top = Some(pad); style })
    }

    /// The width of a vertical `Tabs` selection bar, or the height of a horizontal one.
    pub fn bar_thickness(mut self, thickness: Scalar) -> Self {
        self.style.bar_thickness = Some(Some(thickness));
        self
    }

    builder_methods!{
        pub starting_tab_idx { maybe_starting_tab_idx = Some(usize) }
        pub label_color { style.label_color = Some(Color) }
        pub label_font_size { style.label_font_size = Some(FontSize) }
    }

}


impl<'a> Widget for Tabs<'a> {
    type State = State;
    type Style = Style;
    type Event = ();

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State {
            tabs: Vec::new(),
            maybe_selected_tab_idx: None,
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// The area on which child widgets will be placed when using the `Place` Positionable methods.
    fn kid_area(&self, args: widget::KidAreaArgs<Self>) -> widget::KidArea {
        let widget::KidAreaArgs { rect, style, theme, fonts } = args;
        let font_size = style.label_font_size(theme);
        let bar_thickness = style.bar_thickness(theme);
        let canvas_style = style.canvas(theme);
        match style.layout(theme) {
            Layout::Horizontal => {
                let tab_bar_h = horizontal_tab_bar_h(bar_thickness, font_size as Scalar);
                widget::KidArea {
                    rect: rect.pad_top(tab_bar_h),
                    pad: canvas_style.padding(theme),
                }
            },
            Layout::Vertical => {
                let max_text_width = style.font_id(theme)
                    .or(fonts.ids().next())
                    .and_then(|id| fonts.get(id))
                    .map(|font| max_text_width(self.tabs.iter(), font_size, font))
                    .unwrap_or(0.0);
                let tab_bar_w = vertical_tab_bar_w(bar_thickness, max_text_width as Scalar);
                widget::KidArea {
                    rect: rect.pad_left(tab_bar_w),
                    pad: canvas_style.padding(theme),
                }
            },
        }
    }

    /// Update the state of the Tabs.
    fn update(self, args: widget::UpdateArgs<Self>) {
        let widget::UpdateArgs { id, state, rect, style, mut ui, .. } = args;
        let Tabs { tabs, maybe_starting_tab_idx, .. } = self;
        let layout = style.layout(&ui.theme);
        let font_size = style.label_font_size(&ui.theme);
        let canvas_style = style.canvas(&ui.theme);
        let max_text_width = style.font_id(&ui.theme)
            .or(ui.fonts.ids().next())
            .and_then(|id| ui.fonts.get(id))
            .map(|font| max_text_width(self.tabs.iter(), font_size, font))
            .unwrap_or(0.0);

        // Calculate the area of the tab bar.
        let font_height = font_size as Scalar;
        let bar_thickness = style.bar_thickness(&ui.theme);
        let dim = rect.dim();
        let rel_tab_bar_rect =
            rel_tab_bar_area(dim, layout, bar_thickness, font_height, max_text_width);

        // Update the `tabs` **Vec** stored within our **State**, only if there have been changes.
        let tabs_have_changed = state.tabs.len() != tabs.len()
            || state.tabs.iter().zip(tabs.iter())
                .any(|(tab, &(id, _))| tab.id != id);
        if tabs_have_changed {
            state.update(|state| {
                let num_tabs = state.tabs.len();
                let num_new_tabs = tabs.len();

                // Ensure the `widget::Id`s are in the same order as the given tabs slice.
                for (tab, &(id, _)) in state.tabs.iter_mut().zip(tabs.iter()) {
                    tab.id = id;
                }

                // If we have less tabs than we need, extend our `tabs` Vec.
                if num_tabs < num_new_tabs {
                    let id_gen = &mut ui.widget_id_generator();
                    let extension = tabs[num_tabs..].iter().map(|&(id, _)| Tab {
                        id: id,
                        button_id: id_gen.next(),
                    });
                    state.tabs.extend(extension);
                }
            });
        }


        // Instantiate the widgets associated with each Tab.
        let maybe_selected_tab_idx = {
            let color = canvas_style.color(&ui.theme);
            let border = canvas_style.border(&ui.theme);
            let border_color = canvas_style.border_color(ui.theme());
            let label_color = style.label_color(ui.theme());
            let mut maybe_selected_tab_idx = state.maybe_selected_tab_idx
                .or(maybe_starting_tab_idx)
                .or_else(|| if tabs.len() > 0 { Some(0) } else { None });
            let mut tab_rects = TabRects::new(tabs, layout, rel_tab_bar_rect);
            let mut i = 0;
            while let Some((tab_rect, _, label)) = tab_rects.next_with_id_and_label() {
                use {Colorable, Borderable, Labelable, Positionable, Sizeable};
                let tab = state.tabs[i];
                let (xy, dim) = tab_rect.xy_dim();

                // We'll instantiate each selectable **Tab** as a **Button** widget.
                if widget::Button::new()
                    .wh(dim)
                    .xy_relative_to(id, xy)
                    .color(color)
                    .border(border)
                    .border_color(border_color)
                    .label(label)
                    .label_color(label_color)
                    .parent(id)
                    .set(tab.button_id, &mut ui)
                    .was_clicked()
                {
                    maybe_selected_tab_idx = Some(i);
                }

                i += 1;
            }
            maybe_selected_tab_idx
        };

        if state.maybe_selected_tab_idx != maybe_selected_tab_idx {
            state.update(|state| state.maybe_selected_tab_idx = maybe_selected_tab_idx);
        }

        // If we do have some selected tab, we'll draw a Canvas for it.
        if let Some(selected_idx) = maybe_selected_tab_idx {
            use position::{Positionable, Sizeable};

            let &(child_id, _) = &tabs[selected_idx];
            widget::Canvas::new()
                .with_style(canvas_style)
                .kid_area_wh_of(id)
                .middle_of(id)
                .parent(id)
                .set(child_id, &mut ui);
        }

    }

}


/// Calculate the max text width yielded by a string in the tabs slice.
fn max_text_width<'a, I>(tabs: I, font_size: FontSize, font: &text::Font) -> Scalar
    where I: Iterator<Item=&'a (widget::Id, &'a str)>,
{
    tabs.fold(0.0, |max_w, &(_, string)| {
        let w = text::line::width(string, font, font_size);
        if w > max_w { w } else { max_w }
    })
}


/// Calculate the dimensions and position of the Tab Bar relative to the center of the widget.
fn rel_tab_bar_area(dim: Dimensions,
                    layout: Layout,
                    maybe_bar_thickness: Option<Scalar>,
                    font_size: f64,
                    max_text_width: f64) -> Rect
{
    match layout {
        Layout::Horizontal => {
            let w = dim[0];
            let h = horizontal_tab_bar_h(maybe_bar_thickness, font_size);
            let x = 0.0;
            let y = dim[1] / 2.0 - h / 2.0;
            Rect::from_xy_dim([x, y], [w, h])
        },
        Layout::Vertical => {
            let w = vertical_tab_bar_w(maybe_bar_thickness, max_text_width);
            let h = dim[1];
            let x = -dim[0] / 2.0 + w / 2.0;
            let y = 0.0;
            Rect::from_xy_dim([x, y], [w, h])
        },
    }
}

/// The height of a horizontally laid out tab bar area.
fn horizontal_tab_bar_h(maybe_bar_thickness: Option<Scalar>, font_size: Scalar) -> Scalar {
    maybe_bar_thickness.unwrap_or_else(|| font_size + TAB_BAR_LABEL_PADDING * 2.0)
}

/// The width of a vertically laid out tab bar area.
fn vertical_tab_bar_w(maybe_bar_thickness: Option<Scalar>, max_text_width: Scalar) -> Scalar {
    maybe_bar_thickness.unwrap_or_else(|| max_text_width + TAB_BAR_LABEL_PADDING * 2.0)
}

fn tab_dim(num_tabs: usize, tab_bar_dim: Dimensions, layout: Layout) -> Dimensions {
    let width_multi = 1.0 / num_tabs as Scalar;
    match layout {
        Layout::Horizontal =>
            [width_multi * tab_bar_dim[0], tab_bar_dim[1]],
        Layout::Vertical =>
            [tab_bar_dim[0], width_multi * tab_bar_dim[1]],
    }
}


impl<'a> ::color::Colorable for Tabs<'a> {
    fn color(self, color: Color) -> Self {
        self.map_canvas_style(|mut style| {
            style.color = Some(color);
            style
        })
    }
}

impl<'a> ::border::Borderable for Tabs<'a> {
    fn border(self, width: f64) -> Self {
        self.map_canvas_style(|mut style| {
            style.border = Some(width);
            style
        })
    }
    fn border_color(self, color: Color) -> Self {
        self.map_canvas_style(|mut style| {
            style.border_color = Some(color);
            style
        })
    }
}

/// An iterator yielding the **Rect** for each Tab in the given list.
pub struct TabRects<'a> {
    tabs: std::slice::Iter<'a, (widget::Id, &'a str)>,
    tab_dim: Dimensions,
    next_xy: Point,
    xy_step: Point,
}

impl<'a> TabRects<'a> {

    /// Construct a new **TabRects** iterator.
    pub fn new(tabs: &'a [(widget::Id, &'a str)],
               layout: Layout,
               rel_tab_bar_rect: Rect) -> Self
    {
        let num_tabs = tabs.len();
        let tab_bar_dim = rel_tab_bar_rect.dim();
        let tab_dim = tab_dim(num_tabs, tab_bar_dim, layout);
        let unpositioned_tab_rect = Rect::from_xy_dim([0.0, 0.0], tab_dim);
        let start_tab_rect = unpositioned_tab_rect.top_left_of(rel_tab_bar_rect);
        let start_xy = start_tab_rect.xy();
        let xy_step = match layout {
            Layout::Horizontal => [tab_dim[0], 0.0],
            Layout::Vertical => [0.0, tab_dim[1]],
        };
        TabRects {
            tabs: tabs.iter(),
            tab_dim: tab_dim,
            next_xy: start_xy,
            xy_step: xy_step,
        }
    }

    /// Yield the next **Tab** **Rect**, along with the associated ID and label.
    pub fn next_with_id_and_label(&mut self) -> Option<(Rect, widget::Id, &'a str)> {
        let TabRects { ref mut tabs, tab_dim, ref mut next_xy, xy_step } = *self;
        tabs.next().map(|&(id, label)| {
            let xy = *next_xy;
            *next_xy = utils::vec2_add(*next_xy, xy_step);
            let rect = Rect::from_xy_dim(xy, tab_dim);
            (rect, id, label)
        })
    }

}
