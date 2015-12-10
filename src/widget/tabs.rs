
use {
    Button,
    Canvas,
    CharacterCache,
    Color,
    Dimensions,
    FontSize,
    GlyphCache,
    NodeIndex,
    Point,
    Rect,
    Scalar,
    Theme,
    Widget,
};
use super::canvas;
use widget;


/// A wrapper around a list of canvasses that displays thema s a list of selectable tabs.
pub struct Tabs<'a> {
    tabs: &'a [(widget::Id, &'a str)],
    style: Style,
    common: widget::CommonBuilder,
    maybe_starting_tab_idx: Option<usize>,
}

/// The state to be cached within the Canvas.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// An owned, ordered list of the **Tab**s and their associated indices.
    tabs: Vec<Tab>,
    /// An index into the `tabs` slice that represents the currently selected Canvas.
    maybe_selected_tab_idx: Option<usize>,
    /// The relative location of the tab bar to the centre of the **Tabs** widget.
    tab_bar_rect: Rect,
}

/// A single **Tab** in the list owned by the **Tabs** **State**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tab {
    /// The public identifier, given by the user.
    id: widget::Id,
    /// The **Tab**'s selectable **Button**.
    button_idx: NodeIndex,
}

/// Unique kind for the widget type.
pub const KIND: widget::Kind = "Tabs";

/// The padding between the edge of the title bar and the title bar's label.
const TAB_BAR_LABEL_PADDING: f64 = 4.0;

/// The styling for Canvas Tabs.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    /// The direction in which the tabs will be laid out.
    pub maybe_layout: Option<Layout>,
    /// The width of the tab bar.
    ///
    /// For horizontally laid out tabs, this is the height of the bar.
    ///
    /// For vertically laid out tabs, this is the width of the bar.
    pub maybe_bar_width: Option<Scalar>,
    /// The color of the tabs' labels.
    pub maybe_label_color: Option<Color>,
    /// The font size for the tabs' labels.
    pub maybe_label_font_size: Option<FontSize>,
    /// Styling for each of the canvasses passed to the Canvas.
    pub canvas: canvas::Style,
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
            common: widget::CommonBuilder::new(),
            tabs: tabs,
            style: Style::new(),
            maybe_starting_tab_idx: None,
        }
    }

    /// Set the exact width of the tab bar.
    pub fn bar_width(mut self, width: Scalar) -> Self {
        self.style.maybe_bar_width = Some(width);
        self
    }

    /// Set the initially selected tab with an index for our tab list.
    pub fn starting_tab_idx(mut self, idx: usize) -> Self {
        self.maybe_starting_tab_idx = Some(idx);
        self
    }

    /// Set the initially selected tab with a Canvas via its widget::Id.
    pub fn starting_canvas(mut self, canvas_id: widget::Id) -> Self {
        let maybe_idx = self.tabs.iter().enumerate()
            .find(|&(_, &(id, _))| canvas_id == id)
            .map(|(idx, &(_, _))| idx);
        self.maybe_starting_tab_idx = maybe_idx;
        self
    }

    /// Layout the tabs horizontally.
    pub fn layout_horizontally(mut self) -> Self {
        self.style.maybe_layout = Some(Layout::Horizontal);
        self
    }

    /// Layout the tabs vertically.
    pub fn layout_vertically(mut self) -> Self {
        self.style.maybe_layout = Some(Layout::Vertical);
        self
    }

    /// The color of the tab labels.
    pub fn label_color(mut self, color: Color) -> Self {
        self.style.maybe_label_color = Some(color);
        self
    }

    /// The font size for the tab labels.
    pub fn label_font_size(mut self, size: FontSize) -> Self {
        self.style.maybe_label_font_size = Some(size);
        self
    }

    ///// Styling inherited by the inner canvas. /////

    /// The style that will be used for the selected Canvas.
    pub fn canvas_style(mut self, canvas_style: canvas::Style) -> Self {
        self.style.canvas = canvas_style;
        self
    }

    /// Set the padding from the left edge.
    pub fn pad_left(mut self, pad: Scalar) -> Tabs<'a> {
        self.style.canvas.padding.maybe_left = Some(pad);
        self
    }

    /// Set the padding from the right edge.
    pub fn pad_right(mut self, pad: Scalar) -> Tabs<'a> {
        self.style.canvas.padding.maybe_right = Some(pad);
        self
    }

    /// Set the padding from the top edge.
    pub fn pad_top(mut self, pad: Scalar) -> Tabs<'a> {
        self.style.canvas.padding.maybe_top = Some(pad);
        self
    }

    /// Set the padding from the bottom edge.
    pub fn pad_bottom(mut self, pad: Scalar) -> Tabs<'a> {
        self.style.canvas.padding.maybe_bottom = Some(pad);
        self
    }

    /// Set the padding for all edges.
    pub fn pad(self, pad: Scalar) -> Tabs<'a> {
        self.pad_left(pad).pad_right(pad).pad_top(pad).pad_bottom(pad)
    }

    /// Set the margin from the left edge.
    pub fn margin_left(mut self, mgn: Scalar) -> Tabs<'a> {
        self.style.canvas.margin.maybe_left = Some(mgn);
        self
    }

    /// Set the margin from the right edge.
    pub fn margin_right(mut self, mgn: Scalar) -> Tabs<'a> {
        self.style.canvas.margin.maybe_right = Some(mgn);
        self
    }

    /// Set the margin from the top edge.
    pub fn margin_top(mut self, mgn: Scalar) -> Tabs<'a> {
        self.style.canvas.margin.maybe_top = Some(mgn);
        self
    }

    /// Set the margin from the bottom edge.
    pub fn margin_bottom(mut self, mgn: Scalar) -> Tabs<'a> {
        self.style.canvas.margin.maybe_bottom = Some(mgn);
        self
    }

    /// Set the margin for all edges.
    pub fn margin(self, mgn: Scalar) -> Tabs<'a> {
        self.margin_left(mgn).margin_right(mgn).margin_top(mgn).margin_bottom(mgn)
    }

}


