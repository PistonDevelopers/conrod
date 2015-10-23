
use color::{Color, Colorable};
use elmesque::Element;
use frame::Frameable;
use graphics::character::CharacterCache;
use graphics::math::Scalar;
use label::{FontSize, Labelable};
use mouse::Mouse;
use num::{Float, NumCast, ToPrimitive};
use theme::Theme;
use ui::GlyphCache;
use utils::{clamp, percentage, value_from_perc};
use widget::{self, Widget};


/// Linear value selection. If the slider's width is greater than it's height, it will
/// automatically become a horizontal slider, otherwise it will be a vertical slider. Its reaction
/// is triggered if the value is updated or if the mouse button is released while the cursor is
/// above the rectangle.
pub struct Slider<'a, T, F> {
    common: widget::CommonBuilder,
    value: T,
    min: T,
    max: T,
    skew: f32,
    maybe_react: Option<F>,
    maybe_label: Option<&'a str>,
    style: Style,
    enabled: bool,
}

/// Styling for the Slider, necessary for constructing its renderable Element.
#[allow(missing_docs, missing_copy_implementations)]
#[derive(Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    pub maybe_color: Option<Color>,
    pub maybe_frame: Option<Scalar>,
    pub maybe_frame_color: Option<Color>,
    pub maybe_label_color: Option<Color>,
    pub maybe_label_font_size: Option<u32>,
}

/// Represents the state of the Slider widget.
#[derive(Clone, Debug, PartialEq)]
pub struct State<T> {
    value: T,
    min: T,
    max: T,
    skew: f32,
    maybe_label: Option<String>,
    interaction: Interaction,
}

/// The ways in which the Slider can be interacted with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Interaction {
    Normal,
    Highlighted,
    Clicked,
}


impl<T> State<T> {
    /// Return the color associated with the state.
    fn color(&self, color: Color) -> Color {
        match self.interaction {
            Interaction::Normal => color,
            Interaction::Highlighted => color.highlighted(),
            Interaction::Clicked => color.clicked(),
        }
    }
}

/// Check the current state of the slider.
fn get_new_interaction(is_over: bool, prev: Interaction, mouse: Mouse) -> Interaction {
    use mouse::ButtonPosition::{Down, Up};
    use self::Interaction::{Normal, Highlighted, Clicked};
    match (is_over, prev, mouse.left.position) {
        (true,  Normal,  Down) => Normal,
        (true,  _,       Down) => Clicked,
        (true,  _,       Up)   => Highlighted,
        (false, Clicked, Down) => Clicked,
        _ => Normal,
    }
}

impl<'a, T, F> Slider<'a, T, F> {

    /// Construct a new Slider widget.
    pub fn new(value: T, min: T, max: T) -> Slider<'a, T, F> {
        Slider {
            common: widget::CommonBuilder::new(),
            value: value,
            min: min,
            max: max,
            skew: 1.0,
            maybe_react: None,
            maybe_label: None,
            style: Style::new(),
            enabled: true,
        }
    }

    /// Set the amount in which the slider's display should be skewed.
    /// Higher skew amounts (above 1.0) will weight lower values.
    /// Lower skew amounts (below 1.0) will weight heigher values.
    /// All skew amounts should be greater than 0.0.
    pub fn skew(mut self, skew: f32) -> Slider<'a, T, F> {
        self.skew = skew;
        self
    }

    /// Set the reaction for the Slider. It will be triggered if the value is updated or if the
    /// mouse button is released while the cursor is above the rectangle.
    pub fn react(mut self, reaction: F) -> Slider<'a, T, F> {
        self.maybe_react = Some(reaction);
        self
    }

    /// If true, will allow user inputs.  If false, will disallow user inputs.
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
    }

}

