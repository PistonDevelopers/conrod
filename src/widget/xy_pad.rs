
use {
    CharacterCache,
    Color,
    Colorable,
    Corner,
    Dimension,
    Frameable,
    FontSize,
    Labelable,
    Line,
    Mouse,
    Positionable,
    Scalar,
    Sizeable,
    Theme,
    Ui,
    Widget,
};
use position;
use widget;
use utils::{clamp, map_range, val_to_string};
use vecmath::vec2_sub;


/// Used for displaying and controlling a 2D point on a cartesian plane within a given range.
///
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
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    /// The color of the XYPad's rectangle.
    pub maybe_color: Option<Color>,
    /// The width of the frame surrounding the rectangle.
    pub maybe_frame: Option<Scalar>,
    /// The color of the surrounding rectangle frame.
    pub maybe_frame_color: Option<Color>,
    /// The color of the XYPad's label and value label text.
    pub maybe_label_color: Option<Color>,
    /// The font size for the XYPad's label.
    pub maybe_label_font_size: Option<FontSize>,
    /// The font size for the XYPad's *value* label.
    pub maybe_value_font_size: Option<FontSize>,
    /// The thickness of the XYPad's crosshair lines.
    pub maybe_line_thickness: Option<f64>,
}

/// The state of the XYPad.
#[derive(Clone, Debug, PartialEq)]
pub struct State<X, Y> {
    x: X, min_x: X, max_x: X,
    y: Y, min_y: Y, max_y: Y,
    interaction: Interaction,
    maybe_frame_idx: Option<NodeIndex>,
    maybe_label_idx: Option<NodeIndex>,
    maybe_h_line_idx: Option<NodeIndex>,
    maybe_v_line_idx: Option<NodeIndex>,
    maybe_value_label_idx: Option<NodeIndex>,
}

/// Unique kind for the widget type.
pub const KIND: widget::Kind = "XYPad";

/// The interaction state of the XYPad.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Interaction {
    Normal,
    Highlighted,
    Clicked,
}

