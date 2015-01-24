use quack::{ GetFrom, SetAt, Get, Set };
use color::Color;
use internal;
use mouse::Mouse;
use opengl_graphics::Gl;
use point::Point;
use rectangle;
use ui_context::{
    UIID,
    UiContext,
};
use vecmath::vec2_add;
use widget::Widget;
use Dimensions;
use Frame;
use FrameColor;
use Label;
use Position;
use Text;

/// Tuple / Callback params.
pub type Idx = usize;
pub type Len = usize;

/// Represents the state of the menu.
#[derive(PartialEq, Clone, Copy)]
pub enum State {
    Closed(DrawState),
    Open(DrawState),
}

/// Represents the state of the DropDownList widget.
#[derive(PartialEq, Clone, Copy)]
pub enum DrawState {
    Normal,
    Highlighted(Idx, Len),
    Clicked(Idx, Len),
}

impl DrawState {
    /// Translate the DropDownList's DrawState to the equivalent rectangle::State.
    fn as_rect_state(&self) -> rectangle::State {
        match self {
            &DrawState::Normal => rectangle::State::Normal,
            &DrawState::Highlighted(_, _) => rectangle::State::Highlighted,
            &DrawState::Clicked(_, _) => rectangle::State::Clicked,
        }
    }
}

impl State {
    /// Translate the DropDownList's State to the equivalent rectangle::State.
    fn as_rect_state(&self) -> rectangle::State {
        match self {
            &State::Open(draw_state) | &State::Closed(draw_state) => draw_state.as_rect_state(),
        }
    }
}

widget_fns!(DropDownList, State,
    Widget::DropDownList(State::Closed(DrawState::Normal)));

/// Is the cursor currently over the
fn is_over(pos: Point,
           mouse_pos: Point,
           dim: internal::Dimensions,
           state: State,
           len: Len) -> Option<Idx> {
    match state {
        State::Closed(_) => {
            match rectangle::is_over(pos, mouse_pos, dim) {
                false => None,
                true => Some(0us),
            }
        },
        State::Open(_) => {
            let total_h = dim[1] * len as f64;
            match rectangle::is_over(pos, mouse_pos, [dim[0], total_h]) {
                false => None,
                true => Some((((mouse_pos[1] - pos[1]) / total_h) * len as f64) as usize),
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
                        (Highlighted(_, _), Up)   => State::Closed(Highlighted(0us, len)),
                        (Highlighted(_, _), Down) => State::Closed(Clicked(0us, len)),
                        (Clicked(_, _),     Down) => State::Closed(Clicked(0us, len)),
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

///////////////////////////////// NEW DESIGN ///////////////////////////////////

pub struct DropDownList<'a> {
    strings: &'a mut Vec<String>,
    selected: &'a mut Option<Idx>,
    pos: Point,
    dim: internal::Dimensions,
    maybe_color: Option<internal::Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<internal::Color>,
    maybe_label: Option<Label<'a>>,
}

impl<'a> DropDownList<'a> {
    /// Creates a new drop down list.
    pub fn new(
        strings: &'a mut Vec<String>,
        selected: &'a mut Option<Idx>
    ) -> Self {
        DropDownList {
            strings: strings,
            selected: selected,
            pos: [0.0, 0.0],
            dim: [128.0, 32.0],
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label: None,
        }
    }

    /// Draws drop down list.
    pub fn draw(
        &mut self,
        ui_id: UIID,
        mut maybe_callback: Option<Box<FnMut(&mut Option<Idx>, Idx, String) + 'a>>,
        uic: &mut UiContext,
        graphics: &mut Gl
    ) {

        let state = *get_state(uic, ui_id);
        let mouse = uic.get_mouse_state();
        let is_over_idx = is_over(self.pos, mouse.pos, self.dim, state, self.strings.len());
        let new_state = get_new_state(is_over_idx, self.strings.len(), state, mouse);

        let sel = match *self.selected {
            Some(idx) if idx < self.strings.len() => { Some(idx) },
            _ => None,
        };
        let color = self.maybe_color.unwrap_or(uic.theme.shape_color.0);
        let t_size = self.maybe_label.map(|x| x.font_size(&uic.theme))
            .unwrap_or(uic.theme.font_size_medium);
        let t_color = self.maybe_label.map(|x| x.color(&uic.theme))
            .unwrap_or(uic.theme.label_color.0);

        // Call the `callback` closure if mouse was released
        // on one of the DropDownMenu items.
        match (state, new_state) {
            (State::Open(o_d_state), State::Closed(c_d_state)) => {
                match (o_d_state, c_d_state) {
                    (DrawState::Clicked(idx, _), DrawState::Normal) => {
                        match maybe_callback {
                            Some(ref mut callback) => (*callback)(
                                self.selected,
                                idx,
                                (*self.strings)[idx].clone()
                            ),
                            None => (),
                        }
                    }, _ => (),
                }
            }, _ => (),
        }

        let frame_w = self.maybe_frame.unwrap_or(uic.theme.frame_width);
        let maybe_frame = match frame_w > 0.0 {
            true => Some((frame_w, self.maybe_frame_color
                    .map(|x| Color(x))
                    .unwrap_or(uic.theme.frame_color))),
            false => None,
        };

        match new_state {

            State::Closed(_) => {
                let rect_state = new_state.as_rect_state();
                let text = match sel {
                    Some(idx) => &(*self.strings)[idx][],
                    None => match self.maybe_label {
                        Some(label) => {
                            let Text(text) = label.get();
                            text
                        },
                        None => &(*self.strings)[0][],
                    },
                };
                rectangle::draw_with_centered_label(
                    uic.win_w, uic.win_h, graphics, uic, rect_state,
                    self.pos, self.dim, maybe_frame, Color(color),
                    text, t_size, Color(t_color)
                )
            },

            State::Open(draw_state) => {
                for (i, string) in self.strings.iter().enumerate() {
                    let rect_state = match sel {
                        None => {
                            match draw_state {
                                DrawState::Normal => rectangle::State::Normal,
                                DrawState::Highlighted(idx, _) => {
                                    if i == idx { rectangle::State::Highlighted }
                                    else { rectangle::State::Normal }
                                },
                                DrawState::Clicked(idx, _) => {
                                    if i == idx { rectangle::State::Clicked }
                                    else { rectangle::State::Normal }
                                },
                            }
                        },
                        Some(sel_idx) => {
                            if sel_idx == i { rectangle::State::Clicked }
                            else {
                                match draw_state {
                                    DrawState::Normal => rectangle::State::Normal,
                                    DrawState::Highlighted(idx, _) => {
                                        if i == idx { rectangle::State::Highlighted }
                                        else { rectangle::State::Normal }
                                    },
                                    DrawState::Clicked(idx, _) => {
                                        if i == idx { rectangle::State::Clicked }
                                        else { rectangle::State::Normal }
                                    },
                                }
                            }
                        },
                    };
                    let idx_y = self.dim[1] * i as f64 - i as f64 * frame_w;
                    let idx_pos = vec2_add(self.pos, [0.0, idx_y]);
                    rectangle::draw_with_centered_label(
                        uic.win_w, uic.win_h, graphics, uic, rect_state, idx_pos,
                        self.dim, maybe_frame, Color(color), string.as_slice(),
                        t_size, Color(t_color)
                    )
                }
            },

        }

        set_state(uic, ui_id, new_state, self.pos, self.dim);

    }
}

impl<'a> GetFrom for (Position, DropDownList<'a>) {
    type Property = Position;
    type Object = DropDownList<'a>;

    fn get_from(drop_down_list: &DropDownList<'a>) -> Position {
        Position(drop_down_list.pos)
    }
}

impl<'a> SetAt for (Position, DropDownList<'a>) {
    type Property = Position;
    type Object = DropDownList<'a>;

    fn set_at(Position(pos): Position, drop_down_list: &mut DropDownList<'a>) {
        drop_down_list.pos = pos;
    }
}

impl<'a> GetFrom for (Dimensions, DropDownList<'a>) {
    type Property = Dimensions;
    type Object = DropDownList<'a>;

