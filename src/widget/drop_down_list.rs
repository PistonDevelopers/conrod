
use canvas::CanvasId;
use color::{Color, Colorable};
use elmesque::Element;
use frame::Frameable;
use graphics::character::CharacterCache;
use label::{FontSize, Labelable};
use mouse::Mouse;
use position::{Depth, Dimensions, HorizontalAlign, Point, Position, Positionable, VerticalAlign};
use theme::Theme;
use ui::{UiId, Ui};
use widget::{self, Widget, WidgetId};


/// Tuple / React params.
pub type Idx = usize;
pub type Len = usize;

/// Displays a given `Vec<String>` as a selectable drop down menu. It's reaction is triggered upon
/// selection of a list item.
pub struct DropDownList<'a, F> {
    strings: &'a mut Vec<String>,
    selected: &'a mut Option<Idx>,
    pos: Position,
    dim: Dimensions,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    depth: Depth,
    maybe_react: Option<F>,
    maybe_label: Option<&'a str>,
    style: Style,
    enabled: bool,
    maybe_canvas_id: Option<CanvasId>,
}

/// Styling for the DropDownList, necessary for constructing its renderable Element.
#[derive(PartialEq, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Style {
    pub maybe_color: Option<Color>,
    pub maybe_frame: Option<f64>,
    pub maybe_frame_color: Option<Color>,
    pub maybe_label_color: Option<Color>,
    pub maybe_label_font_size: Option<u32>,
}

/// Represents the state of the DropDownList.
#[derive(PartialEq, Clone, Debug)]
pub struct State {
    menu_state: MenuState,
    maybe_label: Option<String>,
    strings: Vec<String>,
    maybe_selected: Option<Idx>,
}

/// Whether the DropDownList is currently open or closed.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MenuState {
    Closed(Interaction),
    Open(Interaction),
}

/// Describes how the DropDownList is currently being interacted with.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Interaction {
    Normal,
    Highlighted(Idx, Len),
    Clicked(Idx, Len),
}


impl Interaction {
    fn color(&self, color: Color) -> Color {
        match *self {
            Interaction::Normal => color,
            Interaction::Highlighted(_, _) => color.highlighted(),
            Interaction::Clicked(_, _) => color.clicked(),
        }
    }
}


/// Is the cursor currently over the widget? If so which item?
fn is_over(mouse_pos: Point,
           frame_w: f64,
           dim: Dimensions,
           menu_state: MenuState,
           len: Len) -> Option<Idx> {
    use utils::is_over_rect;
    match menu_state {
        MenuState::Closed(_) => match is_over_rect([0.0, 0.0], mouse_pos, dim) {
            false => None,
            true => Some(0),
        },
        MenuState::Open(_) => {
            let item_h = dim[1] - frame_w;
            let total_h = item_h * len as f64;
            let open_centre_y = -(total_h - item_h) / 2.0;
            match is_over_rect([0.0, open_centre_y], mouse_pos, [dim[0], total_h]) {
                false => None,
                true => Some(((mouse_pos[1] - item_h / 2.0).abs() / item_h) as usize),
            }
        },
    }
}


/// Determine and return the new State by comparing the mouse state
/// and position to the previous State.
fn get_new_menu_state(is_over_idx: Option<Idx>,
                      len: Len,
                      menu_state: MenuState,
                      mouse: Mouse) -> MenuState {
    use self::Interaction::{Normal, Clicked, Highlighted};
    use mouse::ButtonState::{Down, Up};
    match menu_state {
        MenuState::Closed(draw_state) => {
            match is_over_idx {
                Some(_) => {
                    match (draw_state, mouse.left) {
                        (Normal,            Down) => MenuState::Closed(Normal),
                        (Normal,            Up)   |
                        (Highlighted(_, _), Up)   => MenuState::Closed(Highlighted(0, len)),
                        (Highlighted(_, _), Down) => MenuState::Closed(Clicked(0, len)),
                        (Clicked(_, _),     Down) => MenuState::Closed(Clicked(0, len)),
                        (Clicked(_, _),     Up)   => MenuState::Open(Normal),
                    }
                },
                None => MenuState::Closed(Normal),
            }
        },
        MenuState::Open(draw_state) => {
            match is_over_idx {
                Some(idx) => {
                    match (draw_state, mouse.left) {
                        (Normal,            Down) => MenuState::Open(Normal),
                        (Normal,            Up)   |
                        (Highlighted(_, _), Up)   => MenuState::Open(Highlighted(idx, len)),
                        (Highlighted(_, _), Down) => MenuState::Open(Clicked(idx, len)),
                        (Clicked(p_idx, _), Down) => MenuState::Open(Clicked(p_idx, len)),
                        (Clicked(_, _),     Up)   => MenuState::Closed(Normal),
                    }
                },
                None => {
                    match (draw_state, mouse.left) {
                        (Highlighted(p_idx, _), Up) => MenuState::Open(Highlighted(p_idx, len)),
                        _ => MenuState::Closed(Normal),
                    }
                },
            }
        }
    }
}

impl<'a, F> DropDownList<'a, F> {

