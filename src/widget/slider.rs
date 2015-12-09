
use {
    CharacterCache,
    Color,
    Colorable,
    Dimension,
    FontSize,
    Frameable,
    Labelable,
    IndexSlot,
    KidArea,
    Mouse,
    Padding,
    Positionable,
    Range,
    Rect,
    Rectangle,
    Scalar,
    Sizeable,
    Text,
    Theme,
    Ui,
    Widget,
};
use num::{Float, NumCast, ToPrimitive};
use widget;


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
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    /// The color of the slidable rectangle.
    pub maybe_color: Option<Color>,
    /// The length of the frame around the edges of the slidable rectangle.
    pub maybe_frame: Option<Scalar>,
    /// The color of the Slider's frame.
    pub maybe_frame_color: Option<Color>,
    /// The color of the Slider's label.
    pub maybe_label_color: Option<Color>,
    /// The font-size for the Slider's label.
    pub maybe_label_font_size: Option<u32>,
}

/// Represents the state of the Slider widget.
#[derive(Clone, Debug, PartialEq)]
pub struct State<T> {
    value: T,
    min: T,
    max: T,
    skew: f32,
    interaction: Interaction,
    frame_idx: IndexSlot,
    slider_idx: IndexSlot,
    label_idx: IndexSlot,
}

/// Unique kind for the widget type.
pub const KIND: widget::Kind = "Slider";

/// The ways in which the Slider can be interacted with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Interaction {
    Normal,
    Highlighted,
    Clicked,
}


