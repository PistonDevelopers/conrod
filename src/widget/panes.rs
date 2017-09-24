//! The `PaneCanvas` widget and related items.

use {
    Color,
    Colorable,
    FontSize,
    Borderable,
    Labelable,
    Positionable,
    Sizeable,
    Theme,
    Ui,
    UiCell,
    Widget,
};
use position::{self, Dimensions, Padding, Place, Position, Range, Rect, Scalar};
use position::Direction::{Forwards, Backwards};
use text;
use widget;
use cursor;

/// **PaneCanvas** is designed to be a "container"-like "parent" widget that simplifies placement of
/// "children" widgets.
///
/// Widgets can be placed on a **PaneCanvas** in a variety of ways using methods from the
/// [**Positionable**](../position/trait.Positionable) trait.
///
/// **PaneCanvas** provides methods for padding the kid widget area which can make using the
/// **Place**-related **Positionable** methods a little easier.
///
/// A **PaneCanvas** can also be divided into a sequence of smaller **PaneCanvas**ses using the `.flow_*`
/// methods. This creates a kind of **PaneCanvas** tree, where each "split" can be sized using the
/// `.length` or `.length_weight` methods.
///
/// See the `canvas.rs` example for a demonstration of the **PaneCanvas** type.
#[derive(Clone, Debug, WidgetCommon_)]
pub struct PaneCanvas<'a> {
    /// Data necessary and common for all widget builder types.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// The builder data related to the style of the PaneCanvas.
    pub style: Style,
    /// The label for the **PaneCanvas**' **TitleBar** if there is one.
    pub maybe_title_bar_label: Option<&'a str>, 
    /// A list of child **PaneCanvas**ses as splits of this **PaneCanvas** flowing in the given direction.
    pub maybe_splits: Option<FlowOfSplits<'a>>,
}

/// **PaneCanvas** state to be cached.
pub struct State {
    ids: Ids,
    /// Sizes of splits if any.
    split_weights: Vec<Scalar>,
}

widget_ids! {
    struct Ids {
        rectangle,
        title_bar,
        dividers[],
    }
}

/// Unique styling for the PaneCanvas.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// The color of the PaneCanvas' rectangle surface.
    #[conrod(default = "theme.background_color")]
    pub color: Option<Color>,
    /// The width of the border surrounding the PaneCanvas' rectangle.
    #[conrod(default = "theme.border_width")]
    pub border: Option<Scalar>,
    /// The color of the PaneCanvas' border.
    #[conrod(default = "theme.border_color")]
    pub border_color: Option<Color>,
    /// If this PaneCanvas is a split of some parent PaneCanvas, this is the length of the split.
    #[conrod(default = "Length::Weight(1.0)")]
    pub length: Option<Length>,
    /// If this PaneCanvas is a split of some parent PaneCanvas, this is the minimum allowed length of the split.
    #[conrod(default = "Length::Weight(0.5)")]
    pub min_length: Option<Length>,

    /// Width of the divider between splits. By default, dividers are 5 pixels wide.
    #[conrod(default = "5.0")]
    pub divider_width: Option<Scalar>,
    /// Color of the divider between splits.
    #[conrod(default = "theme.background_color")]
    pub divider_color: Option<Color>,
    /// Padding between the splits. By default this is 0.
    ///
    /// NOTE: if divider_width > pad_splits then the divider will overlap with neighbouring splits.
    /// This can be useful if you don't want to display the divider, but you still want to have a
    /// reasonably sized handle.
    #[conrod(default = "0.0")]
    pub pad_splits: Option<Scalar>,

    /// Padding for the left edge of the PaneCanvas' kid area.
    #[conrod(default = "theme.padding.x.start")]
    pub pad_left: Option<Scalar>,
    /// Padding for the right edge of the PaneCanvas' kid area.
    #[conrod(default = "theme.padding.x.end")]
    pub pad_right: Option<Scalar>,
    /// Padding for the bottom edge of the PaneCanvas' kid area.
    #[conrod(default = "theme.padding.y.start")]
    pub pad_bottom: Option<Scalar>,
    /// Padding for the top edge of the PaneCanvas' kid area.
    #[conrod(default = "theme.padding.y.end")]
    pub pad_top: Option<Scalar>,

    /// The color of the title bar. Defaults to the color of the PaneCanvas.
    #[conrod(default = "None")]
    pub title_bar_color: Option<Option<Color>>,
    /// The color of the title bar's text.
    #[conrod(default = "theme.label_color")]
    pub title_bar_text_color: Option<Color>,
    /// The font size for the title bar's text.
    #[conrod(default = "theme.font_size_medium")]
    pub title_bar_font_size: Option<FontSize>,
    /// The way in which the title bar's text should wrap.
    #[conrod(default = "Some(widget::text::Wrap::Whitespace)")]
    pub title_bar_maybe_wrap: Option<Option<widget::text::Wrap>>,
    /// The distance between lines for multi-line title bar text.
    #[conrod(default = "1.0")]
    pub title_bar_line_spacing: Option<Scalar>,
    /// The label's typographic alignment over the *x* axis.
    #[conrod(default = "text::Justify::Center")]
    pub title_bar_justify: Option<text::Justify>,
}

