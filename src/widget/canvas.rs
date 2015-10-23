
use Scalar;
use clock_ticks::precise_time_ns;
use color::Color;
use elmesque::element::Element;
use graphics::character::CharacterCache;
use label::FontSize;
use mouse::Mouse;
use position::{self, Dimensions, Horizontal, Margin, Padding, Place, Point, Position, Rect};
use theme::Theme;
use widget::{self, Widget};
use ui::GlyphCache;


/// A widget designed to be a parent for other widgets.
pub struct Canvas<'a> {
    common: widget::CommonBuilder,
    /// The builder data related to the style of the Canvas.
    pub style: Style,
    maybe_title_bar_label: Option<&'a str>,
    show_title_bar: bool,
}

/// **Canvas** state to be cached.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    interaction: Interaction,
    maybe_title_bar: Option<TitleBar>,
    time_last_clicked: u64,
}

/// The padding between the edge of the title bar and the title bar's label.
const TITLE_BAR_LABEL_PADDING: f64 = 4.0;

/// State of the title bar.
#[derive(Clone, Debug, PartialEq)]
pub struct TitleBar {
    maybe_label: Option<(String, FontSize)>,
    /// The rectangle representing the **TitleBar**'s area relative to that of the **Canvas**.
    rect: Rect,
}

/// A builder for the padding of the area where child widgets will be placed.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct PaddingBuilder {
    /// The padding for the left of the area where child widgets will be placed.
    pub maybe_left: Option<Scalar>,
    /// The padding for the right of the area where child widgets will be placed.
    pub maybe_right: Option<Scalar>,
    /// The padding for the top of the area where child widgets will be placed.
    pub maybe_top: Option<Scalar>,
    /// The padding for the bottom of the area where child widgets will be placed.
    pub maybe_bottom: Option<Scalar>,
}

/// A builder for the margin of the area where child widgets will be placed.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct MarginBuilder {
    /// The margin for the left of the area where child widgets will be placed.
    pub maybe_left: Option<Scalar>,
    /// The margin for the right of the area where child widgets will be placed.
    pub maybe_right: Option<Scalar>,
    /// The margin for the top of the area where child widgets will be placed.
    pub maybe_top: Option<Scalar>,
    /// The margin for the bottom of the area where child widgets will be placed.
    pub maybe_bottom: Option<Scalar>,
}

/// Describes the style of a Canvas.
#[allow(missing_copy_implementations)]
#[derive(Clone, Debug, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Style {
    /// Color of the canvas
    pub maybe_color: Option<Color>,
    /// The width of the frame surrounding the canvas.
    pub maybe_frame: Option<f64>,
    /// The color of the canvas' frame.
    pub maybe_frame_color: Option<Color>,
    /// The font size of the canvas' title bar label.
    pub maybe_title_bar_font_size: Option<FontSize>,
    /// The alignment of the canvas' title bar label.
    pub maybe_title_bar_label_align: Option<Horizontal>,
    /// The color of the title bar's label.
    pub maybe_title_bar_label_color: Option<Color>,
    /// Padding of the kid area.
    pub padding: PaddingBuilder,
    /// Margin for the kid area.
    pub margin: MarginBuilder,
}

/// Describes an interaction with the Canvas.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Interaction {
    Normal,
    Highlighted(Elem),
    Clicked(Elem, Point),
}

/// Describes the different Elements that make up the Canvas.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Elem {
    TitleBar,
    WidgetArea,
}


impl<'a> Canvas<'a> {

