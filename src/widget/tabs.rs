
use ::{GlyphCache, Scalar};
use color::Color;
use elmesque::Element;
use graphics::character::CharacterCache;
use label::FontSize;
use position::{Dimensions, Point};
use super::canvas::{self, Canvas};
use theme::Theme;
use widget::{self, Widget};


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
    /// An owned, ordered list of the Widget/String pairs.
    tabs: Vec<(widget::Id, String)>,
    /// The widget::Id of the currently selected Canvas.
    maybe_selected_tab_idx: Option<usize>,
    /// The current interaction with the Tabs.
    interaction: Interaction,
    /// The dimensions of the tab bar.
    tab_bar_dim: Dimensions,
    /// The position of the tab bar relative to the position of the Tabs widget.
    tab_bar_rel_xy: Point,
}

/// The padding between the edge of the title bar and the title bar's label.
const TAB_BAR_LABEL_PADDING: f64 = 4.0;

/// The current interaction with the Tabs.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Interaction {
    /// The normal state of interaction.
    Normal,
    /// Some element is currently highlighted.
    Highlighted(Elem),
    /// Some element is currently clicked.
    Clicked(Elem),
}

/// The elements that make up the Tabs widget.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Elem {
    /// A tab, with an index specifying which one.
    Tab(usize),
    /// The body of the tabbed area.
    Body,
}