impl<'a> Widget for Tabs<'a> {
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
            tabs: Vec::new(),
            maybe_selected_tab_idx: None,
            tab_bar_rect: Rect::from_xy_dim([0.0, 0.0], [0.0, 0.0]),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// The area on which child widgets will be placed when using the `Place` Positionable methods.
    fn kid_area<C: CharacterCache>(&self, args: widget::KidAreaArgs<Self, C>) -> widget::KidArea {
        let widget::KidAreaArgs { rect, style, theme, glyph_cache } = args;
        let font_size = style.font_size(theme);
        match style.layout(theme) {
            Layout::Horizontal => {
                let tab_bar_h = horizontal_tab_bar_h(style.maybe_bar_width, font_size as Scalar);
                widget::KidArea {
                    rect: rect.pad_top(tab_bar_h),
                    pad: style.canvas.padding(theme),
                }
            },
            Layout::Vertical => {
                let max_text_width = max_text_width(self.tabs.iter(), font_size, glyph_cache);
                let tab_bar_w = vertical_tab_bar_w(style.maybe_bar_width, max_text_width as Scalar);
                widget::KidArea {
                    rect: rect.pad_left(tab_bar_w),
                    pad: style.canvas.padding(theme),
                }
            },
        }
    }

    /// Update the state of the Tabs.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let Tabs { tabs, maybe_starting_tab_idx, .. } = self;
        let layout = style.layout(ui.theme());
        let font_size = style.font_size(ui.theme());
        let max_text_width = max_text_width(tabs.iter(), font_size, ui.glyph_cache());

        // Calculate the area of the tab bar.
        let font_height = font_size as Scalar;
        let maybe_bar_width = style.maybe_bar_width;
        let dim = rect.dim();
        let rel_tab_bar_rect =
            rel_tab_bar_area(dim, layout, maybe_bar_width, font_height, max_text_width);

        // Update the `tabs` **Vec** stored within our **State**, only if there have been changes.
        let tabs_have_changed = state.view().tabs.len() != tabs.len()
            || state.view().tabs.iter().zip(tabs.iter())
                .any(|(tab, &(id, _))| tab.id != id);
        if tabs_have_changed {
            state.update(|state| {
                let num_tabs = state.tabs.len();
                let num_new_tabs = tabs.len();

                // Ensure our stored `tabs` Vec is no longer than the number of tabs given.
                if num_tabs < num_new_tabs {
                    state.tabs.truncate(num_new_tabs);
                }

                // Ensure the `widget::Id`s are in the same order as the given tabs slice.
                for (tab, &(id, _)) in state.tabs.iter_mut().zip(tabs.iter()) {
                    tab.id = id;
                }

                // If we have less tabs than we need, extend our `tabs` Vec.
                if num_tabs < num_new_tabs {
                    let extension = tabs[num_tabs..].iter().map(|&(id, _)| Tab {
                        id: id,
                        button_idx: ui.new_unique_node_index(),
                    });
                    state.tabs.extend(extension);
                }
            });
        }


        // Instantiate the widgets associated with each Tab.
        let maybe_selected_tab_idx = {
            let color = style.canvas.color(ui.theme());
            let frame = style.canvas.frame(ui.theme());
            let frame_color = style.canvas.frame_color(ui.theme());
            let label_color = style.label_color(ui.theme());
            let mut maybe_selected_tab_idx = state.view().maybe_selected_tab_idx
                .or(maybe_starting_tab_idx)
                .or_else(|| if tabs.len() > 0 { Some(0) } else { None });
            let mut tab_rects = TabRects::new(tabs, layout, rel_tab_bar_rect);
            let mut i = 0;
            while let Some((tab_rect, _, label)) = tab_rects.next_with_id_and_label() {
                use {Colorable, Frameable, Labelable, Positionable, Sizeable};
                let tab = state.view().tabs[i];
                let (xy, dim) = tab_rect.xy_dim();

                // We'll instantiate each selectable **Tab** as a **Button** widget.
                Button::new()
                    .dim(dim)
                    .xy_relative_to(idx, xy)
                    .color(color)
                    .frame(frame)
                    .frame_color(frame_color)
                    .label(label)
                    .label_color(label_color)
                    .parent(Some(idx))
                    .react(|| maybe_selected_tab_idx = Some(i))
                    .set(tab.button_idx, &mut ui);

                i += 1;
            }
            maybe_selected_tab_idx
        };

