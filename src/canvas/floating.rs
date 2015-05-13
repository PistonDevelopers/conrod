
use clock_ticks::precise_time_ns;
use color::Color;
use graphics::character::CharacterCache;
use graphics::math::Scalar;
use label::{self, FontSize};
use mouse::Mouse;
use position::{self, Dimensions, HorizontalAlign, Place, Point, Position, VerticalAlign};
use theme::Theme;
use ui::{Ui, UiId};

use super::{CanvasId, Kind};
//use super::split::Split;

/// The current state of a Floating.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    interaction: Interaction,
    xy: Point,
    dim: Dimensions,
    pub time_last_clicked: u64,
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

/// A type of Canvas for flexibly designing and guiding widget layout as splits of a window.
pub struct Floating<'a> {
    init_dim: Dimensions,
    init_pos: Position,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    //maybe_splits: Option<(Direction, &'a [Split<'a>])>,
    maybe_title_bar_label: Option<&'a str>,
    show_title_bar: bool,
    style: Style,
//     maybe_adjustable: Option<Adjustable>,
}

/// Describes the style of a Canvas Floating.
#[derive(Clone, Debug, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Style {
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_title_bar_font_size: Option<FontSize>,
    maybe_title_bar_label_align: Option<HorizontalAlign>,
    maybe_title_bar_label_color: Option<Color>,
    padding: Padding,
}

// /// Adjustable dimensions within these bounds.
// #[derive(Clone, Copy, Debug, PartialEq)]
// pub struct Adjustable {
//     width: Bounds,
//     height: Bounds,
// }
// 
// /// The minimum and maximum for a dimension of a Floating.
// #[derive(Clone, Copy, Debug, PartialEq)]
// pub struct Bounds {
//     pub min: f64,
//     pub max: f64,
// }

/// The distance between the edge of a widget and the inner edge of a Canvas' frame.
#[derive(Clone, Debug, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Padding {
    maybe_top: Option<f64>,
    maybe_bottom: Option<f64>,
    maybe_left: Option<f64>,
    maybe_right: Option<f64>,
}


impl<'a> Floating<'a> {

