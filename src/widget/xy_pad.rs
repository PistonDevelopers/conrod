
use color::{Color, Colorable};
use elmesque::Element;
use frame::Frameable;
use graphics::character::CharacterCache;
use graphics::math::Scalar;
use label::{FontSize, Labelable};
use mouse::Mouse;
use num::Float;
use position::{self, Corner};
use theme::Theme;
use ui::GlyphCache;
use utils::{clamp, map_range, val_to_string};
use vecmath::vec2_sub;
use widget::{self, Widget};


/// Used for displaying and controlling a 2D point on a cartesian plane within a given range.
/// Its reaction is triggered when the value is updated or if the mouse button is released while
/// the cursor is above the rectangle.
pub struct XYPad<'a, X, Y, F> {
    common: widget::CommonBuilder,
    x: X, min_x: X, max_x: X,
    y: Y, min_y: Y, max_y: Y,
    maybe_label: Option<&'a str>,
    maybe_react: Option<F>,
    style: Style,
    enabled: bool,
}

/// Styling for the XYPad, necessary for constructing its renderable Element.
#[allow(missing_docs, missing_copy_implementations)]
#[derive(Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    pub maybe_color: Option<Color>,
    pub maybe_frame: Option<Scalar>,
    pub maybe_frame_color: Option<Color>,
    pub maybe_label_color: Option<Color>,
    pub maybe_label_font_size: Option<FontSize>,
    pub maybe_value_font_size: Option<FontSize>,
    pub maybe_line_width: Option<f64>,
}

/// The state of the XYPad.
#[derive(Clone, Debug, PartialEq)]
pub struct State<X, Y> {
    x: X, min_x: X, max_x: X,
    y: Y, min_y: Y, max_y: Y,
    maybe_label: Option<String>,
    interaction: Interaction,
}

/// The interaction state of the XYPad.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Interaction {
    Normal,
    Highlighted,
    Clicked,
}


impl<X, Y> State<X, Y> {
    /// The color associated with the current state.
    fn color(&self, color: Color) -> Color {
        match self.interaction {
            Interaction::Normal => color,
            Interaction::Highlighted => color.highlighted(),
            Interaction::Clicked => color.clicked(),
        }
    }
}


/// Check the current state of the button.
fn get_new_interaction(is_over: bool,
                       prev: Interaction,
                       mouse: Mouse) -> Interaction {
    use mouse::ButtonPosition::{Down, Up};
    use self::Interaction::{Normal, Highlighted, Clicked};
    match (is_over, prev, mouse.left.position) {
        (true,  Normal,  Down) => Normal,
        (true,  _,       Down) => Clicked,
        (true,  _,       Up)   => Highlighted,
        (false, Clicked, Down) => Clicked,
        _                      => Normal,
    }
}


impl<'a, X, Y, F> XYPad<'a, X, Y, F> {

    /// Construct a new XYPad widget.
    pub fn new(x_val: X, min_x: X, max_x: X, y_val: Y, min_y: Y, max_y: Y) -> XYPad<'a, X, Y, F> {
        XYPad {
            common: widget::CommonBuilder::new(),
            x: x_val, min_x: min_x, max_x: max_x,
            y: y_val, min_y: min_y, max_y: max_y,
            maybe_react: None,
            maybe_label: None,
            style: Style::new(),
            enabled: true,
        }
    }

    /// Set the width of the XYPad's crosshair lines.
    #[inline]
    pub fn line_width(mut self, width: f64) -> XYPad<'a, X, Y, F> {
        self.style.maybe_line_width = Some(width);
        self
    }

    /// Set the font size for the displayed crosshair value.
    #[inline]
    pub fn value_font_size(mut self, size: FontSize) -> XYPad<'a, X, Y, F> {
        self.style.maybe_value_font_size = Some(size);
        self
    }

    /// Set the reaction for the XYPad. It will be triggered when the value is updated or if the
    /// mouse button is released while the cursor is above the rectangle.
    pub fn react(mut self, reaction: F) -> Self {
        self.maybe_react = Some(reaction);
        self
    }

    /// If true, will allow user inputs.  If false, will disallow user inputs.
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
    }

}