    /// Construct a new Canvas builder.
    pub fn new() -> Canvas<'a> {
        Canvas {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
            maybe_title_bar_label: None,
            show_title_bar: false,
        }
    }

    /// Show or hide the title bar.
    pub fn show_title_bar(mut self, show: bool) -> Self {
        self.show_title_bar = show;
        self
    }

    /// Set the padding of the left of the area where child widgets will be placed.
    #[inline]
    pub fn pad_left(mut self, pad: Scalar) -> Self {
        self.style.padding.maybe_left = Some(pad);
        self
    }

    /// Set the padding of the right of the area where child widgets will be placed.
    #[inline]
    pub fn pad_right(mut self, pad: Scalar) -> Self {
        self.style.padding.maybe_right = Some(pad);
        self
    }

    /// Set the padding of the top of the area where child widgets will be placed.
    #[inline]
    pub fn pad_top(mut self, pad: Scalar) -> Self {
        self.style.padding.maybe_top = Some(pad);
        self
    }

    /// Set the padding of the bottom of the area where child widgets will be placed.
    #[inline]
    pub fn pad_bottom(mut self, pad: Scalar) -> Self {
        self.style.padding.maybe_bottom = Some(pad);
        self
    }

    /// Set the padding of the area where child widgets will be placed.
    #[inline]
    pub fn padding(self, pad: Padding) -> Self {
        self.pad_left(pad.left)
            .pad_right(pad.right)
            .pad_right(pad.top)
            .pad_right(pad.bottom)
    }

    /// Set the margin of the left of the area where child widgets will be placed.
    #[inline]
    pub fn margin_left(mut self, mgn: Scalar) -> Self {
        self.style.margin.maybe_left = Some(mgn);
        self
    }

    /// Set the margin of the right of the area where child widgets will be placed.
    #[inline]
    pub fn margin_right(mut self, mgn: Scalar) -> Self {
        self.style.margin.maybe_right = Some(mgn);
        self
    }

    /// Set the margin of the top of the area where child widgets will be placed.
    #[inline]
    pub fn margin_top(mut self, mgn: Scalar) -> Self {
        self.style.margin.maybe_top = Some(mgn);
        self
    }

    /// Set the margin of the bottom of the area where child widgets will be placed.
    #[inline]
    pub fn margin_bottom(mut self, mgn: Scalar) -> Self {
        self.style.margin.maybe_bottom = Some(mgn);
        self
    }

    /// Set the padding of the area where child widgets will be placed.
    #[inline]
    pub fn margin(self, mgn: Margin) -> Self {
        self.pad_left(mgn.left)
            .pad_right(mgn.right)
            .pad_right(mgn.top)
            .pad_right(mgn.bottom)
    }

}


impl<'a> Widget for Canvas<'a> {
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder { &self.common }
    fn common_mut(&mut self) -> &mut widget::CommonBuilder { &mut self.common }
    fn unique_kind(&self) -> &'static str { "Canvas" }
    fn init_state(&self) -> State {
        State {
            interaction: Interaction::Normal,
            time_last_clicked: precise_time_ns(),
            maybe_title_bar: None,
        }
    }
    fn style(&self) -> Style {
        self.style.clone()
    }

    fn default_position(&self, _theme: &Theme) -> Position {
        Position::Place(Place::Middle, None)
    }

    fn default_width<C: CharacterCache>(&self, theme: &Theme, _: &GlyphCache<C>) -> Scalar {
        const DEFAULT_WIDTH: Scalar = 160.0;
        theme.maybe_button.as_ref().map(|default| {
            default.common.maybe_width.unwrap_or(DEFAULT_WIDTH)
        }).unwrap_or(DEFAULT_WIDTH)
    }

    fn default_height(&self, theme: &Theme) -> Scalar {
        const DEFAULT_HEIGHT: Scalar = 80.0;
        theme.maybe_button.as_ref().map(|default| {
            default.common.maybe_height.unwrap_or(DEFAULT_HEIGHT)
        }).unwrap_or(DEFAULT_HEIGHT)
    }

    /// The title bar area at which the Canvas can be clicked and dragged.
    /// The position of the area should be relative to the center of the widget..
    fn drag_area(&self, dim: Dimensions, style: &Style, theme: &Theme) -> Option<Rect> {
        if self.show_title_bar {
            let font_size = style.title_bar_font_size(theme);
            let (h, y) = title_bar_h_rel_y(dim[1], font_size);
            Some(Rect::from_xy_dim([0.0, y], [dim[0], h]))
        } else {
            None
        }
    }

