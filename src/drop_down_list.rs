use callback::Callable;
use frame::Frameable;
use label::{ FontSize, Labelable };
use color::{ Color, Colorable };
use dimensions::Dimensions;
use mouse::Mouse;
use point::Point;
use position::Positionable;
use rectangle;
use ui_context::{ UIID, UiContext };
use vecmath::vec2_add;
use graphics::Graphics;
use graphics::character::CharacterCache;
use widget::Widget;

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

widget_fns!(DropDownList, State, Widget::DropDownList(State::Closed(DrawState::Normal)));

/// Is the cursor currently over the widget? If so which item?
fn is_over(pos: Point,
           mouse_pos: Point,
           dim: Dimensions,
           state: State,
           len: Len) -> Option<Idx> {
    match state {
        State::Closed(_) => {
            match rectangle::is_over(pos, mouse_pos, dim) {
                false => None,
                true => Some(0),
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

/// A context on which the builder pattern can be implemented.
pub struct DropDownList<'a, F> {
    ui_id: UIID,
    strings: &'a mut Vec<String>,
    selected: &'a mut Option<Idx>,
    pos: Point,
    dim: Dimensions,
    maybe_callback: Option<F>,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_label: Option<&'a str>,
    maybe_label_color: Option<Color>,
    maybe_label_font_size: Option<u32>,
}

impl<'a, F> DropDownList<'a, F> {
    pub fn new(ui_id: UIID,
               strings: &'a mut Vec<String>,
               selected: &'a mut Option<Idx>) -> DropDownList<'a, F> {
        DropDownList {
            ui_id: ui_id,
            strings: strings,
            selected: selected,
            pos: [0.0, 0.0],
            dim: [128.0, 32.0],
            maybe_callback: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
        }
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

impl<'a, F> Callable<F> for DropDownList<'a, F> {
    fn callback(mut self, cb: F) -> Self {
        self.maybe_callback = Some(cb);
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
    fn point(mut self, pos: Point) -> Self {
        self.pos = pos;
        self
    }
}

/*
quack! {
    list: DropDownList['a, F]
    get:
        fn () -> Size [] { Size(list.dim) }
        fn () -> DefaultWidgetState [] {
            DefaultWidgetState(
                Widget::DropDownList(State::Closed(DrawState::Normal))
            )
        }
        fn () -> Id [] { Id(list.ui_id) }
    set:
        fn (val: Position) [] { list.pos = val.0 }
        fn (val: Size) [] { list.dim = val.0 }
    action:
}
*/

impl<'a, F> ::draw::Drawable for DropDownList<'a, F>
    where
        F: FnMut(&mut Option<Idx>, Idx, String) + 'a
{

    fn draw<B, C>(&mut self, uic: &mut UiContext<C>, graphics: &mut B)
        where
            B: Graphics<Texture = <C as CharacterCache>::Texture>,
            C: CharacterCache
    {

        let state = *get_state(uic, self.ui_id);
        let mouse = uic.get_mouse_state();
        let is_over_idx = is_over(self.pos, mouse.pos, self.dim, state, self.strings.len());
        let new_state = get_new_state(is_over_idx, self.strings.len(), state, mouse);

        let sel = match *self.selected {
            Some(idx) if idx < self.strings.len() => { Some(idx) },
            _ => None,
        };
        let color = self.maybe_color.unwrap_or(uic.theme.shape_color);
        let t_size = self.maybe_label_font_size.unwrap_or(uic.theme.font_size_medium);
        let t_color = self.maybe_label_color.unwrap_or(uic.theme.label_color);

        // Call the `callback` closure if mouse was released
        // on one of the DropDownMenu items.
        match (state, new_state) {
            (State::Open(o_d_state), State::Closed(c_d_state)) => {
                match (o_d_state, c_d_state) {
                    (DrawState::Clicked(idx, _), DrawState::Normal) => {
                        match self.maybe_callback {
                            Some(ref mut callback) => (*callback)(self.selected, idx, (*self.strings)[idx].clone()),
                            None => (),
                        }
                    }, _ => (),
                }
            }, _ => (),
        }

        let frame_w = self.maybe_frame.unwrap_or(uic.theme.frame_width);
        let maybe_frame = match frame_w > 0.0 {
            true => Some((frame_w, self.maybe_frame_color.unwrap_or(uic.theme.frame_color))),
            false => None,
        };

        match new_state {

            State::Closed(_) => {
                let rect_state = new_state.as_rect_state();
                let text = match sel {
                    Some(idx) => &(*self.strings)[idx][..],
                    None => match self.maybe_label {
                        Some(text) => text,
                        None => &(*self.strings)[0][..],
                    },
                };
                rectangle::draw_with_centered_label(
                    uic.win_w, uic.win_h, graphics, uic, rect_state,
                    self.pos, self.dim, maybe_frame, color,
                    text, t_size, t_color
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
                        self.dim, maybe_frame, color, &string,
                        t_size, t_color
                    )
                }
            },

        }

        set_state(uic, self.ui_id, Widget::DropDownList(new_state), self.pos, self.dim);

    }
}
