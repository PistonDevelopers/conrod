
use color::Color;
use frame::{
    Framing,
    Frame,
    NoFrame,
};
use label;
use label::{
    Labeling,
    NoLabel,
    Label,
};
use mouse_state::{
    MouseState,
    Up,
    Down,
};
use opengl_graphics::Gl;
use point::Point;
use rectangle;
use ui_context::{
    UIID,
    UIContext,
};
use widget::DropDownList;

/// Tuple / Callback params.
pub type Idx = uint;
pub type Len = uint;

/// Represents the state of the menu.
#[deriving(PartialEq)]
pub enum State {
    Closed(DrawState),
    Open(DrawState),
}

/// Represents the state of the DropDownList widget.
#[deriving(PartialEq)]
pub enum DrawState {
    Normal,
    Highlighted(Idx, Len),
    Clicked(Idx, Len),
}

impl DrawState {
    /// Translate the DropDownList's DrawState to the equivalent rectangle::State.
    fn as_rect_state(&self) -> rectangle::State {
        match self {
            &Normal => rectangle::Normal,
            &Highlighted(_, _) => rectangle::Highlighted,
            &Clicked(_, _) => rectangle::Clicked,
        }
    }
}

impl State {
    /// Translate the DropDownList's State to the equivalent rectangle::State.
    fn as_rect_state(&self) -> rectangle::State {
        match self {
            &Open(draw_state) | &Closed(draw_state) => draw_state.as_rect_state(),
        }
    }
}

widget_fns!(DropDownList, State, DropDownList(Closed(Normal)))

/// Draw a dropdownlist generated from the given &Vec<String>.
pub fn draw(gl: &mut Gl,
            uic: &mut UIContext,
            ui_id: UIID,
            pos: Point<f64>,
            width: f64,
            height: f64,
            frame: Framing,
            color: Color,
            label: Labeling,
            strings: &mut Vec<String>,
            selected: &mut Option<Idx>,
            callback: |&mut Option<Idx>, Idx, String|) {

    let len = strings.len();
    if len == 0u { return }
    let sel = match *selected {
        Some(idx) => if idx >= len { None } else { Some(idx) },
        None => *selected,
    };
    let state = *get_state(uic, ui_id);
    let mouse = uic.get_mouse_state();
    let is_over_idx = is_over(pos, mouse.pos, width, height, state, len);
    let new_state = get_new_state(is_over_idx, len, state, mouse);
    let frame_width = match frame { Frame(w, _) => w, NoFrame => 0f64 };

    // Draw the DropDownList.
    match new_state {

        Closed(_) => {
            let rect_state = new_state.as_rect_state();
            let (t_size, t_color) = match label {
                Label(_, t_size, t_color) => (t_size, t_color),
                NoLabel => (label::auto_size_from_rect_height(height),
                            color.plain_contrast()),
            };
            let text = match sel {
                Some(idx) => (*strings)[idx].as_slice(),
                None => match label {
                    Label(text, _, _) => text,
                    NoLabel => (*strings)[0u].as_slice(),
                },
            };
            rectangle::draw_with_centered_label(uic.win_w, uic.win_h, gl, uic, 
                                                rect_state, pos, width, height,
                                                frame, color, text, t_size, t_color)
        },

        Open(draw_state) => {
            let (t_size, t_color) = match label {
                Label(_, t_size, t_color) => (t_size, t_color),
                NoLabel => (label::auto_size_from_rect_height(height),
                            color.plain_contrast()),
            };
            for (i, string) in strings.iter().enumerate() {
                let rect_state = match sel {
                    None => {
                        match draw_state {
                            Normal => rectangle::Normal,
                            Highlighted(idx, _) => if i == idx { rectangle::Highlighted }
                                                   else { rectangle::Normal },
                            Clicked(idx, _) => if i == idx { rectangle::Clicked }
                                                  else { rectangle::Normal },
                        }
                    },
                    Some(sel_idx) => {
                        match sel_idx == i {
                            true => rectangle::Clicked,
                            false => {
                                match draw_state {
                                    Normal => rectangle::Normal,
                                    Highlighted(idx, _) => if i == idx { rectangle::Highlighted }
                                                           else { rectangle::Normal },
                                    Clicked(idx, _) => if i == idx { rectangle::Clicked }
                                                          else { rectangle::Normal },
                                }
                            },
                        }
                    },
                };
                let idx_pos = pos + Point::new(0.0, height * i as f64 - i as f64 * frame_width, 0.0);
                rectangle::draw_with_centered_label(uic.win_w, uic.win_h, gl, uic, rect_state,
                                                    idx_pos, width, height, frame, color,
                                                    string.as_slice(), t_size, t_color)
            }
        },

    }

    set_state(uic, ui_id, new_state);

    // Call the `callback` closure if mouse was released
    // on one of the DropDownMenu items.
    match (state, new_state) {
        (Open(o_d_state), Closed(c_d_state)) => {
            match (o_d_state, c_d_state) {
                (Clicked(idx, _), Normal) => {
                    callback(selected, idx, (*strings)[idx].clone())
                },
                _ => (),
            }
        },
        _ => (),
    }

}

