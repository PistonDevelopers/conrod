
use {
    Backend,
    Color,
    Colorable,
    Frameable,
    FramedRectangle,
    FontSize,
    IndexSlot,
    Labelable,
    Line,
    Mouse,
    Positionable,
    Scalar,
    Sizeable,
    Text,
    Widget,
};
use num::Float;
use widget;
use utils::{map_range, val_to_string};


/// Used for displaying and controlling a 2D point on a cartesian plane within a given range.
///
/// Its reaction is triggered when the value is updated or if the mouse button is released while
/// the cursor is above the rectangle.
pub struct XYPad<'a, X, Y, F> {
    common: widget::CommonBuilder,
    x: X, min_x: X, max_x: X,
    y: Y, min_y: Y, max_y: Y,
    maybe_label: Option<&'a str>,
    /// The reaction function for the XYPad.
    ///
    /// It will be triggered when the value is updated or if the mouse button is released while the
    /// cursor is above the rectangle.
    pub maybe_react: Option<F>,
    style: Style,
    /// Indicates whether the XYPad will respond to user input.
    pub enabled: bool,
}

/// Unique kind for the widget type.
pub const KIND: widget::Kind = "XYPad";

widget_style!{
    KIND;
    /// Unique graphical styling for the XYPad.
    style Style {
        /// The color of the XYPad's rectangle.
        - color: Color { theme.shape_color }
        /// The width of the frame surrounding the rectangle.
        - frame: Scalar { theme.frame_width }
        /// The color of the surrounding rectangle frame.
        - frame_color: Color { theme.frame_color }
        /// The color of the XYPad's label and value label text.
        - label_color: Color { theme.label_color }
        /// The font size for the XYPad's label.
        - label_font_size: FontSize { theme.font_size_medium }
        /// The font size for the XYPad's *value* label.
        - value_font_size: FontSize { 14 }
        /// The thickness of the XYPad's crosshair lines.
        - line_thickness: Scalar { 2.0 }
    }
}

/// The state of the XYPad.
#[derive(Clone, Debug, PartialEq)]
pub struct State<X, Y> {
    x: X, min_x: X, max_x: X,
    y: Y, min_y: Y, max_y: Y,
    interaction: Interaction,
    rectangle_idx: IndexSlot,
    label_idx: IndexSlot,
    h_line_idx: IndexSlot,
    v_line_idx: IndexSlot,
    value_label_idx: IndexSlot,
}

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

    builder_methods!{
        pub line_thickness { style.line_thickness = Some(Scalar) }
        pub value_font_size { style.value_font_size = Some(FontSize) }
        pub react { maybe_react = Some(F) }
        pub enabled { enabled = bool }
    }

}