        if state.view().maybe_selected_tab_idx != maybe_selected_tab_idx {
            state.update(|state| state.maybe_selected_tab_idx = maybe_selected_tab_idx);
        }

        // If we do have some selected tab, we'll draw a Canvas for it.
        if let Some(selected_idx) = maybe_selected_tab_idx {
            use position::{Positionable, Sizeable};

            let &(child_id, _) = &tabs[selected_idx];
            Canvas::new()
                .with_style(style.canvas)
                .kid_area_dim_of(idx)
                .middle_of(idx)
                .parent(Some(idx))
                .set(child_id, &mut ui);
        }

    }

}


/// Calculate the max text width yielded by a string in the tabs slice.
fn max_text_width<'a, I, C>(tabs: I, font_size: FontSize, glyph_cache: &GlyphCache<C>) -> Scalar
    where I: Iterator<Item=&'a (widget::Id, &'a str)>,
          C: CharacterCache,
{
    tabs.fold(0.0, |max_w, &(_, string)| {
        let w = glyph_cache.width(font_size, &string);
        if w > max_w { w } else { max_w }
    })
}


/// Calculate the dimensions and position of the Tab Bar relative to the center of the widget.
fn rel_tab_bar_area(dim: Dimensions,
                    layout: Layout,
                    maybe_bar_width: Option<Scalar>,
                    font_size: f64,
                    max_text_width: f64) -> Rect
{
    match layout {
        Layout::Horizontal => {
            let w = dim[0];
            let h = horizontal_tab_bar_h(maybe_bar_width, font_size);
            let x = 0.0;
            let y = dim[1] / 2.0 - h / 2.0;
            Rect::from_xy_dim([x, y], [w, h])
        },
        Layout::Vertical => {
            let w = vertical_tab_bar_w(maybe_bar_width, max_text_width);
            let h = dim[1];
            let x = -dim[0] / 2.0 + w / 2.0;
            let y = 0.0;
            Rect::from_xy_dim([x, y], [w, h])
        },
    }
}

/// The height of a horizontally laid out tab bar area.
fn horizontal_tab_bar_h(maybe_bar_width: Option<Scalar>, font_size: Scalar) -> Scalar {
    maybe_bar_width.unwrap_or_else(|| font_size + TAB_BAR_LABEL_PADDING * 2.0)
}

/// The width of a vertically laid out tab bar area.
fn vertical_tab_bar_w(maybe_bar_width: Option<Scalar>, max_text_width: Scalar) -> Scalar {
    maybe_bar_width.unwrap_or_else(|| max_text_width + TAB_BAR_LABEL_PADDING * 2.0)
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


impl Style {

    /// Construct the default `Tabs` style.
    pub fn new() -> Style {
        Style {
            maybe_layout: None,
            maybe_bar_width: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
            canvas: canvas::Style::new(),
        }
    }

    /// Get the layout of the tabs for the `Tabs` widget.
    pub fn layout(&self, theme: &Theme) -> Layout {
        const DEFAULT_LAYOUT: Layout = Layout::Horizontal;
        self.maybe_layout.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_layout.unwrap_or(DEFAULT_LAYOUT)
        })).unwrap_or(DEFAULT_LAYOUT)
    }

    /// Get the color for the tab labels.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the font size for the tab labels.
    pub fn font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_label_font_size.unwrap_or(theme.font_size_medium)
        })).unwrap_or(theme.font_size_medium)
    }

}


impl<'a> ::color::Colorable for Tabs<'a> {
    fn color(mut self, color: Color) -> Self {
        self.style.canvas.framed_rectangle.maybe_color = Some(color);
        self
    }
}

impl<'a> ::frame::Frameable for Tabs<'a> {
    fn frame(mut self, width: f64) -> Self {
        self.style.canvas.framed_rectangle.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.canvas.framed_rectangle.maybe_frame_color = Some(color);
        self
    }
}

/// An iterator yielding the **Rect** for each Tab in the given list.
pub struct TabRects<'a> {
    tabs: ::std::slice::Iter<'a, (widget::Id, &'a str)>,
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
            *next_xy = ::vecmath::vec2_add(*next_xy, xy_step);
            let rect = Rect::from_xy_dim(xy, tab_dim);
            (rect, id, label)
        })
    }

}