/// A series of **PaneCanvas** splits along with their unique identifiers.
pub type ListOfSplits<'a> = &'a [(widget::Id, PaneCanvas<'a>)];

/// A series of **PaneCanvas** splits flowing in the specified direction.
pub type FlowOfSplits<'a> = (Direction, ListOfSplits<'a>);

/// The length of a `Split` given as a weight.
///
/// The length is determined by determining what percentage each `Split`'s weight contributes to
/// the total weight of all `Split`s in a flow list.
pub type Weight = Scalar;

/// Used to describe the desired length for a `Split`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Length {
    /// The length as an absolute scalar.
    Absolute(Scalar),
    /// The length as a weight of the non-absolute length of the parent **PaneCanvas**.
    Weight(Weight),
}

/// The direction in which a sequence of canvas splits will be laid out.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    /// Lay splits along the *x* axis.
    X(position::Direction),
    /// Lay splits along the *y* axis.
    Y(position::Direction),
}


impl<'a> PaneCanvas<'a> {

    /// Construct a new PaneCanvas builder.
    pub fn new() -> Self {
        PaneCanvas {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            maybe_title_bar_label: None,
            maybe_splits: None,
        }
    }

    builder_methods!{
        pub title_bar { maybe_title_bar_label = Some(&'a str) }
        pub pad_left { style.pad_left = Some(Scalar) }
        pub pad_right { style.pad_right = Some(Scalar) }
        pub pad_bottom { style.pad_bottom = Some(Scalar) }
        pub pad_top { style.pad_top = Some(Scalar) }
        pub with_style { style = Style }
    }

    /// Set the initial length of the Split as an absolute scalar.
    pub fn length(mut self, length: Scalar) -> Self {
        self.style.length = Some(Length::Absolute(length));
        self
    }

    /// Set the initial length of the Split as a weight.
    ///
    /// The default length weight for each widget is `1.0`.
    pub fn length_weight(mut self, weight: Weight) -> Self {
        self.style.length = Some(Length::Weight(weight));
        self
    }

    /// Set the minimum length of the Split as an absolute scalar.
    pub fn min_length(mut self, length: Scalar) -> Self {
        self.style.min_length = Some(Length::Absolute(length));
        self
    }

    /// Set the minimum length of the Split as a weight.
    ///
    /// The default min length weight for each widget is `0.5`.
    pub fn min_length_weight(mut self, weight: Weight) -> Self {
        self.style.min_length = Some(Length::Weight(weight));
        self
    }

    /// Set the child PaneCanvas Splits of the current PaneCanvas flowing in a given direction.
    fn flow(mut self, direction: Direction, splits: ListOfSplits<'a>) -> Self {
        self.maybe_splits = Some((direction, splits));
        self
    }

    /// Set the child PaneCanvasses flowing to the right.
    pub fn flow_right(self, splits: ListOfSplits<'a>) -> Self {
        self.flow(Direction::X(Forwards), splits)
    }

    /// Set the child PaneCanvasses flowing to the left.
    pub fn flow_left(self, splits: ListOfSplits<'a>) -> Self {
        self.flow(Direction::X(Backwards), splits)
    }

    /// Set the child PaneCanvasses flowing upwards.
    pub fn flow_up(self, splits: ListOfSplits<'a>) -> Self {
        self.flow(Direction::Y(Forwards), splits)
    }

    /// Set the child PaneCanvasses flowing downwards.
    pub fn flow_down(self, splits: ListOfSplits<'a>) -> Self {
        self.flow(Direction::Y(Backwards), splits)
    }

    /// Set the width of the Split divider. By default, dividers are 5 pixels wide.
    /// Setting this to 0.0 effectively disables dividers, which makes this widget act exactly like
    /// the **Canvas** widget.
    pub fn divider_width(mut self, width: Scalar) -> Self {
        self.style.divider_width = Some(width);
        self
    }

    /// Set the color of the Split divider.
    pub fn divider_color(mut self, color: Color) -> Self {
        self.style.divider_color = Some(color);
        self
    }

    /// Set the padding between splits. By default this is 0.
    ///
    /// NOTE: if divider_width > pad_splits then the divider will overlap with neighbouring splits.
    /// This can be useful if you don't want to display the divider, but you still want to have a
    /// reasonably sized handle.
    pub fn pad_splits(mut self, pad: Scalar) -> Self {
        self.style.pad_splits = Some(pad);
        self
    }

    /// Set the padding for all edges of the area where child widgets will be placed.
    #[inline]
    pub fn pad(self, pad: Scalar) -> Self {
        self.pad_left(pad).pad_right(pad).pad_bottom(pad).pad_top(pad)
    }

    /// Set the padding of the area where child widgets will be placed.
    #[inline]
    pub fn padding(self, pad: Padding) -> Self {
        self.pad_left(pad.x.start)
            .pad_right(pad.x.end)
            .pad_bottom(pad.y.start)
            .pad_top(pad.y.end)
    }

    /// Set the color of the `PaneCanvas`' `TitleBar` if it is visible.
    pub fn title_bar_color(mut self, color: Color) -> Self {
        self.style.title_bar_color = Some(Some(color));
        self
    }

}


impl<'a> Widget for PaneCanvas<'a> {
    type State = State;
    type Style = Style;
    type Event = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
            split_weights: Vec::new(),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn default_x_position(&self, _ui: &Ui) -> Position {
        Position::Relative(position::Relative::Place(Place::Middle), None)
    }

    fn default_y_position(&self, _ui: &Ui) -> Position {
        Position::Relative(position::Relative::Place(Place::Middle), None)
    }

    /// The title bar area at which the PaneCanvas can be clicked and dragged.
    ///
    /// Note: the position of the returned **Rect** should be relative to the center of the widget.
    fn drag_area(&self, dim: Dimensions, style: &Style, theme: &Theme) -> Option<Rect> {
        self.maybe_title_bar_label.map(|_| {
            let font_size = style.title_bar_font_size(theme);
            let (h, rel_y) = title_bar_h_rel_y(dim[1], font_size);
            let rel_xy = [0.0, rel_y];
            let dim = [dim[0], h];
            Rect::from_xy_dim(rel_xy, dim)
        })
    }

    /// The area of the widget below the title bar, upon which child widgets will be placed.
    fn kid_area(&self, args: widget::KidAreaArgs<Self>) -> widget::KidArea {
        let widget::KidAreaArgs { rect, style, theme, .. } = args;
        if self.maybe_title_bar_label.is_some() {
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

    /// Update the state of the PaneCanvas.
    fn update(self, args: widget::UpdateArgs<Self>) {
        let widget::UpdateArgs { id, state, rect, mut ui, .. } = args;
        let PaneCanvas { style, maybe_title_bar_label, maybe_splits, .. } = self;

        // BorderedRectangle widget as the rectangle backdrop.
        let dim = rect.dim();
        let color = style.color(ui.theme());
        let border = style.border(ui.theme());
        let border_color = style.border_color(ui.theme());
        widget::BorderedRectangle::new(dim)
            .color(color)
            .border(border)
            .border_color(border_color)
            .middle_of(id)
            .graphics_for(id)
            .place_on_kid_area(false)
            .set(state.ids.rectangle, &mut ui);

        // TitleBar widget if we were given some label.
        if let Some(label) = maybe_title_bar_label {
            let color = style.title_bar_color(&ui.theme).unwrap_or(color);
            let font_size = style.title_bar_font_size(&ui.theme);
            let label_color = style.title_bar_text_color(&ui.theme);
            let justify = style.title_bar_justify(&ui.theme);
            let line_spacing = style.title_bar_line_spacing(&ui.theme);
            let maybe_wrap = style.title_bar_maybe_wrap(&ui.theme);
            widget::TitleBar::new(label, state.ids.rectangle)
                .and_mut(|title_bar| {
                    title_bar.style.maybe_wrap = Some(maybe_wrap);
                    title_bar.style.justify = Some(justify);
                })
                .color(color)
                .border(border)
                .border_color(border_color)
                .label_font_size(font_size)
                .label_color(label_color)
                .line_spacing(line_spacing)
                .graphics_for(id)
                .place_on_kid_area(false)
                .set(state.ids.title_bar, &mut ui);
        }

        // If we were given some child canvas splits, we should instantiate them.
        if let Some((direction, splits)) = maybe_splits {
            // Divider widget as a draggable button
            let divider_width = style.divider_width(ui.theme());
            let split_padding = style.pad_splits(ui.theme());
            let divider_color = style.divider_color(ui.theme());

            let last_split_idx = splits.len() - 1;

            // Ensure we have enough divider Ids.
            if state.ids.dividers.len() != last_split_idx {
                state.update(|state| {
                    state.ids.dividers.resize(last_split_idx, &mut ui.widget_id_generator());
                });
            }

            // No need to calculate kid_area again, we'll just get it from the graph.
            let kid_area = ui.kid_area_of(id).expect("No KidArea found");

            let kid_area_range = match direction {
                Direction::X(_) => kid_area.x,
                Direction::Y(_) => kid_area.y,
            };

            // Total length available for child splits
            let total_length = kid_area_range.len();

            let (total_abs, total_weight) =
                splits.iter().fold((0.0, 0.0), |(abs, weight), &(_, ref split)| {
                    match split.style.length(ui.theme()) {
                        Length::Absolute(a) => (abs + a, weight),
                        Length::Weight(w) => (abs, weight + w),
                    }
                });

            let non_abs_length = (total_length - total_abs).max(0.0);
            let weight_normaliser = 1.0 / total_weight;

            // Compute the length of a given split
            let to_abs_length = |len: Length| -> Scalar {
                match len {
                    // Length of the absolutely set split sizes
                    Length::Absolute(length) => length,
                    // Length of the sizes set by a weight as a percentage of what hasn't been
                    // occupied by the absolutely sized splits
                    Length::Weight(weight) => weight * weight_normaliser * non_abs_length,
                }
            };

            // Initialize split sizes if not yet initialized or if they had been changed
            if state.split_weights.len() != splits.len() {

                state.update(|state| {
                    state.split_weights.clear();
                    for &(_, ref split) in splits.iter() {
                        let length = split.style.length(ui.theme());
                        // Compute the percentage of the total length occupied by each split.
                        state.split_weights.push(to_abs_length(length) / total_length);
                    }
                });
            }

            debug_assert!(splits.len() == state.ids.dividers.len() + 1);

            let set_split = |split_id: widget::Id, split: PaneCanvas<'a>, ui: &mut UiCell| {
                split.parent(id).set(split_id, ui);
            };

            // Instantiate each of the splits, matching on the direction first for efficiency.
            macro_rules! draw_splits {
                ($inner:ident,
                 $outer:ident,
                 $dir:ident,
                 $div_dir:ident,
                 $init_dir:ident,
                 $axis_index:expr,
                 $sign:expr) => {{
                    // Determine the split sizes from divider drag events
                    for (i, &divider_id) in state.ids.dividers.clone().iter().enumerate() {
                        if divider_width > 0.0 {
                            let prev_split_size = total_length * state.split_weights[i];
                            let next_split_size = total_length * state.split_weights[i+1];

                            for drag in ui.widget_input(divider_id).drags().left() {
                                let delta = $sign * drag.delta_xy[$axis_index];
                                let prev_target_size = prev_split_size + delta;
                                let next_target_size = next_split_size - delta;
                                let prev_min_size = to_abs_length(splits[i].1.style.min_length(ui.theme())) 
                                                  + if i == 0 { 0.5 } else { 1.0 } * split_padding;
                                let next_min_size = to_abs_length(splits[i+1].1.style.min_length(ui.theme()))
                                                  + if i+1 == last_split_idx { 0.5 } else { 1.0 } * split_padding;
                                // clamp allowed delta if it surpasses the allowable size
                                let allowed_delta = 
                                    if prev_target_size < prev_min_size {
                                        prev_min_size - prev_split_size
                                    } else if next_target_size < next_min_size {
                                        next_split_size - next_min_size
                                    } else { delta };
                                state.update(|state| {
                                    state.split_weights[i] = (prev_split_size + allowed_delta) / total_length;
                                    state.split_weights[i+1] = (next_split_size - allowed_delta) / total_length;
                                });
                            }
                        }
                    }

                    // Actually build the splits
                    for (i, &(split_id, ref split)) in splits.iter().enumerate() {
                        let size = total_length * state.split_weights[i];

                        let split = match i {
                            0 => split.clone()
                                .$outer(kid_area.$outer())
                                .$init_dir()
                                .$inner(size - 0.5*split_padding),
                            j => {
                                let offset = split_padding *
                                    if j == last_split_idx { 0.5 } else { 1.0 };
                                split.clone().$dir(split_padding).$inner(size - offset)
                            },
                        };
                        set_split(split_id, split, &mut ui);
                    }

                    // Then add the dividers possibly overlapping the splits
                    for (i, &divider_id) in state.ids.dividers.clone().iter().enumerate() {
                        if divider_width > 0.0 {
                            if let Some(_) = ui.widget_input(divider_id).mouse() {
                                ui.set_mouse_cursor(
                                    if $axis_index == 0 {
                                        cursor::MouseCursor::ResizeHorizontal
                                    } else { 
                                        cursor::MouseCursor::ResizeVertical
                                    });
                            }
                            let dim =
                                if $axis_index == 0 {
                                    [divider_width, kid_area.$outer()]
                                } else {
                                    [kid_area.$outer(), divider_width]
                                };
                            let prev_split_id = splits[i].0;
                            widget::Rectangle::fill(dim)
                                .color(divider_color)
                                .place_on_kid_area(false)
                                .$div_dir(prev_split_id, 0.5*(split_padding - divider_width))
                                .parent(id)
                                .set(divider_id, ui);
                        }
                    }
                }};
            }

            match direction {
                Direction::X(dir) => match dir {
                    Forwards => draw_splits!(w, h, right, right_from, mid_left, 0, 1.0),
                    Backwards => draw_splits!(w, h, left, left_from, mid_right, 0, -1.0),
                },

                Direction::Y(dir) => match dir {
                    Forwards => draw_splits!(h, w, up, up_from, mid_bottom, 1, 1.0),
                    Backwards => draw_splits!(h, w, down, down_from, mid_top, 1, -1.0),
                },
            }

        }
    }

}


/// The height and relative y coordinate of a PaneCanvas' title bar given some canvas height and font
/// size for the title bar.
fn title_bar_h_rel_y(canvas_h: Scalar, font_size: FontSize) -> (Scalar, Scalar) {
    let h = widget::title_bar::calc_height(font_size);
    let rel_y = canvas_h / 2.0 - h / 2.0;
    (h, rel_y)
}

/// The Rect for the PaneCanvas' title bar.
fn title_bar(canvas: Rect, font_size: FontSize) -> Rect {
    let (c_w, c_h) = canvas.w_h();
    let (h, rel_y) = title_bar_h_rel_y(c_h, font_size);
    let xy = [0.0, rel_y];
    let dim = [c_w, h];
    Rect::from_xy_dim(xy, dim)
}

impl Style {
    /// Get the Padding for the PaneCanvas' kid area.
    pub fn padding(&self, theme: &Theme) -> position::Padding {
        position::Padding {
            x: Range::new(self.pad_left(theme), self.pad_right(theme)),
            y: Range::new(self.pad_bottom(theme), self.pad_top(theme)),
        }
    }
}

impl<'a> ::color::Colorable for PaneCanvas<'a> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a> ::border::Borderable for PaneCanvas<'a> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a> ::label::Labelable<'a> for PaneCanvas<'a> {
    fn label(self, text: &'a str) -> Self {
        self.title_bar(text)
    }
    builder_methods!{
        label_color { style.title_bar_text_color = Some(Color) }
        label_font_size { style.title_bar_font_size = Some(FontSize) }
    }
}