    fn get_from(drop_down_list: &DropDownList<'a>) -> Dimensions {
        Dimensions(drop_down_list.dim)
    }
}

impl<'a> SetAt for (Dimensions, DropDownList<'a>) {
    type Property = Dimensions;
    type Object = DropDownList<'a>;

    fn set_at(
        Dimensions(dim): Dimensions,
        drop_down_list: &mut DropDownList<'a>
    ) {
        drop_down_list.dim = dim;
    }
}

impl<'a> SetAt for (Color, DropDownList<'a>) {
    type Property = Color;
    type Object = DropDownList<'a>;

    fn set_at(Color(color): Color, drop_down_list: &mut DropDownList<'a>) {
        drop_down_list.maybe_color = Some(color);
    }
}

impl<'a> SetAt for (Text<'a>, DropDownList<'a>) {
    type Property = Text<'a>;
    type Object = DropDownList<'a>;

    fn set_at(Text(text): Text<'a>, drop_down_list: &mut DropDownList<'a>) {
        drop_down_list.maybe_label = match drop_down_list.maybe_label {
            None => Some(Label::new(text)),
            Some(x) => Some(x.set(Text(text)))
        };
    }
}

impl<'a> SetAt for (Label<'a>, DropDownList<'a>) {
    type Property = Label<'a>;
    type Object = DropDownList<'a>;

    fn set_at(label: Label<'a>, drop_down_list: &mut DropDownList<'a>) {
        drop_down_list.maybe_label = Some(label);
    }
}

impl<'a> SetAt for (Frame, DropDownList<'a>) {
    type Property = Frame;
    type Object = DropDownList<'a>;

    fn set_at(Frame(frame): Frame, drop_down_list: &mut DropDownList<'a>) {
        drop_down_list.maybe_frame = Some(frame);
    }
}

impl<'a> SetAt for (FrameColor, DropDownList<'a>) {
    type Property = FrameColor;
    type Object = DropDownList<'a>;

    fn set_at(
        FrameColor(color): FrameColor,
        drop_down_list: &mut DropDownList<'a>
    ) {
        drop_down_list.maybe_frame_color = Some(color);
    }
}