/// The styling for Canvas Tabs.
#[derive(Clone, Debug, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Style {
    /// The direction in which the tabs will be laid out.
    pub maybe_layout: Option<Layout>,
    /// The width of the tab bar.
    /// For horizontally laid out tabs, this is the height of the bar.
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
#[derive(Copy, Clone, Debug, PartialEq, RustcDecodable, RustcEncodable)]
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

    fn common(&self) -> &widget::CommonBuilder { &self.common }
    fn common_mut(&mut self) -> &mut widget::CommonBuilder { &mut self.common }
    fn unique_kind(&self) -> &'static str { "Tabs" }
    fn init_state(&self) -> State {
        State {
            tabs: Vec::new(),
            maybe_selected_tab_idx: None,
            interaction: Interaction::Normal,
            tab_bar_dim: [0.0, 0.0],
            tab_bar_rel_xy: [0.0, 0.0],
        }
    }
    fn style(&self) -> Style { self.style.clone() }

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
        let (xy, dim) = rect.xy_dim();
        let layout = style.layout(ui.theme());
        let font_size = style.font_size(ui.theme());
        let max_text_width = max_text_width(self.tabs.iter(), font_size, ui.glyph_cache());

        // Calculate the area of the tab bar.
        let (tab_bar_dim, tab_bar_rel_xy) =
            tab_bar_area(dim, layout, style.maybe_bar_width, font_size as f64, max_text_width);

        let num_tabs = self.tabs.len();
        let width_multi = if num_tabs == 0 { 1.0 } else { 1.0 / num_tabs as f64 };
        let tab_dim = match layout {
            Layout::Horizontal =>
                [width_multi * tab_bar_dim[0], tab_bar_dim[1]],
            Layout::Vertical =>
                [tab_bar_dim[0], width_multi * tab_bar_dim[1]],
        };

        // Get the mouse relative to the `Tabs` widget's position.
        let maybe_mouse = ui.input().maybe_mouse.map(|mouse| mouse.relative_to(xy));

        // Determine whether the mouse is currently over part of the widget..
        let is_over_elem = || if let Some(mouse) = maybe_mouse {
            use position::is_over_rect;
            if is_over_rect([0.0, 0.0], dim, mouse.xy) {
                if is_over_rect(tab_bar_rel_xy, tab_bar_dim, mouse.xy) {
                    match layout {
                        Layout::Horizontal => {
                            let start_tab_x = -dim[0] / 2.0 + tab_dim[0] / 2.0;
                            for i in 0..self.tabs.len() {
                                let tab_x = start_tab_x + i as f64 * tab_dim[0];
                                let tab_xy = [tab_x, tab_bar_rel_xy[1]];
                                if is_over_rect(tab_xy, tab_dim, mouse.xy) {
                                    return Some(Elem::Tab(i));
                                }
                            }
                            Some(Elem::Body)
                        },
                        Layout::Vertical => {
                            let start_tab_y = dim[1] / 2.0 - tab_dim[1] / 2.0;
                            for i in 0..self.tabs.len() {
                                let tab_y = start_tab_y - i as f64 * tab_dim[1];
                                let tab_xy = [tab_bar_rel_xy[0], tab_y];
                                if is_over_rect(tab_xy, tab_dim, mouse.xy) {
                                    return Some(Elem::Tab(i));
                                }
                            }
                            Some(Elem::Body)
                        },
                    }
                } else {
                    Some(Elem::Body)
                }
            } else {
                None
            }
        } else {
            None
        };

        // Determine the new current `Interaction` for the widget.
        let new_interaction = if let Some(mouse) = maybe_mouse {
            use mouse::ButtonPosition::{Down, Up};
            use self::Interaction::{Normal, Highlighted, Clicked};
            match (is_over_elem(), state.view().interaction, mouse.left.position) {
                (Some(_),    Normal,          Down) => Normal,
                (Some(elem), _,               Up)   => Highlighted(elem),
                (Some(elem), Highlighted(_),  Down) => Clicked(elem),
                (_,          Clicked(elem),   Down) => Clicked(elem),
                _                                   => Normal,
            }
        } else {
            Interaction::Normal
        };

        // Determine the currently selected tab by comparing our current and previous interactions.
        let maybe_selected_tab_idx = match (state.view().interaction, new_interaction) {
            (Interaction::Clicked(Elem::Tab(prev_idx)), Interaction::Highlighted(Elem::Tab(idx)))
                if prev_idx == idx => Some(idx),
            _ =>
                if let Some(idx) = state.view().maybe_selected_tab_idx { Some(idx) }
                else if let Some(idx) = self.maybe_starting_tab_idx { Some(idx) }
                else if self.tabs.len() > 0 { Some(0) }
                else { None },
        };

        // If we do have some selected tab, we'll draw a Canvas for it.
        if let Some(selected_idx) = maybe_selected_tab_idx {
            use position::{Positionable, Sizeable};

            let &(child_id, _) = &self.tabs[selected_idx];
            let mut canvas = Canvas::new();
            let canvas_dim = match style.layout(ui.theme()) {
                Layout::Horizontal => [dim[0], dim[1] - tab_bar_dim[1]],
                Layout::Vertical   => [dim[0] - tab_bar_dim[0], dim[1]],
            };
            canvas.style = style.canvas.clone();
            canvas
                .show_title_bar(false)
                .dim(canvas_dim)
                .floating(false)
                .middle_of(idx)
                .parent(Some(idx))
                .set(child_id, &mut ui);
        }

        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        if state.view().tab_bar_dim != tab_bar_dim {
            state.update(|state| state.tab_bar_dim = tab_bar_dim);
        }

        if state.view().tab_bar_rel_xy != tab_bar_rel_xy {
            state.update(|state| state.tab_bar_rel_xy = tab_bar_rel_xy);
        }

        if state.view().maybe_selected_tab_idx != maybe_selected_tab_idx {
            state.update(|state| state.maybe_selected_tab_idx = maybe_selected_tab_idx);
        }

        let tabs_have_changed = state.view().tabs.len() != self.tabs.len()
            || state.view().tabs.iter().zip(self.tabs.iter())
                .any(|(&(prev_id, ref prev_label), &(id, label))| {
                    prev_id != id || &prev_label[..] != label
                });

        if tabs_have_changed {
            state.update(|state| {
                state.tabs = self.tabs.iter().map(|&(id, s)| (id, s.to_owned())).collect();
            });
        }
    }


    /// Construct an Element from the given Button State.
    fn draw<C: CharacterCache>(args: widget::DrawArgs<Self, C>) -> Element {
        use elmesque::form::{self, collage, text};
        use elmesque::text::Text;

        let widget::DrawArgs { rect, state, style, theme, .. } = args;
        let State {
            ref tabs,
            ref interaction,
            maybe_selected_tab_idx,
            tab_bar_dim,
            tab_bar_rel_xy,
            ..
        } = *state;

        let (xy, dim) = rect.xy_dim();
        let frame = style.canvas.frame(theme);
        let inner_dim = ::vecmath::vec2_sub(dim, [frame * 2.0; 2]);
        let color = style.canvas.color(theme);
        let frame_color = style.canvas.frame_color(theme);
        let frame_form = form::rect(dim[0], dim[1]).filled(frame_color);
        let rect_form = form::rect(inner_dim[0], inner_dim[1]).filled(color);
        let font_size = style.font_size(theme);
        let label_color = style.label_color(theme);
        let layout = style.layout(theme);
        let maybe_highlighted_idx = match *interaction {
            Interaction::Highlighted(Elem::Tab(idx)) => Some(idx),
            _                                        => None,
        };
        let maybe_clicked_idx = match *interaction {
            Interaction::Clicked(Elem::Tab(idx)) => Some(idx),
            _                                    => None,
        };

        let base_forms = Some(frame_form).into_iter().chain(Some(rect_form));

        // Only draw the tab bar if we actually have some tabs.
        if !tabs.is_empty() {
            let num_tabs = tabs.len();
            let width_multi = 1.0 / num_tabs as f64;

            // Function for producing the rectangular tab forms.
            let rect_form = |dim: Dimensions, color: Color| form::rect(dim[0], dim[1]).filled(color);

            // Produces the label form for a tab.
            let label_form = |label: &str| text(Text::from_string(label.to_owned())
                                                .color(label_color)
                                                .height(font_size as f64));

            // Produces the correct color for the tab at the given `i`.
            let color = |i: usize|
                if      Some(i) == maybe_selected_tab_idx { color.clicked() }
                else if Some(i) == maybe_clicked_idx { color.clicked() }
                else if Some(i) == maybe_highlighted_idx { color.highlighted() }
                else { color };

            match layout {
                Layout::Horizontal => {
                    let tab_dim = [width_multi * tab_bar_dim[0], tab_bar_dim[1]];
                    let inner_tab_dim = ::vecmath::vec2_sub(tab_dim, [frame * 2.0; 2]);
                    let start_tab_x = -dim[0] / 2.0 + tab_dim[0] / 2.0;
                    let tab_bar_forms = tabs.iter().enumerate().flat_map(|(i, &(_, ref label))| {
                        let tab_x = start_tab_x + i as f64 * tab_dim[0];
                        let tab_xy = [tab_x, tab_bar_rel_xy[1]];
                        Some(rect_form(tab_dim, frame_color)).into_iter()
                            .chain(Some(rect_form(inner_tab_dim, color(i))))
                            .chain(Some(label_form(label)))
                            .map(move |form| form.shift(tab_xy[0], tab_xy[1]))
                    });
                    let form_chain = base_forms.chain(tab_bar_forms)
                        .map(move |form| form.shift(xy[0], xy[1]));
                    collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
                },

                Layout::Vertical => {
                    let tab_dim = [tab_bar_dim[0], width_multi * tab_bar_dim[1]];
                    let inner_tab_dim = ::vecmath::vec2_sub(tab_dim, [frame * 2.0; 2]);
                    let start_tab_y = dim[1] / 2.0 - tab_dim[1] / 2.0;
                    let tab_bar_forms = tabs.iter().enumerate().flat_map(|(i, &(_, ref label))| {
                        let tab_y = start_tab_y - i as f64 * tab_dim[1];
                        let tab_xy = [tab_bar_rel_xy[0], tab_y];
                        Some(rect_form(tab_dim, frame_color)).into_iter()
                            .chain(Some(rect_form(inner_tab_dim, color(i))))
                            .chain(Some(label_form(label)))
                            .map(move |form| form.shift(tab_xy[0], tab_xy[1]))
                    });
                    let form_chain = base_forms.chain(tab_bar_forms)
                        .map(move |form| form.shift(xy[0], xy[1]));
                    collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
                },
            }
        }
        // Otherwise, just draw the base forms.
        else {
            let form_chain = base_forms.map(|form| form.shift(xy[0], xy[1]));
            collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
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
fn tab_bar_area(dim: Dimensions,
                layout: Layout,
                maybe_bar_width: Option<Scalar>,
                font_size: f64,
                max_text_width: f64) -> (Dimensions, Point)
{
    match layout {
        Layout::Horizontal => {
            let h = horizontal_tab_bar_h(maybe_bar_width, font_size);
            let tab_bar_dim = [dim[0], h];
            let y = dim[1] / 2.0 - h / 2.0;
            let tab_bar_rel_xy = [0.0, y];
            (tab_bar_dim, tab_bar_rel_xy)
        },
        Layout::Vertical => {
            let w = vertical_tab_bar_w(maybe_bar_width, max_text_width);
            let tab_bar_dim = [w, dim[1]];
            let x = -dim[0] / 2.0 + w / 2.0;
            let tab_bar_rel_xy = [x, 0.0];
            (tab_bar_dim, tab_bar_rel_xy)
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
        self.maybe_layout.or(theme.maybe_tabs.as_ref().map(|default| {
            default.style.maybe_layout.unwrap_or(DEFAULT_LAYOUT)
        })).unwrap_or(DEFAULT_LAYOUT)
    }

    /// Get the color for the tab labels.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.maybe_tabs.as_ref().map(|default| {
            default.style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the font size for the tab labels.
    pub fn font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.maybe_tabs.as_ref().map(|default| {
            default.style.maybe_label_font_size.unwrap_or(theme.font_size_medium)
        })).unwrap_or(theme.font_size_medium)
    }

}


impl<'a> ::color::Colorable for Tabs<'a> {
    fn color(mut self, color: Color) -> Self {
        self.style.canvas.maybe_color = Some(color);
        self
    }
}

impl<'a> ::frame::Frameable for Tabs<'a> {
    fn frame(mut self, width: f64) -> Self {
        self.style.canvas.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.canvas.maybe_frame_color = Some(color);
        self
    }
}