/// Is the cursor currently over the 
fn is_over(pos: Point<f64>,
           mouse_pos: Point<f64>,
           width: f64,
           height: f64,
           state: State,
           len: Len) -> Option<Idx> {
    match state {
        Closed(_) => {
            match rectangle::is_over(pos, mouse_pos, width, height) {
                false => None,
                true => Some(0u),
            }
        },
        Open(_) => {
            let total_h = height * len as f64;
            match rectangle::is_over(pos, mouse_pos, width, total_h) {
                false => None,
                true => Some((((mouse_pos.y - pos.y) / total_h) * len as f64) as uint),
            }
        },
    }
}

/// Determine and return the new State by comparing the mouse state
/// and position to the previous State.
fn get_new_state(is_over_idx: Option<Idx>,
                 len: Len,
                 state: State,
                 mouse: MouseState) -> State {
    match state {
        Closed(draw_state) => {
            match is_over_idx {
                Some(_) => {
                    match (draw_state, mouse.left) {
                        (Normal, Down) => Closed(Normal),
                        (Normal, Up) | (Highlighted(_, _), Up) => Closed(Highlighted(0u, len)),
                        (Highlighted(_, _), Down) => Closed(Clicked(0u, len)),
                        (Clicked(_, _), Down) => Closed(Clicked(0u, len)),
                        (Clicked(_, _), Up) => Open(Normal),
                    }
                },
                None => Closed(Normal),
            }
        },
        Open(draw_state) => {
            match is_over_idx {
                Some(idx) => {
                    match (draw_state, mouse.left) {
                        (Normal, Down) => Open(Normal),
                        (Normal, Up) | (Highlighted(_, _), Up) => Open(Highlighted(idx, len)),
                        (Highlighted(_, _), Down) => Open(Clicked(idx, len)),
                        (Clicked(p_idx, _), Down) => Open(Clicked(p_idx, len)),
                        (Clicked(_, _), Up) => Closed(Normal),
                    }
                },
                None => {
                    match (draw_state, mouse.left) {
                        (Highlighted(p_idx, _), Up) => Open(Highlighted(p_idx, len)),
                        _ => Closed(Normal),
                    }
                },
            }
        }
    }
}

/// A context on which the builder pattern can be implemented.
pub struct DropDownListContext<'a> {
    uic: &'a mut UIContext,
    strings: &mut Vec<String>,
    selected: &mut Option<Idx>,
    state: State,
    new_state: State,
    is_over_idx: Option<Idx>,
    pos: Point<f64>,
    width: f64,
    height: f64,
    maybe_color: Option<Color>,
    maybe_frame: Option<(f64, Color)>,
    maybe_label: Option<(&'a str, u32, Color)>,
}

pub trait DropDownListBuilder<'a> {
    /// A dropdownlist builder method to be implemented by the UIContext.
    fn drop_down_list(&'a mut self, ui_id: UIID,
                      strings: &'a mut Vec<String>, selected: &'a mut Option<Idx>,
                      x: f64, y: f64, width: f64, height: f64) -> DropDownListContext<'a>;
}