impl<'a, X, Y, F> Widget for XYPad<'a, X, Y, F>
    where
        X: Float + ToString + ::std::fmt::Debug + ::std::any::Any,
        Y: Float + ToString + ::std::fmt::Debug + ::std::any::Any,
        F: FnMut(X, Y),
{
    type State = State<X, Y>;
    type Style = Style;
    fn common(&self) -> &widget::CommonBuilder { &self.common }
    fn common_mut(&mut self) -> &mut widget::CommonBuilder { &mut self.common }
    fn unique_kind(&self) -> &'static str { "XYPad" }
    fn init_state(&self) -> State<X, Y> {
        State {
            interaction: Interaction::Normal,
            x: self.x, min_x: self.min_x, max_x: self.max_x,
            y: self.y, min_y: self.min_y, max_y: self.max_y,
            maybe_label: None,
        }
    }
    fn style(&self) -> Style { self.style.clone() }

    fn default_width<C: CharacterCache>(&self, theme: &Theme, _: &GlyphCache<C>) -> Scalar {
        const DEFAULT_WIDTH: Scalar = 128.0;
        theme.maybe_xy_pad.as_ref().map(|default| {
            default.common.maybe_width.unwrap_or(DEFAULT_WIDTH)
        }).unwrap_or(DEFAULT_WIDTH)
    }

    fn default_height(&self, theme: &Theme) -> Scalar {
        const DEFAULT_HEIGHT: Scalar = 128.0;
        theme.maybe_xy_pad.as_ref().map(|default| {
            default.common.maybe_height.unwrap_or(DEFAULT_HEIGHT)
        }).unwrap_or(DEFAULT_HEIGHT)
    }

    /// Update the XYPad's cached state.
    fn update<C: CharacterCache>(mut self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { state, rect, style, mut ui, .. } = args;

        let (xy, dim) = rect.xy_dim();
        let maybe_mouse = ui.input().maybe_mouse.map(|mouse| mouse.relative_to(xy));
        let frame = style.frame(ui.theme());
        let pad_dim = vec2_sub(dim, [frame * 2.0; 2]);
        let new_interaction = match (self.enabled, maybe_mouse) {
            (false, _) | (true, None) => Interaction::Normal,
            (true, Some(mouse)) => {
                let is_over_pad = position::is_over_rect([0.0, 0.0], pad_dim, mouse.xy);
                get_new_interaction(is_over_pad, state.view().interaction, mouse)
            },
        };

        // Capture the mouse if clicked, uncapture if released.
        match (state.view().interaction, new_interaction) {
            (Interaction::Highlighted, Interaction::Clicked) => { ui.capture_mouse(); },
            (Interaction::Clicked, Interaction::Highlighted) |
            (Interaction::Clicked, Interaction::Normal)      => { ui.uncapture_mouse(); },
            _ => (),
        }

        // Determine new values from the mouse position over the pad.
        let (new_x, new_y) = match (maybe_mouse, new_interaction) {
            (None, _) | (_, Interaction::Normal) | (_, Interaction::Highlighted) => (self.x, self.y),
            (Some(mouse), Interaction::Clicked) => {
                let half_pad_w = pad_dim[0] / 2.0;
                let half_pad_h = pad_dim[1] / 2.0;
                let temp_x = clamp(mouse.xy[0], -half_pad_w, half_pad_w);
                let temp_y = clamp(mouse.xy[1], -half_pad_h, half_pad_h);
                (map_range(temp_x, -half_pad_w, half_pad_w, self.min_x, self.max_x),
                 map_range(temp_y, -half_pad_h, half_pad_h, self.min_y, self.max_y))
            }
        };

        // React if value is changed or the pad is clicked/released.
        if let Some(ref mut react) = self.maybe_react {
            if self.x != new_x || self.y != new_y { react(new_x, new_y) }
            else {
                match (state.view().interaction, new_interaction) {
                    (Interaction::Highlighted, Interaction::Clicked) |
                    (Interaction::Clicked, Interaction::Highlighted) => react(new_x, new_y),
                    _ => (),
                }
            }
        }

        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        let value_or_bounds_have_changed = {
            let v = state.view();
            v.x != self.x || v.y != self.y
                || v.min_x != self.min_x || v.max_x != self.max_x
                || v.min_y != self.min_y || v.max_y != self.max_y
        };

        if value_or_bounds_have_changed {
            state.update(|state| {
                state.x = self.x;
                state.y = self.y;
                state.min_x = self.min_x;
                state.max_x = self.max_x;
                state.min_y = self.min_y;
                state.max_y = self.max_y;
            })
        }

        if state.view().maybe_label.as_ref().map(|label| &label[..]) != self.maybe_label {
            state.update(|state| {
                state.maybe_label = self.maybe_label.as_ref().map(|label| label.to_string());
            });
        }
    }

    /// Construct an Element from the given XYPad State.
    fn draw<C: CharacterCache>(args: widget::DrawArgs<Self, C>) -> Element {
        use elmesque::form::{self, collage, line, solid, text};
        use elmesque::text::Text;

        let widget::DrawArgs { rect, state, style, theme, glyph_cache, .. } = args;
        let (xy, dim) = rect.xy_dim();
        let frame = style.frame(theme);
        let pad_dim = vec2_sub(dim, [frame * 2.0; 2]);
        let (half_pad_w, half_pad_h) = (pad_dim[0] / 2.0, pad_dim[1] / 2.0);

        // Construct the frame and inner rectangle Forms.
        let color = state.color(style.color(theme));
        let frame_color = style.frame_color(theme);
        let frame_form = form::rect(dim[0], dim[1]).filled(frame_color);
        let pressable_form = form::rect(pad_dim[0], pad_dim[1]).filled(color);

        // Construct the label Form.
        let maybe_label_form = state.maybe_label.as_ref().map(|l_text| {
            let l_color = style.label_color(theme);
            let l_size = style.label_font_size(theme) as f64;
            text(Text::from_string(l_text.clone()).color(l_color).height(l_size))
        });

        // Construct the crosshair line Forms.
        let ch_x = map_range(state.x, state.min_x, state.max_x, -half_pad_w, half_pad_w).floor();
        let ch_y = map_range(state.y, state.min_y, state.max_y, -half_pad_h, half_pad_h).floor();
        let line_width = style.line_width(theme);
        let line_style = solid(color.plain_contrast()).width(line_width);
        let vert_form = line(line_style.clone(), 0.0, -half_pad_h, 0.0, half_pad_h).shift_x(ch_x);
        let hori_form = line(line_style, -half_pad_w, 0.0, half_pad_w, 0.0).shift_y(ch_y);

        // Construct the value string Form.
        let x_string = val_to_string(state.x, state.max_x, state.max_x - state.min_x, dim[0] as usize);
        let y_string = val_to_string(state.y, state.max_y, state.max_y - state.min_y, dim[1] as usize);
        let value_string = format!("{}, {}", x_string, y_string);
        let value_text_form = {
            const PAD: f64 = 5.0; // Slight padding between the crosshair and the text.
            let value_font_size = style.value_font_size(theme);
            let w = glyph_cache.width(value_font_size, &value_string);
            let h = value_font_size as f64;
            let x_shift = w / 2.0 + PAD;
            let y_shift = h / 2.0 + PAD;
            let (value_text_x, value_text_y) = match position::corner([ch_x, ch_y], pad_dim) {
                Corner::TopLeft => (x_shift, -y_shift),
                Corner::TopRight => (-x_shift, -y_shift),
                Corner::BottomLeft => (x_shift, y_shift),
                Corner::BottomRight => (-x_shift, y_shift),
            };
            text(Text::from_string(value_string).color(color.plain_contrast()).height(h))
                .shift(ch_x, ch_y)
                .shift(value_text_x.floor(), value_text_y.floor())
        };

        // Chain the Forms and shift them into position.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(pressable_form).into_iter())
            .chain(maybe_label_form.into_iter())
            .chain(Some(vert_form).into_iter())
            .chain(Some(hori_form).into_iter())
            .chain(Some(value_text_form).into_iter())
            .map(|form| form.shift(xy[0].round(), xy[1].round()));

        // Turn the form into a renderable Element.
        collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
    }

}


