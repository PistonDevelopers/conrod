
use Scalar;
use clock_ticks::precise_time_ns;
use color::Color;
use elmesque::element::Element;
use graphics::character::CharacterCache;
use label::FontSize;
use mouse::Mouse;
use position::{self, Dimensions, Horizontal, Margin, Padding, Place, Point, Position};
use super::drag;
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

/// Canvas state to be cached.
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
    y: Scalar,
    h: Scalar,
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

/// Describes the style of a Canvas Floating.
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

/// Describes an interaction with the Floating Canvas.
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
        self.common.maybe_width.or(theme.maybe_button.as_ref().map(|default| {
            default.common.maybe_width.unwrap_or(DEFAULT_WIDTH)
        })).unwrap_or(DEFAULT_WIDTH)
    }

    fn default_height(&self, theme: &Theme) -> Scalar {
        const DEFAULT_HEIGHT: Scalar = 80.0;
        self.common.maybe_height.or(theme.maybe_button.as_ref().map(|default| {
            default.common.maybe_height.unwrap_or(DEFAULT_HEIGHT)
        })).unwrap_or(DEFAULT_HEIGHT)
    }

    /// If the title bar was clicked, capture the mouse.
    fn capture_mouse(prev: &State, new: &State) -> bool {
        use self::Interaction::{Highlighted, Clicked};
        match (prev.interaction, new.interaction) {
            (Highlighted(Elem::TitleBar), Clicked(Elem::TitleBar, _)) => true,
            _ => false,
        }
    }

    /// If the title bar was released, uncapture the mouse.
    fn uncapture_mouse(prev: &Self::State, new: &Self::State) -> bool {
        use self::Interaction::{Normal, Highlighted, Clicked};
        match (prev.interaction, new.interaction) {
            (Clicked(Elem::TitleBar, _), Highlighted(_)) |
            (Clicked(Elem::TitleBar, _), Normal)         => true,
            _ => false,
        }
    }

    /// The title bar area at which the Canvas can be clicked and dragged.
    fn drag_area(&self,
                 xy: Point,
                 dim: Dimensions,
                 style: &Style,
                 theme: &Theme) -> Option<drag::Area>
    {
        if self.show_title_bar {
            let font_size = style.title_bar_font_size(theme);
            let (h, y) = title_bar_h_y(dim, font_size as f64);
            Some(drag::Area {
                xy: [xy[0], y],
                dim: [dim[0], h],
            })
        } else {
            None
        }
    }

    /// The area of the widget below the title bar, upon which child widgets will be placed.
    fn kid_area(state: &widget::State<State>, style: &Style, theme: &Theme) -> widget::KidArea {
        let widget::State { ref state, xy, dim, .. } = *state;
        match state.maybe_title_bar {
            None => widget::KidArea {
                xy: xy,
                dim: dim,
                pad: style.padding(theme),
            },
            Some(ref title_bar) => widget::KidArea {
                xy: [xy[0], title_bar.y],
                dim: [dim[0], dim[1] - title_bar.h],
                pad: style.padding(theme),
            },
        }
    }

    /// Update the state of the Canvas.
    fn update<'b, 'c, C>(self, args: widget::UpdateArgs<'b, 'c, Self, C>) -> Option<State>
        where C: CharacterCache,
    {
        let widget::UpdateArgs { prev_state, xy, dim, input, theme, .. } = args;
        let widget::State { ref state, .. } = *prev_state;
        let State { interaction, time_last_clicked, ref maybe_title_bar } = *state;
        let maybe_mouse = input.maybe_mouse.map(|mouse| mouse.relative_to(xy));
        let title_bar_font_size = self.style.title_bar_font_size(theme);

        // Calculate the height and y coord of the title bar.
        let (title_bar_h, title_bar_y) = if self.show_title_bar {
            title_bar_h_y(dim, title_bar_font_size as f64)
        } else {
            (0.0, 0.0)
        };

        // If there is new mouse state, check for a new interaction.
        let new_interaction = if let Some(mouse) = maybe_mouse {
            let is_over_elem = is_over(mouse.relative_to(xy).xy, dim, title_bar_y, title_bar_h);
            get_new_interaction(is_over_elem, interaction, mouse)
        } else {
            interaction
        };

        // If the canvas was clicked, dragged or released, update the time_last_clicked.
        let new_time_last_clicked = match (interaction, new_interaction) {
            (Interaction::Highlighted(_), Interaction::Clicked(_, _)) |
            (Interaction::Clicked(_, _), Interaction::Highlighted(_)) |
            (Interaction::Clicked(_, _), Interaction::Clicked(_, _))  => precise_time_ns(),
            _ => time_last_clicked,
        };

        // A function for constructing a new state.
        let new_state = || State {
            interaction: new_interaction,
            time_last_clicked: new_time_last_clicked,
            maybe_title_bar: if self.show_title_bar {
                Some(TitleBar {
                    maybe_label: self.maybe_title_bar_label.as_ref()
                        .map(|label| (label.to_string(), title_bar_font_size)),
                    h: title_bar_h,
                    y: title_bar_y,
                })
            } else {
                None
            },
        };

        // Check whether or not the state has changed since the previous update.
        let state_has_changed = interaction != new_interaction
            || time_last_clicked != new_time_last_clicked
            || match *maybe_title_bar {
                None => self.show_title_bar,
                Some(ref title_bar) => {
                    title_bar.y != title_bar_y
                    || title_bar.h != title_bar_h
                    || match title_bar.maybe_label {
                        None => false,
                        Some((ref label, font_size)) => {
                            Some(&label[..]) != self.maybe_title_bar_label
                            || font_size != title_bar_font_size
                        },
                    }
                },
            };

        // Construct the new state if there was a change.
        if state_has_changed { Some(new_state()) } else { None }
    }

    /// Draw the canvas.
    fn draw<'b, C>(args: widget::DrawArgs<'b, Self, C>) -> Element
        where C: CharacterCache
    {
        use elmesque::form::{collage, rect, text};

        let widget::DrawArgs { state, style, theme, glyph_cache } = args;
        let widget::State { ref state, dim, xy, .. } = *state;

        let frame = style.frame(theme);
        let inner_dim = ::vecmath::vec2_sub(dim, [frame * 2.0; 2]);
        let color = style.color(theme);
        let frame_color = style.frame_color(theme);
        let frame_form = rect(dim[0], dim[1]).filled(frame_color.alpha(0.7));
        let rect_form = rect(inner_dim[0], inner_dim[1]).filled(color.alpha(0.7));

        // Check whether or not to draw the title bar.
        let maybe_title_bar_form = if let Some(ref title_bar) = state.maybe_title_bar {
            let inner_dim = ::vecmath::vec2_sub([dim[0], title_bar.h], [frame * 2.0; 2]);
            let title_bar_frame_form = rect(dim[0], title_bar.h).filled(frame_color);
            let title_bar_rect_form = rect(inner_dim[0], inner_dim[1]).filled(color);
            // Check whether or not to draw the title bar's label.
            let maybe_label_form = title_bar.maybe_label.as_ref().map(|&(ref label, font_size)| {
                use elmesque::text::Text;
                let label_color = style.title_bar_label_color(theme);
                let align = style.title_bar_label_align(theme);
                let label_width = glyph_cache.width(font_size, label);
                let label_x = align.to(inner_dim[0], label_width) + TITLE_BAR_LABEL_PADDING;
                text(Text::from_string(label.clone())
                        .color(label_color)
                        .height(font_size as f64)).shift_x(label_x)
            });
            Some(Some(title_bar_frame_form).into_iter()
                .chain(Some(title_bar_rect_form).into_iter())
                .chain(maybe_label_form)
                .map(move |form| form.shift_y(title_bar.y)))
        } else {
            None
        };

        // Chain the Canvas' Forms together.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(rect_form).into_iter())
            .chain(maybe_title_bar_form.into_iter().flat_map(|it| it))
            .map(|form| form.shift(xy[0], xy[1]));

        // Construct the renderable element.
        collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
    }

}