    /// Construct a default Floating canvas.
    pub fn new() -> Floating<'a> {
        Floating {
            init_dim: [160.0, 80.0],
            init_pos: Position::Place(Place::Middle, None),
            maybe_h_align: None,
            maybe_v_align: None,
            //maybe_splits: None,
            maybe_title_bar_label: None,
            show_title_bar: true,
            style: Style::new(),
        }
    }

    /// Show or hide the title bar.
    pub fn show_title_bar(mut self, show: bool) -> Floating<'a> {
        self.show_title_bar = show;
        self
    }

    /// Set the padding from the left edge.
    pub fn pad_left(mut self, pad: Scalar) -> Floating<'a> {
        self.style.padding.maybe_left = Some(pad);
        self
    }

    /// Set the padding from the right edge.
    pub fn pad_right(mut self, pad: Scalar) -> Floating<'a> {
        self.style.padding.maybe_right = Some(pad);
        self
    }

    /// Set the padding from the top edge.
    pub fn pad_top(mut self, pad: Scalar) -> Floating<'a> {
        self.style.padding.maybe_top = Some(pad);
        self
    }

    /// Set the padding from the bottom edge.
    pub fn pad_bottom(mut self, pad: Scalar) -> Floating<'a> {
        self.style.padding.maybe_bottom = Some(pad);
        self
    }

    /// Set the padding for all edges.
    pub fn pad(self, pad: Scalar) -> Floating<'a> {
        self.pad_left(pad).pad_right(pad).pad_top(pad).pad_bottom(pad)
    }

    /// Register the Canvas within the Ui.
    pub fn set<C>(self, id: CanvasId, ui: &mut Ui<C>)
        where
            C: CharacterCache
    {
        use elmesque::form::{collage, rect, text};

        let Floating {
            show_title_bar,
            //ref maybe_splits,
            ref style,
            ref maybe_title_bar_label,
            ..
        } = self;

        let State { interaction, xy, dim, time_last_clicked } = match ui.get_canvas_state(id) {
            Some(Kind::Floating(state)) => state,
            _ => {
                let init_dim = self.init_dim;
                let h_align = self.maybe_h_align.unwrap_or(ui.theme.align.horizontal);
                let v_align = self.maybe_v_align.unwrap_or(ui.theme.align.vertical);
                let init_xy = ui.get_xy(self.init_pos, init_dim, h_align, v_align);
                State {
                    interaction: Interaction::Normal, 
                    xy: init_xy,
                    dim: init_dim,
                    time_last_clicked: precise_time_ns(),
                }
            },
        };

        let maybe_mouse = ui.get_mouse_state(UiId::Canvas(id), Some(id));
        let pad = style.padding(&ui.theme);
        let title_bar_font_size = style.title_bar_font_size(&ui.theme);
        const TITLE_BAR_LABEL_PADDING: f64 = 2.0;
        let (title_bar_h, title_bar_y) = {
            if show_title_bar {
                let h = title_bar_font_size as f64 + TITLE_BAR_LABEL_PADDING * 2.0;
                let y = position::align_top_of(dim[1], h);
                (h, y)
            } else {
                (0.0, 0.0)
            }
        };

        // If there is new mouse state, check for a new interaction.
        let new_interaction = if let Some(mouse) = maybe_mouse {
            let is_over_elem = is_over(mouse.relative_to(xy).xy, dim, title_bar_y, title_bar_h);
            get_new_interaction(is_over_elem, interaction, mouse)
        } else {
            interaction
        };

        // Drag the Canvas if the TitleBar remains clicked.
        let new_xy = match (interaction, new_interaction) {
            (Interaction::Clicked(Elem::TitleBar, a), Interaction::Clicked(Elem::TitleBar, b)) =>
                ::vecmath::vec2_add(xy, ::vecmath::vec2_sub(b, a)),
            _ => xy,
        };

        // Check whether or not we need to capture or uncapture the mouse.
        match (interaction, new_interaction) {
            (Interaction::Highlighted(Elem::TitleBar), Interaction::Clicked(Elem::TitleBar, _)) =>
                ui.mouse_captured_by(UiId::Canvas(id)),
            (Interaction::Clicked(Elem::TitleBar, _), _) =>
                ui.mouse_uncaptured_by(UiId::Canvas(id)),
            _ => (),
        }

        // If the canvas was clicked, dragged or released, update the time_last_clicked.
        let new_time_last_clicked = match (interaction, new_interaction) {
            (Interaction::Highlighted(_), Interaction::Clicked(_, _)) |
            (Interaction::Clicked(_, _), Interaction::Highlighted(_)) |
            (Interaction::Clicked(_, _), Interaction::Clicked(_, _))  => precise_time_ns(),
            _ => time_last_clicked,
        };

        // Draw.
        let frame = style.frame(&ui.theme);
        let inner_dim = ::vecmath::vec2_sub(dim, [frame * 2.0; 2]);
        let color = style.color(&ui.theme);
        let frame_color = style.frame_color(&ui.theme);
        let frame_form = rect(dim[0], dim[1]).filled(frame_color.alpha(0.7));
        let rect_form = rect(inner_dim[0], inner_dim[1]).filled(color.alpha(0.7));
        let maybe_title_bar_form = if show_title_bar {
            let inner_dim = ::vecmath::vec2_sub([dim[0], title_bar_h], [frame * 2.0; 2]);
            let title_bar_frame_form = rect(dim[0], title_bar_h).filled(frame_color);
            let title_bar_rect_form = rect(inner_dim[0], inner_dim[1]).filled(color);
            let maybe_label_form = if let Some(label) = *maybe_title_bar_label {
                use elmesque::text::Text;
                let label_color = style.title_bar_label_color(&ui.theme);
                let align = style.title_bar_label_align(&ui.theme);
                let label_width = label::width(ui, title_bar_font_size, label);
                let label_x = align.to(inner_dim[0], label_width) + TITLE_BAR_LABEL_PADDING;
                Some(text(Text::from_string(label.to_string())
                             .color(label_color)
                             .height(title_bar_font_size as f64)).shift_x(label_x))
            } else {
                None
            };
            Some(Some(title_bar_frame_form).into_iter()
                .chain(Some(title_bar_rect_form).into_iter())
                .chain(maybe_label_form)
                .map(|form| form.shift_y(title_bar_y)))
        } else {
            None
        };

        // Chain the Canvas' Forms together.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(rect_form).into_iter())
            .chain(maybe_title_bar_form.into_iter().flat_map(|it| it))
            .map(|form| form.shift(new_xy[0], new_xy[1]));

        // Construct the renderable element.
        let element = collage(dim[0] as i32, dim[1] as i32, form_chain.collect());

        // Construct the new Canvas state.
        let new_state = State {
            interaction: new_interaction,
            xy: new_xy,
            dim: dim,
            time_last_clicked: new_time_last_clicked,
        };

        // Update the canvas within the `Ui`'s `canvas_cache`.
        ui.update_canvas(id, Kind::Floating(new_state), new_xy, pad, Some(element));
    }

}