impl Style {

    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
            maybe_value_font_size: None,
            maybe_line_width: None,
        }
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.maybe_xy_pad.as_ref().map(|default| {
            default.style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.maybe_xy_pad.as_ref().map(|default| {
            default.style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.maybe_xy_pad.as_ref().map(|default| {
            default.style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label Color for an Element.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.maybe_xy_pad.as_ref().map(|default| {
            default.style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn label_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.maybe_xy_pad.as_ref().map(|default| {
            default.style.maybe_label_font_size.unwrap_or(theme.font_size_medium)
        })).unwrap_or(theme.font_size_medium)
    }

    /// Get the value font size for an Element.
    pub fn value_font_size(&self, theme: &Theme) -> FontSize {
        const DEFAULT_VALUE_FONT_SIZE: u32 = 14;
        self.maybe_value_font_size.or(theme.maybe_xy_pad.as_ref().map(|default| {
            default.style.maybe_value_font_size.unwrap_or(DEFAULT_VALUE_FONT_SIZE)
        })).unwrap_or(DEFAULT_VALUE_FONT_SIZE)
    }

    /// Get the point radius size for an Element.
    pub fn line_width(&self, theme: &Theme) -> f64 {
        const DEFAULT_LINE_WIDTH: f64 = 2.0;
        self.maybe_line_width.or(theme.maybe_xy_pad.as_ref().map(|default| {
            default.style.maybe_line_width.unwrap_or(DEFAULT_LINE_WIDTH)
        })).unwrap_or(DEFAULT_LINE_WIDTH)
    }

}

impl<'a, X, Y, F> Colorable for XYPad<'a, X, Y, F> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

impl<'a, X, Y, F> Frameable for XYPad<'a, X, Y, F> {
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, X, Y, F> Labelable<'a> for XYPad<'a, X, Y, F>
{
    fn label(mut self, text: &'a str) -> Self {
        self.maybe_label = Some(text);
        self
    }

    fn label_color(mut self, color: Color) -> Self {
        self.style.maybe_label_color = Some(color);
        self
    }

    fn label_font_size(mut self, size: FontSize) -> Self {
        self.style.maybe_label_font_size = Some(size);
        self
    }
}