/// Calculate the height and y position of the Title Bar.
fn title_bar_h_y(dim: Dimensions, font_size: f64) -> (Scalar, Scalar) {
    let h = font_size as f64 + TITLE_BAR_LABEL_PADDING * 2.0;
    let y = position::align_top_of(dim[1], h);
    (h, y)
}


/// Is the mouse over the canvas, if so which Elem.
fn is_over(mouse_xy: Point,
           dim: Dimensions,
           title_bar_y: f64,
           title_bar_h: f64) -> Option<Elem>
{
    use utils::is_over_rect;
    if is_over_rect([0.0, 0.0], mouse_xy, dim) {
        if is_over_rect([0.0, title_bar_y], mouse_xy, [dim[0], title_bar_h]) {
            Some(Elem::TitleBar)
        } else {
            Some(Elem::WidgetArea)
        }
    } else {
        None
    }
}


/// Determine the new interaction given the mouse state and previous interaction.
fn get_new_interaction(is_over_elem: Option<Elem>, prev: Interaction, mouse: Mouse) -> Interaction {
    use mouse::ButtonState::{Down, Up};
    use self::Interaction::{Normal, Highlighted, Clicked};
    match (is_over_elem, prev, mouse.left) {
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

    /// Get the color for the Floating's Element.
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