impl Interaction {
    /// Return the color associated with the state.
    fn color(&self, color: Color) -> Color {
        match *self {
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
    pub fn new(value: T, min: T, max: T) -> Self {
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
    ///
    /// Higher skew amounts (above 1.0) will weight lower values.
    ///
    /// Lower skew amounts (below 1.0) will weight heigher values.
    ///
    /// All skew amounts should be greater than 0.0.
    pub fn skew(mut self, skew: f32) -> Self {
        self.skew = skew;
        self
    }

    /// Set the reaction for the Slider.
    ///
    /// It will be triggered if the value is updated or if the mouse button is released while the
    /// cursor is above the rectangle.
    pub fn react(mut self, reaction: F) -> Self {
        self.maybe_react = Some(reaction);
        self
    }

    /// If true, will allow adjusting the slider.
    ///
    /// If false, will disallow adjusting the slider.
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
    }

}

impl<'a, T, F> Widget for Slider<'a, T, F> where
    F: FnOnce(T),
    T: ::std::any::Any + ::std::fmt::Debug + Float + NumCast + ToPrimitive,
{
    type State = State<T>;
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

    fn init_state(&self) -> State<T> {
        State {
            value: self.value,
            min: self.min,
            max: self.max,
            skew: self.skew,
            interaction: Interaction::Normal,
            frame_idx: IndexSlot::new(),
            slider_idx: IndexSlot::new(),
            label_idx: IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn default_x_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        widget::default_x_dimension(self, ui).unwrap_or(Dimension::Absolute(192.0))
    }

    fn default_y_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        widget::default_y_dimension(self, ui).unwrap_or(Dimension::Absolute(48.0))
    }

    fn kid_area<C: CharacterCache>(&self, args: widget::KidAreaArgs<Self, C>) -> KidArea {
        KidArea {
            rect: args.rect,
            pad: Padding {
                left: 10.0,
                right: 10.0,
                bottom: 10.0,
                top: 10.0,
            },
        }
    }

    /// Update the state of the Slider.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        use self::Interaction::{Clicked, Highlighted, Normal};
        use utils::{clamp, map_range, percentage, value_from_perc};

        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let Slider { value, min, max, skew, enabled, maybe_label, maybe_react, .. } = self;

        let maybe_mouse = ui.input().maybe_mouse;
        let interaction = state.view().interaction;
        let new_interaction = match (enabled, maybe_mouse) {
            (false, _) | (true, None) => Normal,
            (true, Some(mouse)) => {
                let is_over = rect.is_over(mouse.xy);
                get_new_interaction(is_over, interaction, mouse)
            },
        };

        match (interaction, new_interaction) {
            (Highlighted, Clicked) => { ui.capture_mouse(); },
            (Clicked, Highlighted) |
            (Clicked, Normal)      => { ui.uncapture_mouse(); },
            _ => (),
        }

        let is_horizontal = rect.w() > rect.h();
        let frame = style.frame(ui.theme());
        let inner_rect = rect.pad(frame);
        let new_value = if let Some(mouse) = maybe_mouse {
            if is_horizontal {
                // Horizontal.
                let inner_w = inner_rect.w();
                let w_perc = match (interaction, new_interaction) {
                    (Highlighted, Clicked) | (Clicked, Clicked) => {
                        let slider_w = mouse.xy[0] - inner_rect.x.start;
                        let perc = clamp(slider_w, 0.0, inner_w) / inner_w;
                        let skewed_perc = (perc).powf(skew as f64);
                        skewed_perc
                    },
                    _ => {
                        let value_percentage = percentage(value, min, max);
                        let slider_w = clamp(value_percentage as f64 * inner_w, 0.0, inner_w);
                        let perc = slider_w / inner_w;
                        perc
                    },
                };
                value_from_perc(w_perc as f32, min, max)
            } else {
                // Vertical.
                let inner_h = inner_rect.h();
                let h_perc = match (interaction, new_interaction) {
                    (Highlighted, Clicked) | (Clicked, Clicked) => {
                        let slider_h = mouse.xy[1] - inner_rect.y.start;
                        let perc = clamp(slider_h, 0.0, inner_h) / inner_h;
                        let skewed_perc = (perc).powf(skew as f64);
                        skewed_perc
                    },
                    _ => {
                        let value_percentage = percentage(value, min, max);
                        let slider_h = clamp(value_percentage as f64 * inner_h, 0.0, inner_h);
                        let perc = slider_h / inner_h;
                        perc
                    },
                };
                value_from_perc(h_perc as f32, min, max)
            }
        } else {
            value
        };

        // If the value has just changed, or if the slider has been clicked/released, call the
        // reaction function.
        if let Some(react) = maybe_react {
            let should_react = value != new_value
                || (interaction == Highlighted && new_interaction == Clicked)
                || (interaction == Clicked && new_interaction == Highlighted);
            if should_react {
                react(new_value)
            }
        }

        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        if state.view().value != new_value {
            state.update(|state| state.value = value);
        }

        if state.view().min != min {
            state.update(|state| state.min = min);
        }

        if state.view().max != max {
            state.update(|state| state.max = max);
        }

        if state.view().skew != skew {
            state.update(|state| state.skew = skew);
        }

        // The **Rectangle** for the frame.
        let frame_idx = state.view().frame_idx.get(&mut ui);
        let frame_color = new_interaction.color(style.frame_color(ui.theme()));
        Rectangle::fill(rect.dim())
            .middle_of(idx)
            .graphics_for(idx)
            .color(frame_color)
            .set(frame_idx, &mut ui);

        // The **Rectangle** for the adjustable slider.
        let slider_rect = if is_horizontal {
            let left = inner_rect.x.start;
            let right = map_range(new_value, min, max, left, inner_rect.x.end);
            let x = Range::new(left, right);
            let y = inner_rect.y;
            Rect { x: x, y: y }
        } else {
            let bottom = inner_rect.y.start;
            let top = map_range(new_value, min, max, bottom, inner_rect.y.end);
            let x = inner_rect.x;
            let y = Range::new(bottom, top);
            Rect { x: x, y: y }
        };
        let color = new_interaction.color(style.color(ui.theme()));
        let slider_idx = state.view().slider_idx.get(&mut ui);
        let slider_xy_offset = [slider_rect.x() - rect.x(), slider_rect.y() - rect.y()];
        Rectangle::fill(slider_rect.dim())
            .xy_relative_to(idx, slider_xy_offset)
            .graphics_for(idx)
            .parent(Some(idx))
            .color(color)
            .set(slider_idx, &mut ui);

        // The **Text** for the slider's label (if it has one).
        if let Some(label) = maybe_label {
            let label_color = style.label_color(ui.theme());
            let font_size = style.label_font_size(ui.theme());
            //const TEXT_PADDING: f64 = 10.0;
            let label_idx = state.view().label_idx.get(&mut ui);
            if is_horizontal { Text::new(label).mid_left_of(idx) }
            else             { Text::new(label).mid_bottom_of(idx) }
                .graphics_for(idx)
                .color(label_color)
                .font_size(font_size)
                .set(label_idx, &mut ui);
        }
    }

    // /// Construct an Element from the given Slider State.
    // fn draw<C>(args: widget::DrawArgs<Self, C>) -> Element
    //     where C: CharacterCache,
    // {
    //     use elmesque::form::{self, collage, text};

    //     let widget::DrawArgs { rect, state, style, theme, glyph_cache, .. } = args;
    //     let (xy, dim) = rect.xy_dim();
    //     let frame = style.frame(theme);
    //     let (inner_w, inner_h) = (dim[0] - frame * 2.0, dim[1] - frame * 2.0);
    //     let frame_color = state.color(style.frame_color(theme));
    //     let color = state.color(style.color(theme));

    //     let new_value = NumCast::from(state.value).unwrap();
    //     let is_horizontal = dim[0] > dim[1];
    //     let (pad_rel_xy, pad_dim) = if is_horizontal {
    //         // Horizontal.
    //         let value_percentage = percentage(new_value, state.min, state.max).powf(1.0/state.skew);
    //         let w = clamp(value_percentage as f64 * inner_w, 0.0, inner_w);
    //         let rel_xy = [-(inner_w - w) / 2.0, 0.0];
    //         (rel_xy, [w, inner_h])
    //     } else {
    //         // Vertical.
    //         let value_percentage = percentage(new_value, state.min, state.max).powf(1.0/state.skew);
    //         let h = clamp(value_percentage as f64 * inner_h, 0.0, inner_h);
    //         let rel_xy = [0.0, -(inner_h - h) / 2.0];
    //         (rel_xy, [inner_w, h])
    //     };

    //     // Rectangle frame / backdrop Form.
    //     let frame_form = form::rect(dim[0], dim[1])
    //         .filled(frame_color);
    //     // Slider rectangle Form.
    //     let pad_form = form::rect(pad_dim[0], pad_dim[1])
    //         .filled(color)
    //         .shift(pad_rel_xy[0], pad_rel_xy[1]);

    //     // Label Form.
    //     let maybe_label_form = state.maybe_label.as_ref().map(|label_text| {
    //         use elmesque::text::Text;
    //         use position;
    //         const TEXT_PADDING: f64 = 10.0;
    //         let label_color = style.label_color(theme);
    //         let size = style.label_font_size(theme);
    //         let label_w = glyph_cache.width(size, &label_text);
    //         let is_horizontal = dim[0] > dim[1];
    //         let l_pos = if is_horizontal {
    //             let x = position::align_left_of(dim[0], label_w) + TEXT_PADDING;
    //             [x, 0.0]
    //         } else {
    //             let y = position::align_bottom_of(dim[1], size as f64) + TEXT_PADDING;
    //             [0.0, y]
    //         };
    //         text(Text::from_string(label_text.clone()).color(label_color).height(size as f64))
    //             .shift(l_pos[0].floor(), l_pos[1].floor())
    //             .shift(xy[0].floor(), xy[1].floor())
    //     });

    //     // Chain the Forms and shift them into position.
    //     let form_chain = Some(frame_form).into_iter()
    //         .chain(Some(pad_form))
    //         .map(|form| form.shift(xy[0], xy[1]))
    //         .chain(maybe_label_form);

    //     // Collect the Forms into a renderable Element.
    //     collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
    // }

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
        self.maybe_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label Color for an Element.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn label_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.widget_style::<Self>(KIND).map(|default| {
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