    /// The area of the widget below the title bar, upon which child widgets will be placed.
    fn kid_area<C: CharacterCache>(&self, args: widget::KidAreaArgs<Self, C>) -> widget::KidArea {
        let widget::KidAreaArgs { rect, style, theme, .. } = args;
        if self.show_title_bar {
            let font_size = style.title_bar_font_size(theme);
            let title_bar = title_bar(rect, font_size);
            widget::KidArea {
                rect: rect.pad_top(title_bar.h()),
                pad: style.padding(theme),
            }
        } else {
            widget::KidArea {
                rect: rect,
                pad: style.padding(theme),
            }
        }
    }

    /// Update the state of the Canvas.
    fn update<C>(self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { state, rect, ui, .. } = args;
        let maybe_mouse = ui.input().maybe_mouse;
        let title_bar_font_size = self.style.title_bar_font_size(ui.theme());
        let maybe_title_bar_rect = if self.show_title_bar {
            Some(title_bar(rect, title_bar_font_size))
        } else {
            None
        };

        // If there is new mouse state, check for a new interaction.
        let new_interaction = if let Some(mouse) = maybe_mouse {
            let is_over_elem = is_over(rect, maybe_title_bar_rect, mouse.xy);
            get_new_interaction(is_over_elem, state.view().interaction, mouse)
        } else {
            Interaction::Normal
        };

        // If the canvas was clicked, dragged or released, update the time_last_clicked.
        let new_time_last_clicked = match (state.view().interaction, new_interaction) {
            (Interaction::Highlighted(_), Interaction::Clicked(_, _)) |
            (Interaction::Clicked(_, _), Interaction::Highlighted(_)) |
            (Interaction::Clicked(_, _), Interaction::Clicked(_, _))  => precise_time_ns(),
            _ => state.view().time_last_clicked,
        };

        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        if state.view().time_last_clicked != new_time_last_clicked {
            state.update(|state| state.time_last_clicked = new_time_last_clicked);
        }

        let title_bar_has_changed = match state.view().maybe_title_bar {
            None => self.show_title_bar,
            Some(ref title_bar) => {
                Some(title_bar.rect) != maybe_title_bar_rect
                || match title_bar.maybe_label {
                    None => false,
                    Some((ref label, font_size)) => {
                        Some(&label[..]) != self.maybe_title_bar_label
                        || font_size != title_bar_font_size
                    },
                }
            },
        };

        if title_bar_has_changed {
            state.update(|state| {
                state.maybe_title_bar = maybe_title_bar_rect.map(|rect| TitleBar {
                    rect: rect,
                    maybe_label: self.maybe_title_bar_label.as_ref()
                        .map(|label| (label.to_string(), title_bar_font_size)),
                });
            });
        }
    }