impl Interaction {
    /// The color associated with the current state.
    fn color(&self, color: Color) -> Color {
        match *self {
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

    /// Build a new XYPad widget.
    pub fn new(x_val: X, min_x: X, max_x: X, y_val: Y, min_y: Y, max_y: Y) -> Self {
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
    pub fn line_thickness(mut self, width: f64) -> Self {
        self.style.maybe_line_thickness = Some(width);
        self
    }

    /// Set the font size for the displayed crosshair value.
    pub fn value_font_size(mut self, size: FontSize) -> Self {
        self.style.maybe_value_font_size = Some(size);
        self
    }

    /// Set the reaction for the XYPad.
    ///
    /// It will be triggered when the value is updated or if the mouse button is released while the
    /// cursor is above the rectangle.
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
        F: FnOnce(X, Y),
{
    type State = State<X, Y>;
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

    fn init_state(&self) -> State<X, Y> {
        State {
            interaction: Interaction::Normal,
            x: self.x, min_x: self.min_x, max_x: self.max_x,
            y: self.y, min_y: self.min_y, max_y: self.max_y,
            maybe_frame_idx: None,
            maybe_rectangle_idx: None,
            maybe_label_idx: None,
            maybe_h_line_idx: None,
            maybe_v_line_idx: None,
            maybe_value_label_idx: None,
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn default_x_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        widget::default_x_dimension(self, ui).unwrap_or(Dimension::Absolute(128.0))
    }

    fn default_y_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        widget::default_y_dimension(self, ui).unwrap_or(Dimension::Absolute(128.0))
    }

    /// Update the XYPad's cached state.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        use self::Interaction::{Clicked, Highlighted, Normal};

        let widget::UpdateArgs { state, rect, style, mut ui, .. } = args;
        let XYPad {
            enabled,
            x, min_x, max_x,
            y, min_y, max_y,
            maybe_label,
            maybe_react,
        } = self;

        let maybe_mouse = ui.input().maybe_mouse;
        let frame = style.frame(ui.theme());
        let inner_rect = rect.pad(frame);
        let interaction = state.view().interaction;
        let new_interaction = match (self.enabled, maybe_mouse) {
            (false, _) | (true, None) => Normal,
            (true, Some(mouse)) => {
                let is_over_inner = inner_rect.is_over(mouse.xy);
                get_new_interaction(is_over_inner, interaction, mouse)
            },
        };

        // Capture the mouse if clicked, uncapture if released.
        match (interaction, new_interaction) {
            (Highlighted, Clicked) => { ui.capture_mouse(); },
            (Clicked, Highlighted) | (Clicked, Normal) => { ui.uncapture_mouse(); },
            _ => (),
        }

        // Determine new values from the mouse position over the pad.
        let (new_x, new_y) = match (maybe_mouse, new_interaction) {
            (None, _) | (_, Normal) | (_, Highlighted) => (self.x, self.y),
            (Some(mouse), Clicked) => {
                let unclamped_x = mouse.xy[0] - inner_rect.left();
                let unclamped_y = mouse.xy[1] - inner_rect.bottom();
                let clamped_x = inner_rect.x.clamp_value(unclamped_x);
                let clamped_y = inner_rect.y.clamp_value(unclamped_y);
                let (l, r, b, t) = inner_rect.l_r_b_t();
                let new_x = map_range(clamped_x, l, r, min_x, max_x);
                let new_y = map_range(clamped_y, b, t, min_y, max_y);
                (new_x, new_y)
            },
        };

        // React if value is changed or the pad is clicked/released.
        if let Some(react) = maybe_react {
            let should_react = x != new_x || y != new_y
                || (interaction == Highlighted && new_interaction == Clicked)
                || (interaction == Clicked && new_interaction == Highlighted);
            if should_react {
                react(new_x, new_y);
            }
        }

        if interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        let value_or_bounds_have_changed = {
            let v = state.view();
            v.x != x || v.y != y
                || v.min_x != min_x || v.max_x != max_x
                || v.min_y != min_y || v.max_y != max_y
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

        // The backdrop **FramedRectangle** widget.
        let dim = rect.dim();
        let color = style.color(ui.theme());
        let frame = style.frame(ui.theme());
        let frame_color = style.frame_color(ui.theme());
        let rectangle_idx = state.view().maybe_rectangle_idx
            .unwrap_or_else(|| ui.new_unique_node_index());
        FramedRectangle::new(dim)
            .middle_of(idx)
            .graphics_for(idx)
            .color(color)
            .frame(frame)
            .frame_color(frame_color)
            .set(rectangle_idx, &mut ui);

        // Label **Text** widget.
        let label_color = style.label_color(ui.theme());
        let maybe_label_idx = maybe_label.map(|label| {
            let label_idx = state.view().maybe_label_idx
                .unwrap_or_else(|| ui.new_unique_node_index());
            let label_font_size = style.label_font_size(ui.theme());
            Text::new(label)
                .middle_of(rectangle_idx)
                .graphics_for(idx)
                .color(label_color)
                .font_size(label_font_size)
                .set(label_idx, &mut ui);
            label_idx
        });

        // Crosshair **Line** widgets.
        let (w, h) = inner_rect.w_h();
        let half_w = w / 2.0;
        let half_h = h / 2.0;
        let v_line_x = map_range(new_x, min_x, max_x, -half_w, half_w);
        let h_line_y = map_range(new_y, min_y, max_y, -half_h, half_h);
        let thickness = style.line_thickness(ui.theme());

        let v_line_start = [0.0, -half_h];
        let v_line_end = [0.0, half_h];
        let v_line_idx = state.view().maybe_v_line_idx
            .unwrap_or_else(|| ui.new_unique_node_index());
        Line::centred(v_line_start, v_line_end)
            .color(label_color)
            .relative_to(idx, [v_line_x, 0.0])
            .graphics_for(idx)
            .set(v_line_idx, &mut ui);

        let h_line_start = [-half_w, 0.0];
        let h_line_end = [half_w, 0.0];
        let h_line_idx = state.view().maybe_h_line_idx
            .unwrap_or_else(|| ui.new_unique_node_index());
        Line::centred(h_line_start, h_line_end)
            .color(label_color)
            .relative_to(idx, [0.0, h_line_y])
            .graphics_for(idx)
            .set(h_line_idx, &mut ui);

        // Crosshair value label **Text** widget.
        let x_string = val_to_string(new_x, max_x, max_x - min_x, rect.w() as usize);
        let y_string = val_to_string(new_y, max_y, max_y - min_y, rect.h() as usize);
        let value_string = format!("{}, {}", x_string, y_string);
        let cross_hair_xy = [inner_rect.x() + v_line_x, inner_rect.y() + h_line_y];
        const VALUE_TEXT_PAD: f64 = 5.0;
        let mut text = Text::new(&value_string);
        // text = match inner_rect.closest_corner(cross_hair_xy) {
        //     Corner::TopLeft => text.,
        //     Corner::TopRight => [-VALUE_TEXT_PAD, -VALUE_TEXT_PAD],
        //     Corner::BottomLeft => [VALUE_TEXT_PAD, VALUE_TEXT_PAD],
    }

    // /// Construct an Element from the given XYPad State.
    // fn draw<C: CharacterCache>(args: widget::DrawArgs<Self, C>) -> Element {
    //     use elmesque::form::{self, collage, line, solid, text};
    //     use elmesque::text::Text;

    //     let widget::DrawArgs { rect, state, style, theme, glyph_cache, .. } = args;
    //     let (xy, dim) = rect.xy_dim();
    //     let frame = style.frame(theme);
    //     let pad_dim = vec2_sub(dim, [frame * 2.0; 2]);
    //     let (half_pad_w, half_pad_h) = (pad_dim[0] / 2.0, pad_dim[1] / 2.0);

    //     // Construct the frame and inner rectangle Forms.
    //     let color = state.color(style.color(theme));
    //     let frame_color = style.frame_color(theme);
    //     let frame_form = form::rect(dim[0], dim[1]).filled(frame_color);
    //     let pressable_form = form::rect(pad_dim[0], pad_dim[1]).filled(color);

    //     // Construct the label Form.
    //     let maybe_label_form = state.maybe_label.as_ref().map(|l_text| {
    //         let l_color = style.label_color(theme);
    //         let l_size = style.label_font_size(theme) as f64;
    //         text(Text::from_string(l_text.clone()).color(l_color).height(l_size))
    //     });

    //     // Construct the crosshair line Forms.
    //     let ch_x = map_range(state.x, state.min_x, state.max_x, -half_pad_w, half_pad_w).floor();
    //     let ch_y = map_range(state.y, state.min_y, state.max_y, -half_pad_h, half_pad_h).floor();
    //     let line_thickness = style.line_thickness(theme);
    //     let line_style = solid(color.plain_contrast()).width(line_thickness);
    //     let vert_form = line(line_style.clone(), 0.0, -half_pad_h, 0.0, half_pad_h).shift_x(ch_x);
    //     let hori_form = line(line_style, -half_pad_w, 0.0, half_pad_w, 0.0).shift_y(ch_y);

    //     // Construct the value string Form.
    //     let x_string = val_to_string(state.x, state.max_x, state.max_x - state.min_x, dim[0] as usize);
    //     let y_string = val_to_string(state.y, state.max_y, state.max_y - state.min_y, dim[1] as usize);
    //     let value_string = format!("{}, {}", x_string, y_string);
    //     let value_text_form = {
    //         const PAD: f64 = 5.0; // Slight padding between the crosshair and the text.
    //         let value_font_size = style.value_font_size(theme);
    //         let w = glyph_cache.width(value_font_size, &value_string);
    //         let h = value_font_size as f64;
    //         let x_shift = w / 2.0 + PAD;
    //         let y_shift = h / 2.0 + PAD;
    //         let (value_text_x, value_text_y) = match position::corner([ch_x, ch_y], pad_dim) {
    //             Corner::TopLeft => (x_shift, -y_shift),
    //             Corner::TopRight => (-x_shift, -y_shift),
    //             Corner::BottomLeft => (x_shift, y_shift),
    //             Corner::BottomRight => (-x_shift, y_shift),
    //         };
    //         text(Text::from_string(value_string).color(color.plain_contrast()).height(h))
    //             .shift(ch_x, ch_y)
    //             .shift(value_text_x.floor(), value_text_y.floor())
    //     };

    //     // Chain the Forms and shift them into position.
    //     let form_chain = Some(frame_form).into_iter()
    //         .chain(Some(pressable_form).into_iter())
    //         .chain(maybe_label_form.into_iter())
    //         .chain(Some(vert_form).into_iter())
    //         .chain(Some(hori_form).into_iter())
    //         .chain(Some(value_text_form).into_iter())
    //         .map(|form| form.shift(xy[0].round(), xy[1].round()));

    //     // Turn the form into a renderable Element.
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
            maybe_value_font_size: None,
            maybe_line_thickness: None,
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

    /// Get the value font size for an Element.
    pub fn value_font_size(&self, theme: &Theme) -> FontSize {
        const DEFAULT_VALUE_FONT_SIZE: u32 = 14;
        self.maybe_value_font_size.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_value_font_size.unwrap_or(DEFAULT_VALUE_FONT_SIZE)
        })).unwrap_or(DEFAULT_VALUE_FONT_SIZE)
    }

    /// Get the point radius size for an Element.
    pub fn line_thickness(&self, theme: &Theme) -> f64 {
        const DEFAULT_LINE_THICKNESS: f64 = 2.0;
        self.maybe_line_thickness.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_line_thickness.unwrap_or(DEFAULT_LINE_THICKNESS)
        })).unwrap_or(DEFAULT_LINE_THICKNESS)
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