impl<'a, T, F> Widget for Slider<'a, T, F> where
    F: FnMut(T),
    T: ::std::any::Any + ::std::fmt::Debug + Float + NumCast + ToPrimitive,
{
    type State = State<T>;
    type Style = Style;
    fn common(&self) -> &widget::CommonBuilder { &self.common }
    fn common_mut(&mut self) -> &mut widget::CommonBuilder { &mut self.common }
    fn unique_kind(&self) -> &'static str { "Slider" }
    fn init_state(&self) -> State<T> {
        State {
            value: self.value,
            min: self.min,
            max: self.max,
            skew: self.skew,
            maybe_label: None,
            interaction: Interaction::Normal,
        }
    }
    fn style(&self) -> Style { self.style.clone() }

    fn default_width<C: CharacterCache>(&self, theme: &Theme, _: &GlyphCache<C>) -> Scalar {
        const DEFAULT_WIDTH: Scalar = 192.0;
        theme.maybe_slider.as_ref().map(|default| {
            default.common.maybe_width.unwrap_or(DEFAULT_WIDTH)
        }).unwrap_or(DEFAULT_WIDTH)
    }

    fn default_height(&self, theme: &Theme) -> Scalar {
        const DEFAULT_HEIGHT: Scalar = 48.0;
        theme.maybe_slider.as_ref().map(|default| {
            default.common.maybe_height.unwrap_or(DEFAULT_HEIGHT)
        }).unwrap_or(DEFAULT_HEIGHT)
    }

    /// Update the state of the Slider.
    fn update<C: CharacterCache>(mut self, args: widget::UpdateArgs<Self, C>) {
        use utils::map_range;

        let widget::UpdateArgs { state, rect, style, mut ui, .. } = args;
        let (xy, dim) = rect.xy_dim();
        let maybe_mouse = ui.input().maybe_mouse.map(|mouse| mouse.relative_to(xy));
        let new_interaction = match (self.enabled, maybe_mouse) {
            (false, _) | (true, None) => Interaction::Normal,
            (true, Some(mouse)) => {
                let is_over = ::position::is_over_rect([0.0, 0.0], dim, mouse.xy);
                get_new_interaction(is_over, state.view().interaction, mouse)
            },
        };

        match (state.view().interaction, new_interaction) {
            (Interaction::Highlighted, Interaction::Clicked) => { ui.capture_mouse(); },
            (Interaction::Clicked, Interaction::Highlighted) |
            (Interaction::Clicked, Interaction::Normal)      => { ui.uncapture_mouse(); },
            _ => (),
        }

        let new_value = if let Some(mouse) = maybe_mouse {
            let Slider { value, min, max, skew, .. } = self;
            let frame = style.frame(ui.theme());
            let frame_2 = frame * 2.0;
            let (inner_w, inner_h) = (dim[0] - frame_2, dim[1] - frame_2);
            let (half_inner_w, half_inner_h) = (inner_w / 2.0, inner_h / 2.0);
            let is_horizontal = dim[0] > dim[1];

            if is_horizontal {
                // Horizontal.
                let w_perc = match (state.view().interaction, new_interaction) {
                    (Interaction::Highlighted, Interaction::Clicked) |
                    (Interaction::Clicked, Interaction::Clicked) => {
                        let w = map_range(mouse.xy[0], -half_inner_w, half_inner_w, 0.0, inner_w);
                        let perc = clamp(w, 0.0, inner_w) / inner_w;
                        (perc).powf(skew as f64)
                    },
                    _ => {
                        let value_percentage = percentage(value, min, max);
                        let w = clamp(value_percentage as f64 * inner_w, 0.0, inner_w);
                        (w / inner_w)
                    },
                };
                value_from_perc(w_perc as f32, min, max)
            } else {
                // Vertical.
                let h_perc = match (state.view().interaction, new_interaction) {
                    (Interaction::Highlighted, Interaction::Clicked) |
                    (Interaction::Clicked, Interaction::Clicked) => {
                        let h = map_range(mouse.xy[1], -half_inner_h, half_inner_h, 0.0, inner_h);
                        let perc = clamp(h, 0.0, inner_h) / inner_h;
                        (perc).powf(skew as f64)
                    },
                    _ => {
                        let value_percentage = percentage(value, min, max);
                        let h = clamp(value_percentage as f64 * inner_h, 0.0, inner_h);
                        (h / inner_h)
                    },
                };
                value_from_perc(h_perc as f32, min, max)
            }
        } else {
            self.value
        };

        // React.
        match self.maybe_react {
            Some(ref mut react) => {
                if self.value != new_value || match (state.view().interaction, new_interaction) {
                    (Interaction::Highlighted, Interaction::Clicked) |
                    (Interaction::Clicked, Interaction::Highlighted) => true,
                    _ => false,
                } { react(new_value) }
            }, None => (),
        }

        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        if state.view().value != new_value {
            state.update(|state| state.value = self.value);
        }

        if state.view().min != self.min {
            state.update(|state| state.min = self.min);
        }

        if state.view().max != self.max {
            state.update(|state| state.max = self.max);
        }

        if state.view().skew != self.skew {
            state.update(|state| state.skew = self.skew);
        }

        if state.view().maybe_label.as_ref().map(|label| &label[..]) != self.maybe_label {
            state.update(|state| {
                state.maybe_label = self.maybe_label.as_ref().map(|label| label.to_string());
            })
        }
    }

    /// Construct an Element from the given Slider State.
    fn draw<C>(args: widget::DrawArgs<Self, C>) -> Element
        where C: CharacterCache,
    {
        use elmesque::form::{self, collage, text};

        let widget::DrawArgs { rect, state, style, theme, glyph_cache, .. } = args;
        let (xy, dim) = rect.xy_dim();
        let frame = style.frame(theme);
        let (inner_w, inner_h) = (dim[0] - frame * 2.0, dim[1] - frame * 2.0);
        let frame_color = state.color(style.frame_color(theme));
        let color = state.color(style.color(theme));

        let new_value = NumCast::from(state.value).unwrap();
        let is_horizontal = dim[0] > dim[1];
        let (pad_rel_xy, pad_dim) = if is_horizontal {
            // Horizontal.
            let value_percentage = percentage(new_value, state.min, state.max).powf(1.0/state.skew);
            let w = clamp(value_percentage as f64 * inner_w, 0.0, inner_w);
            let rel_xy = [-(inner_w - w) / 2.0, 0.0];
            (rel_xy, [w, inner_h])
        } else {
            // Vertical.
            let value_percentage = percentage(new_value, state.min, state.max).powf(1.0/state.skew);
            let h = clamp(value_percentage as f64 * inner_h, 0.0, inner_h);
            let rel_xy = [0.0, -(inner_h - h) / 2.0];
            (rel_xy, [inner_w, h])
        };

        // Rectangle frame / backdrop Form.
        let frame_form = form::rect(dim[0], dim[1])
            .filled(frame_color);
        // Slider rectangle Form.
        let pad_form = form::rect(pad_dim[0], pad_dim[1])
            .filled(color)
            .shift(pad_rel_xy[0], pad_rel_xy[1]);

        // Label Form.
        let maybe_label_form = state.maybe_label.as_ref().map(|label_text| {
            use elmesque::text::Text;
            use position;
            const TEXT_PADDING: f64 = 10.0;
            let label_color = style.label_color(theme);
            let size = style.label_font_size(theme);
            let label_w = glyph_cache.width(size, &label_text);
            let is_horizontal = dim[0] > dim[1];
            let l_pos = if is_horizontal {
                let x = position::align_left_of(dim[0], label_w) + TEXT_PADDING;
                [x, 0.0]
            } else {
                let y = position::align_bottom_of(dim[1], size as f64) + TEXT_PADDING;
                [0.0, y]
            };
            text(Text::from_string(label_text.clone()).color(label_color).height(size as f64))
                .shift(l_pos[0].floor(), l_pos[1].floor())
                .shift(xy[0].floor(), xy[1].floor())
        });

        // Chain the Forms and shift them into position.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(pad_form))
            .map(|form| form.shift(xy[0], xy[1]))
            .chain(maybe_label_form);

        // Collect the Forms into a renderable Element.
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
        }
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.maybe_slider.as_ref().map(|default| {
            default.style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.maybe_slider.as_ref().map(|default| {
            default.style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.maybe_slider.as_ref().map(|default| {
            default.style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label Color for an Element.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.maybe_slider.as_ref().map(|default| {
            default.style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn label_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.maybe_slider.as_ref().map(|default| {
            default.style.maybe_label_font_size.unwrap_or(theme.font_size_medium)
        })).unwrap_or(theme.font_size_medium)
    }

}


impl<'a, T, F> Colorable for Slider<'a, T, F> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

impl<'a, T, F> Frameable for Slider<'a, T, F> {
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, T, F> Labelable<'a> for Slider<'a, T, F> {
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