    /// Draw the canvas.
    fn draw<C: CharacterCache>(args: widget::DrawArgs<Self, C>) -> Element {
        use elmesque::form::{self, collage, text};

        let widget::DrawArgs { rect, state, style, theme, glyph_cache, .. } = args;

        let frame = style.frame(theme);
        let inner = rect.sub_frame(frame);
        let color = style.color(theme);
        let frame_color = style.frame_color(theme);
        let frame_form = form::rect(rect.w(), rect.h()).filled(frame_color);
        let rect_form = form::rect(inner.w(), inner.h()).filled(color);

        // Check whether or not to draw the title bar.
        let maybe_title_bar_form = if let Some(ref title_bar) = state.maybe_title_bar {
            let inner = title_bar.rect.sub_frame(frame);
            let (_, rel_y, w, h) = title_bar.rect.x_y_w_h();
            let (inner_w, inner_h) = inner.w_h();
            let title_bar_frame_form = form::rect(w, h).filled(frame_color);
            let title_bar_rect_form = form::rect(inner_w, inner_h).filled(color);
            // Check whether or not to draw the title bar's label.
            let maybe_label_form = title_bar.maybe_label.as_ref().map(|&(ref label, font_size)| {
                use elmesque::text::Text;
                let label_color = style.title_bar_label_color(theme);
                let align = style.title_bar_label_align(theme);
                let label_width = glyph_cache.width(font_size, label);
                let label_x = align.to(inner_w, label_width) + TITLE_BAR_LABEL_PADDING;
                text(Text::from_string(label.clone())
                        .color(label_color)
                        .height(font_size as f64)).shift_x(label_x)
            });
            Some(Some(title_bar_frame_form).into_iter()
                .chain(Some(title_bar_rect_form).into_iter())
                .chain(maybe_label_form)
                .map(move |form| form.shift_y(rel_y)))
        } else {
            None
        };

        // Chain the Canvas' Forms together.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(rect_form).into_iter())
            .chain(maybe_title_bar_form.into_iter().flat_map(|it| it))
            .map(|form| form.shift(rect.x(), rect.y()));

        // Construct the renderable element.
        collage(rect.w() as i32, rect.h() as i32, form_chain.collect())
    }

}


/// The height and relative y coordinate of a Canvas' title bar given some canvas height and font
/// size for the title bar.
fn title_bar_h_rel_y(canvas_h: Scalar, font_size: FontSize) -> (Scalar, Scalar) {
    let h = font_size as Scalar + TITLE_BAR_LABEL_PADDING * 2.0;
    let rel_y = position::align_top_of(canvas_h, h);
    (h, rel_y)
}

/// The Rect for the Canvas' title bar.
fn title_bar(canvas: Rect, font_size: FontSize) -> Rect {
    let (c_w, c_h) = canvas.w_h();
    let (h, rel_y) = title_bar_h_rel_y(c_h, font_size);
    let xy = [0.0, rel_y];
    let dim = [c_w, h];
    Rect::from_xy_dim(xy, dim)
}


/// Is the mouse over the canvas, if so which Elem.
fn is_over(canvas: Rect, maybe_title_bar_rect: Option<Rect>, mouse_xy: Point) -> Option<Elem> {
    if let Some(rect) = maybe_title_bar_rect {
        if rect.is_over(mouse_xy) {
            return Some(Elem::TitleBar);
        }
    }
    if canvas.is_over(mouse_xy) {
        Some(Elem::WidgetArea)
    } else {
        None
    }
}


/// Determine the new interaction given the mouse state and previous interaction.
fn get_new_interaction(is_over_elem: Option<Elem>, prev: Interaction, mouse: Mouse) -> Interaction {
    use mouse::ButtonPosition::{Down, Up};
    use self::Interaction::{Normal, Highlighted, Clicked};
    match (is_over_elem, prev, mouse.left.position) {
        (Some(_),    Normal,          Down)  => Normal,
        (Some(elem), _,               Up)    => Highlighted(elem),
        (Some(elem), Highlighted(_),  Down)  => Clicked(elem, mouse.xy),
        (Some(_),    Clicked(elem, _), Down) => Clicked(elem, mouse.xy),
        (None,       Clicked(elem, _), Down) => Clicked(elem, mouse.xy),
        _                                    => Normal,
    }
}


impl Style {

    /// Construct a default Canvas Style.
    pub fn new() -> Style {
        Style {
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_title_bar_font_size: None,
            maybe_title_bar_label_align: None,
            maybe_title_bar_label_color: None,
            padding: PaddingBuilder {
                maybe_left: None,
                maybe_right: None,
                maybe_top: None,
                maybe_bottom: None,
            },
            margin: MarginBuilder {
                maybe_left: None,
                maybe_right: None,
                maybe_top: None,
                maybe_bottom: None,
            },
        }
    }

