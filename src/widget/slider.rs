
use {
    Backend,
    CharacterCache,
    Color,
    Colorable,
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
    Text,
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
    /// The amount in which the slider's display should be skewed.
    ///
    /// Higher skew amounts (above 1.0) will weight lower values.
    ///
    /// Lower skew amounts (below 1.0) will weight heigher values.
    ///
    /// All skew amounts should be greater than 0.0.
    pub skew: f32,
    /// Set the reaction for the Slider.
    ///
    /// It will be triggered if the value is updated or if the mouse button is released while the
    /// cursor is above the rectangle.
    pub maybe_react: Option<F>,
    maybe_label: Option<&'a str>,
    style: Style,
    /// Whether or not user input is enabled for the Slider.
    pub enabled: bool,
}

widget_style!{
    KIND;
    /// Graphical styling unique to the Slider widget.
    style Style {
        /// The color of the slidable rectangle.
        - color: Color { theme.shape_color }
        /// The length of the frame around the edges of the slidable rectangle.
        - frame: Scalar { theme.frame_width }
        /// The color of the Slider's frame.
        - frame_color: Color { theme.frame_color }
        /// The color of the Slider's label.
        - label_color: Color { theme.label_color }
        /// The font-size for the Slider's label.
        - label_font_size: FontSize { theme.font_size_medium }
    }
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

    builder_methods!{
        pub skew { skew = f32 }
        pub react { maybe_react = Some(F) }
        pub enabled { enabled = bool }
    }

}

impl<'a, T, F> Widget for Slider<'a, T, F>
    where F: FnOnce(T),
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

    fn kid_area<C: CharacterCache>(&self, args: widget::KidAreaArgs<Self, C>) -> KidArea {
        const LABEL_PADDING: Scalar = 10.0;
        KidArea {
            rect: args.rect,
            pad: Padding {
                x: Range::new(LABEL_PADDING, LABEL_PADDING),
                y: Range::new(LABEL_PADDING, LABEL_PADDING),
            },
        }
    }

    /// Update the state of the Slider.
    fn update<B: Backend>(self, args: widget::UpdateArgs<Self, B>) {
        use self::Interaction::{Clicked, Highlighted, Normal};
        use utils::{clamp, map_range, percentage, value_from_perc};

        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let Slider { value, min, max, skew, enabled, maybe_label, maybe_react, .. } = self;

        let maybe_mouse = ui.input(idx).maybe_mouse;
        let interaction = state.view().interaction;
        let new_interaction = match (enabled, maybe_mouse) {
            (false, _) | (true, None) => Normal,
            (true, Some(mouse)) => {
                let is_over = rect.is_over(mouse.xy);
                get_new_interaction(is_over, interaction, mouse)
            },
        };

        match (interaction, new_interaction) {
            (Highlighted, Clicked) => { ui.capture_mouse(idx); },
            (Clicked, Highlighted) |
            (Clicked, Normal)      => { ui.uncapture_mouse(idx); },
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
            .parent(idx)
            .color(color)
            .set(slider_idx, &mut ui);

        // The **Text** for the slider's label (if it has one).
        if let Some(label) = maybe_label {
            let label_color = style.label_color(ui.theme());
            let font_size = style.label_font_size(ui.theme());
            //const TEXT_PADDING: f64 = 10.0;
            let label_idx = state.view().label_idx.get(&mut ui);
            Text::new(label)
                .and(|text| if is_horizontal { text.mid_left_of(idx) }
                            else { text.mid_bottom_of(idx) })
                .graphics_for(idx)
                .color(label_color)
                .font_size(font_size)
                .set(label_idx, &mut ui);
        }
    }

}


impl<'a, T, F> Colorable for Slider<'a, T, F> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, T, F> Frameable for Slider<'a, T, F> {
    builder_methods!{
        frame { style.frame = Some(Scalar) }
        frame_color { style.frame_color = Some(Color) }
    }
}

impl<'a, T, F> Labelable<'a> for Slider<'a, T, F> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