/// Is the mouse over the canvas, if so which Elem.
fn is_over(mouse_xy: Point,
           dim: Dimensions,
           title_bar_y: f64,
           title_bar_h: f64) -> Option<Elem> {
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

    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_title_bar_label_color: None,
            maybe_title_bar_font_size: None,
            maybe_title_bar_label_align: None,
            padding: Padding::new()
        }
    }

    /// Get the color for the Floating's Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.maybe_canvas_floating.as_ref().map(|style| {
            style.maybe_color.unwrap_or(theme.background_color)
        })).unwrap_or(theme.background_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.maybe_canvas_floating.as_ref().map(|style| {
            style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.maybe_canvas_floating.as_ref().map(|style| {
            style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the Padding for the Canvas Floating.
    pub fn padding(&self, theme: &Theme) -> position::Padding {
        position::Padding {
            top: self.padding.maybe_top.or(theme.maybe_canvas_floating.as_ref().map(|style| {
                style.padding.maybe_top.unwrap_or(theme.padding.top)
            })).unwrap_or(theme.padding.top),
            bottom: self.padding.maybe_bottom.or(theme.maybe_canvas_floating.as_ref().map(|style| {
                style.padding.maybe_bottom.unwrap_or(theme.padding.bottom)
            })).unwrap_or(theme.padding.bottom),
            left: self.padding.maybe_left.or(theme.maybe_canvas_floating.as_ref().map(|style| {
                style.padding.maybe_left.unwrap_or(theme.padding.left)
            })).unwrap_or(theme.padding.left),
            right: self.padding.maybe_right.or(theme.maybe_canvas_floating.as_ref().map(|style| {
                style.padding.maybe_right.unwrap_or(theme.padding.right)
            })).unwrap_or(theme.padding.right),
        }
    }

    /// Get the font size of the title bar.
    pub fn title_bar_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_title_bar_font_size.or(theme.maybe_canvas_floating.as_ref().map(|style| {
            style.maybe_title_bar_font_size.unwrap_or(theme.font_size_medium)
        })).unwrap_or(theme.font_size_medium)
    }

    /// Get the alignment of the title bar label.
    pub fn title_bar_label_align(&self, theme: &Theme) -> HorizontalAlign {
        const DEFAULT_ALIGN: HorizontalAlign = HorizontalAlign::Middle;
        self.maybe_title_bar_label_align.or(theme.maybe_canvas_floating.as_ref().map(|style| {
            style.maybe_title_bar_label_align.unwrap_or(DEFAULT_ALIGN)
        })).unwrap_or(DEFAULT_ALIGN)
    }

    /// Get the color of the title bar label.
    pub fn title_bar_label_color(&self, theme: &Theme) -> Color {
        self.maybe_title_bar_label_color.or(theme.maybe_canvas_floating.as_ref().map(|style| {
            style.maybe_title_bar_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

}

impl Padding {
    /// Construct a defualt Padding.
    pub fn new() -> Padding {
        Padding {
            maybe_top: None,
            maybe_bottom: None,
            maybe_left: None,
            maybe_right: None,
        }
    }
}

impl<'a> ::color::Colorable for Floating<'a> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

impl<'a> ::frame::Frameable for Floating<'a> {
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
        self
    }
}

impl<'a> ::position::Positionable for Floating<'a> {
    fn position(mut self, pos: Position) -> Self {
        self.init_pos = pos;
        self
    }
    fn get_position(&self) -> Position { self.init_pos }
    #[inline]
    fn horizontal_align(self, h_align: HorizontalAlign) -> Self {
        Floating { maybe_h_align: Some(h_align), ..self }
    }
    #[inline]
    fn vertical_align(self, v_align: VerticalAlign) -> Self {
        Floating { maybe_v_align: Some(v_align), ..self }
    }
}

impl<'a> ::position::Sizeable for Floating<'a> {
    #[inline]
    fn width(self, w: f64) -> Self {
        let h = self.init_dim[1];
        Floating { init_dim: [w, h], ..self }
    }
    #[inline]
    fn height(self, h: f64) -> Self {
        let w = self.init_dim[0];
        Floating { init_dim: [w, h], ..self }
    }
}

impl<'a> ::label::Labelable<'a> for Floating<'a> {
    fn label(mut self, text: &'a str) -> Self {
        self.maybe_title_bar_label = Some(text);
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