impl<'a, X, Y, F> Widget for XYPad<'a, X, Y, F>
    where X: Float + ToString + ::std::fmt::Debug + ::std::any::Any,
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

    fn init_state(&self) -> Self::State {
        State {
            interaction: Interaction::Normal,
            x: self.x, min_x: self.min_x, max_x: self.max_x,
            y: self.y, min_y: self.min_y, max_y: self.max_y,
            rectangle_idx: IndexSlot::new(),
            label_idx: IndexSlot::new(),
            h_line_idx: IndexSlot::new(),
            v_line_idx: IndexSlot::new(),
            value_label_idx: IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the XYPad's cached state.
    fn update<B: Backend>(self, args: widget::UpdateArgs<Self, B>) {
        use position::{Direction, Edge};
        use self::Interaction::{Clicked, Highlighted, Normal};

        let widget::UpdateArgs { idx, state, rect, style, mut ui, .. } = args;
        let XYPad {
            enabled,
            x, min_x, max_x,
            y, min_y, max_y,
            maybe_label,
            maybe_react,
            ..
        } = self;

        let maybe_mouse = ui.input(idx).maybe_mouse;
        let frame = style.frame(ui.theme());
        let inner_rect = rect.pad(frame);
        let interaction = state.view().interaction;
        let new_interaction = match (enabled, maybe_mouse) {
            (false, _) | (true, None) => Normal,
            (true, Some(mouse)) => {
                let is_over_inner = inner_rect.is_over(mouse.xy);
                get_new_interaction(is_over_inner, interaction, mouse)
            },
        };

        // Capture the mouse if clicked, uncapture if released.
        match (interaction, new_interaction) {
            (Highlighted, Clicked) => { ui.capture_mouse(idx); },
            (Clicked, Highlighted) | (Clicked, Normal) => { ui.uncapture_mouse(idx); },
            _ => (),
        }

        // Determine new values from the mouse position over the pad.
        let (new_x, new_y) = match (maybe_mouse, new_interaction) {
            (None, _) | (_, Normal) | (_, Highlighted) => (x, y),
            (Some(mouse), Clicked) => {
                let clamped_x = inner_rect.x.clamp_value(mouse.xy[0]);
                let clamped_y = inner_rect.y.clamp_value(mouse.xy[1]);
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
                state.x = x;
                state.y = y;
                state.min_x = min_x;
                state.max_x = max_x;
                state.min_y = min_y;
                state.max_y = max_y;
            })
        }

        // The backdrop **FramedRectangle** widget.
        let dim = rect.dim();
        let color = new_interaction.color(style.color(ui.theme()));
        let frame = style.frame(ui.theme());
        let frame_color = style.frame_color(ui.theme());
        let rectangle_idx = state.view().rectangle_idx.get(&mut ui);
        FramedRectangle::new(dim)
            .middle_of(idx)
            .graphics_for(idx)
            .color(color)
            .frame(frame)
            .frame_color(frame_color)
            .set(rectangle_idx, &mut ui);

        // Label **Text** widget.
        let label_color = style.label_color(ui.theme());
        if let Some(label) = maybe_label {
            let label_idx = state.view().label_idx.get(&mut ui);
            let label_font_size = style.label_font_size(ui.theme());
            Text::new(label)
                .middle_of(rectangle_idx)
                .graphics_for(idx)
                .color(label_color)
                .font_size(label_font_size)
                .set(label_idx, &mut ui);
        }

        // Crosshair **Line** widgets.
        let (w, h) = inner_rect.w_h();
        let half_w = w / 2.0;
        let half_h = h / 2.0;
        let v_line_x = map_range(new_x, min_x, max_x, -half_w, half_w);
        let h_line_y = map_range(new_y, min_y, max_y, -half_h, half_h);
        let thickness = style.line_thickness(ui.theme());
        let line_color = label_color.with_alpha(1.0);

        let v_line_start = [0.0, -half_h];
        let v_line_end = [0.0, half_h];
        let v_line_idx = state.view().v_line_idx.get(&mut ui);
        Line::centred(v_line_start, v_line_end)
            .color(line_color)
            .thickness(thickness)
            .x_y_relative_to(idx, v_line_x, 0.0)
            .graphics_for(idx)
            .parent(idx)
            .set(v_line_idx, &mut ui);

        let h_line_start = [-half_w, 0.0];
        let h_line_end = [half_w, 0.0];
        let h_line_idx = state.view().h_line_idx.get(&mut ui);
        Line::centred(h_line_start, h_line_end)
            .color(line_color)
            .thickness(thickness)
            .x_y_relative_to(idx, 0.0, h_line_y)
            .graphics_for(idx)
            .parent(idx)
            .set(h_line_idx, &mut ui);

        // Crosshair value label **Text** widget.
        let x_string = val_to_string(new_x, max_x, max_x - min_x, rect.w() as usize);
        let y_string = val_to_string(new_y, max_y, max_y - min_y, rect.h() as usize);
        let value_string = format!("{}, {}", x_string, y_string);
        let cross_hair_xy = [inner_rect.x() + v_line_x, inner_rect.y() + h_line_y];
        const VALUE_TEXT_PAD: f64 = 5.0;
        let x_direction = match inner_rect.x.closest_edge(cross_hair_xy[0]) {
            Edge::End => Direction::Backwards,
            Edge::Start => Direction::Forwards,
        };
        let y_direction = match inner_rect.y.closest_edge(cross_hair_xy[1]) {
            Edge::End => Direction::Backwards,
            Edge::Start => Direction::Forwards,
        };
        let value_font_size = style.value_font_size(ui.theme());
        let value_label_idx = state.view().value_label_idx.get(&mut ui);
        Text::new(&value_string)
            .x_direction_from(v_line_idx, x_direction, VALUE_TEXT_PAD)
            .y_direction_from(h_line_idx, y_direction, VALUE_TEXT_PAD)
            .color(line_color)
            .graphics_for(idx)
            .parent(idx)
            .font_size(value_font_size)
            .set(value_label_idx, &mut ui);
    }

}


impl<'a, X, Y, F> Colorable for XYPad<'a, X, Y, F> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, X, Y, F> Frameable for XYPad<'a, X, Y, F> {
    builder_methods!{
        frame { style.frame = Some(Scalar) }
        frame_color { style.frame_color = Some(Color) }
    }
}

impl<'a, X, Y, F> Labelable<'a> for XYPad<'a, X, Y, F> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