    /// Construct a new DropDownList.
    pub fn new(strings: &'a mut Vec<String>, selected: &'a mut Option<Idx>) -> DropDownList<'a, F> {
        DropDownList {
            strings: strings,
            selected: selected,
            pos: Position::default(),
            dim: [128.0, 32.0],
            maybe_h_align: None,
            maybe_v_align: None,
            depth: 0.0,
            maybe_react: None,
            maybe_label: None,
            style: Style::new(),
            enabled: true,
            maybe_canvas_id: None,
        }
    }

    /// Set the DropDownList's reaction. It will be triggered upon selection of a list item.
    pub fn react(mut self, reaction: F) -> DropDownList<'a, F> {
        self.maybe_react = Some(reaction);
        self
    }

    /// If true, will allow user inputs.  If false, will disallow user inputs.
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
    }

    /// Set which Canvas to attach the Widget to. Note that you can also attach a widget to a
    /// Canvas by using the canvas placement `Positionable` methods.
    pub fn canvas(mut self, id: CanvasId) -> Self {
        self.maybe_canvas_id = Some(id);
        self
    }

}


impl<'a, F> Widget for DropDownList<'a, F>
    where
        F: FnMut(&mut Option<Idx>, Idx, String),
{
    type State = State;
    type Style = Style;
    fn unique_kind(&self) -> &'static str { "DropDownList" }
    fn init_state(&self) -> State {
        State {
            menu_state: MenuState::Closed(Interaction::Normal),
            strings: Vec::new(),
            maybe_label: None,
            maybe_selected: None,
        }
    }
    fn style(&self) -> Style { self.style.clone() }

    /// Update the state of the DropDownList.
    fn update<C>(mut self,
                 prev_state: &widget::State<State>,
                 style: &Style,
                 id: WidgetId,
                 ui: &mut Ui<C>) -> widget::State<Option<State>>
        where
            C: CharacterCache,
    {

        let widget::State { ref state, .. } = *prev_state;
        let dim = self.dim;
        let h_align = self.maybe_h_align.unwrap_or(ui.theme.align.horizontal);
        let v_align = self.maybe_v_align.unwrap_or(ui.theme.align.vertical);
        let xy = ui.get_xy(self.pos, dim, h_align, v_align);
        let maybe_mouse = ui.get_mouse_state(UiId::Widget(id)).map(|m| m.relative_to(xy));
        let frame = style.frame(&ui.theme);
        let num_strings = self.strings.len();

        // Check for a new interaction with the DropDownList.
        let new_menu_state = match (self.enabled, maybe_mouse) {
            (false, _) | (true, None) => MenuState::Closed(Interaction::Normal),
            (true, Some(mouse)) => {
                let is_over_idx = is_over(mouse.xy, frame, dim, state.menu_state, num_strings);
                get_new_menu_state(is_over_idx, num_strings, state.menu_state, mouse)
            },
        };

        let selected = self.selected.and_then(|idx| if idx < num_strings { Some(idx) }
                                                    else { None });

        // Check whether or not we need to capture or uncapture the mouse.
        // We need to capture the cursor if the DropDownList has just been opened.
        // We need to uncapture the cursor if the DropDownList has just been closed.
        match (state.menu_state, new_menu_state) {
            (MenuState::Closed(_), MenuState::Open(_)) => ui.mouse_captured_by(UiId::Widget(id)),
            (MenuState::Open(_), MenuState::Closed(_)) => ui.mouse_uncaptured_by(UiId::Widget(id)),
            _ => (),
        }

        // Call the `react` closure if mouse was released on one of the DropDownList items.
        if let Some(ref mut react) = self.maybe_react {
            if let (MenuState::Open(o_d_state), MenuState::Closed(c_d_state)) =
                (state.menu_state, new_menu_state) {
                if let (Interaction::Clicked(idx, _), Interaction::Normal) = (o_d_state, c_d_state) {
                    *self.selected = selected;
                    react(self.selected, idx, self.strings[idx].clone())
                }
            }
        }

        // Function for constructing a new DropDownList State.
        let construct_new_state = || {
            State {
                menu_state: new_menu_state,
                maybe_label: self.maybe_label.as_ref().map(|label| label.to_string()),
                strings: self.strings.clone(),
                maybe_selected: *self.selected,
            }
        };

        // Check whether or not the state has changed since the previous update.
        let state_has_changed = state.menu_state != new_menu_state
            || &state.strings[..] != &(*self.strings)[..]
            || state.maybe_selected != *self.selected
            || state.maybe_label.as_ref().map(|string| &string[..]) != self.maybe_label;

        // Construct the new state if there was a change.
        let maybe_new_state = if state_has_changed { Some(construct_new_state()) }
                              else { None };

        // Retrieve the CanvasId.
        let maybe_canvas_id = self.maybe_canvas_id.or_else(|| {
            if let Position::Place(_, maybe_canvas_id) = self.pos { maybe_canvas_id } else { None }
        });

        widget::State {
            state: maybe_new_state,
            dim: dim,
            xy: xy,
            depth: self.depth,
            maybe_canvas_id: maybe_canvas_id,
        }
    }

    /// Construct an Element from the given DropDownList State.
    fn draw<C>(new_state: &widget::State<State>, style: &Style, ui: &mut Ui<C>) -> Element
        where
            C: CharacterCache,
    {
        use elmesque::form::{collage, rect, text};
        use elmesque::text::Text;

        let widget::State { ref state, dim, xy, .. } = *new_state;
        let theme = &ui.theme;

        // Retrieve the styling for the Element..
        let color = style.color(theme);
        let frame = style.frame(theme);
        let frame_color = style.frame_color(theme);
        let label_color = style.label_color(theme);
        let font_size = style.label_font_size(theme);
        let pad_dim = ::vecmath::vec2_sub(dim, [frame * 2.0; 2]);

        // Construct the DropDownList's Element.
        match state.menu_state {

            MenuState::Closed(draw_state) => {
                let string = match state.maybe_selected {
                    Some(idx) => state.strings[idx].clone(),
                    None => match state.maybe_label {
                        Some(ref label) => label.clone(),
                        None => state.strings[0].clone(),
                    },
                };
                let frame_form = rect(dim[0], dim[1]).filled(frame_color);
                let inner_form = rect(pad_dim[0], pad_dim[1]).filled(draw_state.color(color));
                let text_form = text(Text::from_string(string)
                                         .color(label_color)
                                         .height(font_size as f64));

                // Chain and shift the Forms into position.
                let form_chain = Some(frame_form).into_iter()
                    .chain(Some(inner_form).into_iter())
                    .chain(Some(text_form).into_iter())
                    .map(|form| form.shift(xy[0].floor(), xy[1].floor()));

                // Collect the Form's into a renderable Element.
                collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
            },

            MenuState::Open(draw_state) => {
                // Chain and shift the Forms into position.
                let form_chain = state.strings.iter().enumerate().flat_map(|(i, string)| {
                    let color = match state.maybe_selected {
                        None => match draw_state {
                            Interaction::Normal => color,
                            Interaction::Highlighted(idx, _) => {
                                if i == idx { color.highlighted() }
                                else { color }
                            },
                            Interaction::Clicked(idx, _) => {
                                if i == idx { color.clicked() }
                                else { color }
                            },
                        },
                        Some(sel_idx) => {
                            if sel_idx == i { color.clicked() }
                            else {
                                match draw_state {
                                    Interaction::Normal => color,
                                    Interaction::Highlighted(idx, _) => {
                                        if i == idx { color.highlighted() }
                                        else { color }
                                    },
                                    Interaction::Clicked(idx, _) => {
                                        if i == idx { color.clicked() }
                                        else { color }
                                    },
                                }
                            }
                        },
                    };
                    let shift_amt = -(i as f64 * dim[1] - i as f64 * frame).floor();
                    let frame_form = rect(dim[0], dim[1]).filled(frame_color);
                    let inner_form = rect(pad_dim[0], pad_dim[1]).filled(color);
                    let text_form = text(Text::from_string(string.clone())
                                             .color(label_color)
                                             .height(font_size as f64));
                    Some(frame_form.shift_y(shift_amt)).into_iter()
                        .chain(Some(inner_form.shift_y(shift_amt)).into_iter())
                        .chain(Some(text_form.shift_y(shift_amt.floor())).into_iter())
                }).map(|form| form.shift(xy[0].floor(), xy[1].floor()));

                // Collect the Form's into a renderable Element.
                collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
            },

        }

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
        self.maybe_color.or(theme.maybe_drop_down_list.as_ref().map(|style| {
            style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.maybe_drop_down_list.as_ref().map(|style| {
            style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.maybe_drop_down_list.as_ref().map(|style| {
            style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label Color for an Element.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.maybe_drop_down_list.as_ref().map(|style| {
            style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn label_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.maybe_drop_down_list.as_ref().map(|style| {
            style.maybe_label_font_size.unwrap_or(theme.font_size_medium)
        })).unwrap_or(theme.font_size_medium)
    }

}


impl<'a, F> Colorable for DropDownList<'a, F> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

impl<'a, F> Frameable for DropDownList<'a, F> {
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, F> Labelable<'a> for DropDownList<'a, F> {
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

impl<'a, F> Positionable for DropDownList<'a, F> {
    fn position(mut self, pos: Position) -> Self {
        self.pos = pos;
        self
    }
    #[inline]
    fn horizontal_align(self, h_align: HorizontalAlign) -> Self {
        DropDownList { maybe_h_align: Some(h_align), ..self }
    }
    #[inline]
    fn vertical_align(self, v_align: VerticalAlign) -> Self {
        DropDownList { maybe_v_align: Some(v_align), ..self }
    }
}

impl<'a, F> ::position::Sizeable for DropDownList<'a, F> {
    #[inline]
    fn width(self, w: f64) -> Self {
        let h = self.dim[1];
        DropDownList { dim: [w, h], ..self }
    }
    #[inline]
    fn height(self, h: f64) -> Self {
        let w = self.dim[0];
        DropDownList { dim: [w, h], ..self }
    }
}