    /// Get the color for the Canvas' Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.maybe_canvas.as_ref().map(|default| {
            default.style.maybe_color.unwrap_or(theme.background_color)
        })).unwrap_or(theme.background_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.maybe_canvas.as_ref().map(|default| {
            default.style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.maybe_canvas.as_ref().map(|default| {
            default.style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the font size of the title bar.
    pub fn title_bar_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_title_bar_font_size.or(theme.maybe_canvas.as_ref().map(|default| {
            default.style.maybe_title_bar_font_size.unwrap_or(theme.font_size_medium)
        })).unwrap_or(theme.font_size_medium)
    }

    /// Get the alignment of the title bar label.
    pub fn title_bar_label_align(&self, theme: &Theme) -> Horizontal {
        const DEFAULT_ALIGN: Horizontal = Horizontal::Middle;
        self.maybe_title_bar_label_align.or(theme.maybe_canvas.as_ref().map(|default| {
            default.style.maybe_title_bar_label_align.unwrap_or(DEFAULT_ALIGN)
        })).unwrap_or(DEFAULT_ALIGN)
    }

    /// Get the color of the title bar label.
    pub fn title_bar_label_color(&self, theme: &Theme) -> Color {
        self.maybe_title_bar_label_color.or(theme.maybe_canvas.as_ref().map(|default| {
            default.style.maybe_title_bar_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the Padding for the Canvas' kid area.
    pub fn padding(&self, theme: &Theme) -> position::Padding {
        position::Padding {
            top: self.padding.maybe_top.or(theme.maybe_canvas.as_ref().map(|default| {
                default.style.padding.maybe_top.unwrap_or(theme.padding.top)
            })).unwrap_or(theme.padding.top),
            bottom: self.padding.maybe_bottom.or(theme.maybe_canvas.as_ref().map(|default| {
                default.style.padding.maybe_bottom.unwrap_or(theme.padding.bottom)
            })).unwrap_or(theme.padding.bottom),
            left: self.padding.maybe_left.or(theme.maybe_canvas.as_ref().map(|default| {
                default.style.padding.maybe_left.unwrap_or(theme.padding.left)
            })).unwrap_or(theme.padding.left),
            right: self.padding.maybe_right.or(theme.maybe_canvas.as_ref().map(|default| {
                default.style.padding.maybe_right.unwrap_or(theme.padding.right)
            })).unwrap_or(theme.padding.right),
        }
    }

    /// Get the Margin for the Canvas' kid area.
    pub fn margin(&self, theme: &Theme) -> position::Margin {
        position::Margin {
            top: self.margin.maybe_top.or(theme.maybe_canvas.as_ref().map(|default| {
                default.style.margin.maybe_top.unwrap_or(theme.margin.top)
            })).unwrap_or(theme.margin.top),
            bottom: self.margin.maybe_bottom.or(theme.maybe_canvas.as_ref().map(|default| {
                default.style.margin.maybe_bottom.unwrap_or(theme.margin.bottom)
            })).unwrap_or(theme.margin.bottom),
            left: self.margin.maybe_left.or(theme.maybe_canvas.as_ref().map(|default| {
                default.style.margin.maybe_left.unwrap_or(theme.margin.left)
            })).unwrap_or(theme.margin.left),
            right: self.margin.maybe_right.or(theme.maybe_canvas.as_ref().map(|default| {
                default.style.margin.maybe_right.unwrap_or(theme.margin.right)
            })).unwrap_or(theme.margin.right),
        }
    }

}


impl<'a> ::color::Colorable for Canvas<'a> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

impl<'a> ::frame::Frameable for Canvas<'a> {
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
        self
    }
}

impl<'a> ::label::Labelable<'a> for Canvas<'a> {
    fn label(mut self, text: &'a str) -> Self {
        self.maybe_title_bar_label = Some(text);
        self.show_title_bar = true;
        self
    }
    fn label_color(mut self, color: Color) -> Self {
        self.style.maybe_title_bar_label_color = Some(color);
        self
    }
    fn label_font_size(mut self, size: FontSize) -> Self {
        self.style.maybe_title_bar_font_size = Some(size);
        self
    }
}

