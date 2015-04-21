
use color::{Color, Colorable};
use frame::Frameable;
use graphics::character::CharacterCache;
use label::{FontSize, Labelable};
use mouse::Mouse;
use position::{Depth, Dimensions, HorizontalAlign, Point, Position, Positionable, VerticalAlign};
use ui::{UiId, Ui};
use widget::Kind;

/// Tuple / Callback params.
pub type Idx = usize;
pub type Len = usize;

/// Represents the state of the menu.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum State {
    Closed(DrawState),
    Open(DrawState),
}

/// Represents the state of the DropDownList widget.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DrawState {
    Normal,
    Highlighted(Idx, Len),
    Clicked(Idx, Len),
}

impl DrawState {
    fn color(&self, color: Color) -> Color {
        match *self {
            DrawState::Normal => color,
            DrawState::Highlighted(_, _) => color.highlighted(),
            DrawState::Clicked(_, _) => color.clicked(),
        }
    }
}

widget_fns!(DropDownList, State, Kind::DropDownList(State::Closed(DrawState::Normal)));

/// Is the cursor currently over the widget? If so which item?
fn is_over(mouse_pos: Point,
           frame_w: f64,
           dim: Dimensions,
           state: State,
           len: Len) -> Option<Idx> {
    use utils::is_over_rect;
    match state {
        State::Closed(_) => match is_over_rect([0.0, 0.0], mouse_pos, dim) {
            false => None,
            true => Some(0),
        },
        State::Open(_) => {
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
fn get_new_state(is_over_idx: Option<Idx>,
                 len: Len,
                 state: State,
                 mouse: Mouse) -> State {
    use self::DrawState::{Normal, Clicked, Highlighted};
    use mouse::ButtonState::{Down, Up};
    match state {
        State::Closed(draw_state) => {
            match is_over_idx {
                Some(_) => {
                    match (draw_state, mouse.left) {
                        (Normal,            Down) => State::Closed(Normal),
                        (Normal,            Up)   |
                        (Highlighted(_, _), Up)   => State::Closed(Highlighted(0, len)),
                        (Highlighted(_, _), Down) => State::Closed(Clicked(0, len)),
                        (Clicked(_, _),     Down) => State::Closed(Clicked(0, len)),
                        (Clicked(_, _),     Up)   => State::Open(Normal),
                    }
                },
                None => State::Closed(Normal),
            }
        },
        State::Open(draw_state) => {
            match is_over_idx {
                Some(idx) => {
                    match (draw_state, mouse.left) {
                        (Normal,            Down) => State::Open(Normal),
                        (Normal,            Up)   |
                        (Highlighted(_, _), Up)   => State::Open(Highlighted(idx, len)),
                        (Highlighted(_, _), Down) => State::Open(Clicked(idx, len)),
                        (Clicked(p_idx, _), Down) => State::Open(Clicked(p_idx, len)),
                        (Clicked(_, _),     Up)   => State::Closed(Normal),
                    }
                },
                None => {
                    match (draw_state, mouse.left) {
                        (Highlighted(p_idx, _), Up) => State::Open(Highlighted(p_idx, len)),
                        _ => State::Closed(Normal),
                    }
                },
            }
        }
    }
}

/// Displays a given `Vec<String>` as a selectable drop down menu. It's callback is triggered upon
/// selection of a list item.
pub struct DropDownList<'a, F> {
    strings: &'a mut Vec<String>,
    selected: &'a mut Option<Idx>,
    pos: Position,
    dim: Dimensions,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    depth: Depth,
    maybe_callback: Option<F>,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_label: Option<&'a str>,
    maybe_label_color: Option<Color>,
    maybe_label_font_size: Option<u32>,
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
            maybe_callback: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
        }
    }

    /// Set the DropDownList's callback. It will be triggered upon selection of a list item.
    pub fn callback(mut self, cb: F) -> DropDownList<'a, F> {
        self.maybe_callback = Some(cb);
        self
    }

    /// After building the DropDownList, use this method to set its current state into the given
    /// `Ui`. It will use this state for rendering the next time `ui.draw(graphics)` is called.
    pub fn set<C>(mut self, ui_id: UiId, ui: &mut Ui<C>)
        where
            C: CharacterCache,
            F: FnMut(&mut Option<Idx>, Idx, String),
    {
        use elmesque::form::{collage, rect, text};
        use elmesque::text::Text;

        let state = *get_state(ui, ui_id);
        let dim = self.dim;
        let h_align = self.maybe_h_align.unwrap_or(ui.theme.h_align);
        let v_align = self.maybe_v_align.unwrap_or(ui.theme.v_align);
        let xy = ui.get_xy(self.pos, dim, h_align, v_align);
        let mouse = ui.get_mouse_state().relative_to(xy);
        let frame_w = self.maybe_frame.unwrap_or(ui.theme.frame_width);
        let is_over_idx = is_over(mouse.xy, frame_w, dim, state, self.strings.len());
        let new_state = get_new_state(is_over_idx, self.strings.len(), state, mouse);
        let selected = self.selected.and_then(|idx| if idx < self.strings.len() { Some(idx) }
                                                    else { None });

        // Call the `callback` closure if mouse was released on one of the DropDownMenu items.
        if let Some(ref mut callback) = self.maybe_callback {
            if let (State::Open(o_d_state), State::Closed(c_d_state)) = (state, new_state) {
                if let (DrawState::Clicked(idx, _), DrawState::Normal) = (o_d_state, c_d_state) {
                    *self.selected = selected;
                    callback(self.selected, idx, self.strings[idx].clone())
                }
            }
        }

        // Get the DropDownList's styling.
        let color = self.maybe_color.unwrap_or(ui.theme.shape_color);
        let t_size = self.maybe_label_font_size.unwrap_or(ui.theme.font_size_medium);
        let t_color = self.maybe_label_color.unwrap_or(ui.theme.label_color);
        let pad_dim = ::vecmath::vec2_sub(dim, [frame_w * 2.0; 2]);
        let frame_color = self.maybe_frame_color.unwrap_or(ui.theme.frame_color);

        // Construct the DropDownList's Element.
        let element = match new_state {

            State::Closed(draw_state) => {
                let string = match selected {
                    Some(idx) => &(*self.strings)[idx][..],
                    None => match self.maybe_label {
                        Some(text) => text,
                        None => &(*self.strings)[0][..],
                    },
                }.to_string();
                let frame_form = rect(dim[0], dim[1]).filled(frame_color);
                let inner_form = rect(pad_dim[0], pad_dim[1]).filled(draw_state.color(color));
                let text_form = text(Text::from_string(string)
                                         .color(t_color)
                                         .height(t_size as f64));

                // Chain and shift the Forms into position.
                let form_chain = Some(frame_form).into_iter()
                    .chain(Some(inner_form).into_iter())
                    .chain(Some(text_form).into_iter())
                    .map(|form| form.shift(xy[0].floor(), xy[1].floor()));

                // Collect the Form's into a renderable Element.
                collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
            },

            State::Open(draw_state) => {
                // Chain and shift the Forms into position.
                let form_chain = self.strings.iter().enumerate().flat_map(|(i, string)| {
                    let color = match selected {
                        None => match draw_state {
                            DrawState::Normal => color,
                            DrawState::Highlighted(idx, _) => {
                                if i == idx { color.highlighted() }
                                else { color }
                            },
                            DrawState::Clicked(idx, _) => {
                                if i == idx { color.clicked() }
                                else { color }
                            },
                        },
                        Some(sel_idx) => {
                            if sel_idx == i { color.clicked() }
                            else {
                                match draw_state {
                                    DrawState::Normal => color,
                                    DrawState::Highlighted(idx, _) => {
                                        if i == idx { color.highlighted() }
                                        else { color }
                                    },
                                    DrawState::Clicked(idx, _) => {
                                        if i == idx { color.clicked() }
                                        else { color }
                                    },
                                }
                            }
                        },
                    };
                    let shift_amt = -(i as f64 * dim[1] - i as f64 * frame_w).floor();
                    let frame_form = rect(dim[0], dim[1]).filled(frame_color);
                    let inner_form = rect(pad_dim[0], pad_dim[1]).filled(color);
                    let text_form = text(Text::from_string(string.clone())
                                             .color(t_color)
                                             .height(t_size as f64));
                    Some(frame_form.shift_y(shift_amt)).into_iter()
                        .chain(Some(inner_form.shift_y(shift_amt)).into_iter())
                        .chain(Some(text_form.shift_y(shift_amt.floor())).into_iter())
                }).map(|form| form.shift(xy[0].floor(), xy[1].floor()));

                // Collect the Form's into a renderable Element.
                collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
            },

        };

        // Store the drop down list's new state in the Ui.
        ui.set_widget(ui_id, ::widget::Widget {
            kind: Kind::DropDownList(new_state),
            xy: xy,
            depth: self.depth,
            element: Some(element),
        });

    }

}

impl<'a, F> Colorable for DropDownList<'a, F> {
    fn color(mut self, color: Color) -> Self {
        self.maybe_color = Some(color);
        self
    }
}

impl<'a, F> Frameable for DropDownList<'a, F> {
    fn frame(mut self, width: f64) -> Self {
        self.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, F> Labelable<'a> for DropDownList<'a, F>
{
    fn label(mut self, text: &'a str) -> Self {
        self.maybe_label = Some(text);
        self
    }

    fn label_color(mut self, color: Color) -> Self {
        self.maybe_label_color = Some(color);
        self
    }

    fn label_font_size(mut self, size: FontSize) -> Self {
        self.maybe_label_font_size = Some(size);
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