impl<'a> DropDownListBuilder<'a> for UIContext {
    fn drop_down_list(&'a mut self, ui_id: UIID,
                      strings: &'a mut Vec<String>, selected: &'a mut Option<Idx>,
                      x: f64, y: f64, width: f64, height: f64) -> DropDownListContext<'a> {
        if strings.len() == 0u { return; }
        let pos = Point::new(x, y, 0.0);
        let state = *get_state(self, ui_id);
        let mouse = self.get_mouse_state();
        let is_over_idx = is_over(pos, mouse.pos, width, height, state, strings.len());
        let new_state = get_new_state(is_over_idx, len, state, mouse);
        set_state(self, ui_id, new_state);
        DropDownListContext {
            uic: self,
            strings: strings,
            selected: selected,
            state: state,
            new_state: new_state,
            is_over_idx: is_over_idx,
            pos: pos,
            width: f64,
            height: f64,
            maybe_color: None,
            maybe_frame: None,
            maybe_label: None,
        }
    }
}

impl_colorable!(DropDownListContext)
impl_frameable!(DropDownListContext)
impl_labelable!(DropDownListContext)
impl_positionable!(DropDownListContext)

impl<'a> ::callback::Callable<||:'a> for DropDownListContext<'a> {
    #[inline]
    fn callback(self, callback: |&mut Option<Idx>, Idx, String|) -> DropDownListContext<'a> {
        // Call the `callback` closure if mouse was released
        // on one of the DropDownMenu items.
        match (self.state, self.new_state) {
            (Open(o_d_state), Closed(c_d_state)) => {
                match (o_d_state, c_d_state) {
                    (Clicked(idx, _), Normal) => {
                        callback(self.selected, idx, (*self.strings)[idx].clone())
                    }, _ => (),
                }
            }, _ => (),
        }
    }
}

impl<'a> ::draw::Drawable for DropDownListContext<'a> {
    fn draw(&mut self, gl: &mut Gl) {

        let sel = match *selected {
            Some(idx) if idx < strings.len() => { Some(idx) },
            None => None,
        };

        match new_state {

            Closed(_) => {
                let rect_state = self.new_state.as_rect_state();
                let(t_size, t_color) = match label {
                    Label(_, t_size, t_color) => (t_size, t_color),
                    NoLabel => (label::auto_size_from_rect_height(height),
                                color.plain_contrast()),
                };
                let text = match sel {
                    Some(idx) => (*self.strings)[idx].as_slice(),
                    None => match label {
                        Label(text, _, _) => text,
                        NoLabel => (*self.strings)[0u].as_slice(),
                    },
                };
                rectangle::draw_with_centered_label(
                    uic.win_w, uic.win_h, gl, uic, rect_state, pos,
                    width, height, frame, color, text, t_size, t_color
                )
            },

            Open(draw_state) => {
                let (t_size, t_color) = match label {
                    Label(_, t_size, t_color) => (t_size, t_color),
                    NoLabel => (label::auto_size_from_rect_height(height),
                                color.plain_contrast()),
                };
                for (i, string) in strings.iter().enumerate() {
                    let rect_state = match sel {
                        None => {
                            match draw_state {
                                Normal => rectangle::Normal,
                                Highlighted(idx, _) => {
                                    if i == idx { rectangle::Highlighted }
                                    else { rectangle::Normal }
                                },
                                Clicked(idx, _) => {
                                    if i == idx { rectangle::Clicked }
                                    else { rectangle::Normal }
                                },
                            }
                        },
                        Some(sel_idx) => {
                            if sel_idx == i { rectangle::Clicked }
                            else {
                                match draw_state {
                                    Normal => rectangle::Normal,
                                    Highlighted(idx, _) => {
                                        if i == idx { rectangle::Highlighted }
                                        else { rectangle::Normal }
                                    },
                                    Clicked(idx, _) => {
                                        if i == idx { rectangle::Clicked }
                                        else { rectangle::Normal }
                                    },
                                }
                            }
                        },
                    };
                    let frame_width = match frame { Frame(w, _) => w, NoFrame => 0f64 };
                    let idx_y = height * i as f64 - i as f64 * frame_width;
                    let idx_pos = pos + Point::new(0.0, idx_y, 0.0);
                    rectangle::draw_with_centered_label(
                        self.uic.win_w, self.uic.win_h, gl, self.uic, rect_state, idx_pos,
                        self.width, self.height, self.maybe_frame, color, string.as_slice(), 
                        t_size, t_color
                    )
                }
            },

        }
    }
}

