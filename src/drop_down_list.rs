
use color::Color;
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
#[deriving(PartialEq, Clone)]
pub enum State {
    Closed(DrawState),
    Open(DrawState),
}

/// Represents the state of the DropDownList widget.
#[deriving(PartialEq, Clone)]
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
    ui_id: UIID,
    strings: &'a mut Vec<String>,
    selected: &'a mut Option<Idx>,
    pos: Point<f64>,
    width: f64,
    height: f64,
    maybe_callback: Option<|&mut Option<Idx>, Idx, String|:'a>,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_label: Option<&'a str>,
    maybe_label_color: Option<Color>,
    maybe_label_font_size: Option<u32>,
}

pub trait DropDownListBuilder<'a> {
    /// A dropdownlist builder method to be implemented by the UIContext.
    fn drop_down_list(&'a mut self, ui_id: UIID, strings: &'a mut Vec<String>,
                      selected: &'a mut Option<Idx>) -> DropDownListContext<'a>;
}

impl<'a> DropDownListBuilder<'a> for UIContext {
    fn drop_down_list(&'a mut self, ui_id: UIID, strings: &'a mut Vec<String>,
                      selected: &'a mut Option<Idx>) -> DropDownListContext<'a> {
        DropDownListContext {
            uic: self,
            ui_id: ui_id,
            strings: strings,
            selected: selected,
            pos: Point::new(0.0, 0.0, 0.0),
            width: 128.0,
            height: 32.0,
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

impl_callable!(DropDownListContext, |&mut Option<Idx>, Idx, String|:'a)
impl_colorable!(DropDownListContext)
impl_frameable!(DropDownListContext)
impl_labelable!(DropDownListContext)
impl_positionable!(DropDownListContext)
impl_shapeable!(DropDownListContext)

impl<'a> ::draw::Drawable for DropDownListContext<'a> {
    fn draw(&mut self, gl: &mut Gl) {

        let state = *get_state(self.uic, self.ui_id);
        let mouse = self.uic.get_mouse_state();
        let is_over_idx = is_over(self.pos, mouse.pos, self.width, self.height,
                                  state, self.strings.len());
        let new_state = get_new_state(is_over_idx, self.strings.len(), state, mouse);

        let sel = match *self.selected {
            Some(idx) if idx < self.strings.len() => { Some(idx) },
            _ => None,
        };
        let color = self.maybe_color.unwrap_or(self.uic.theme.shape_color);
        let t_size = self.maybe_label_font_size.unwrap_or(self.uic.theme.font_size_medium);
        let t_color = self.maybe_label_color.unwrap_or(self.uic.theme.label_color);

        // Call the `callback` closure if mouse was released
        // on one of the DropDownMenu items.
        match (state, new_state) {
            (Open(o_d_state), Closed(c_d_state)) => {
                match (o_d_state, c_d_state) {
                    (Clicked(idx, _), Normal) => {
                        match self.maybe_callback {
                            Some(ref mut callback) => (*callback)(self.selected, idx, (*self.strings)[idx].clone()),
                            None => (),
                        }
                    }, _ => (),
                }
            }, _ => (),
        }

        let frame_w = self.maybe_frame.unwrap_or(self.uic.theme.frame_width);
        let maybe_frame = match frame_w > 0.0 {
            true => Some((frame_w, self.maybe_frame_color.unwrap_or(self.uic.theme.frame_color))),
            false => None,
        };

        match new_state {

            Closed(_) => {
                let rect_state = new_state.as_rect_state();
                let text = match sel {
                    Some(idx) => (*self.strings)[idx][],
                    None => match self.maybe_label {
                        Some(text) => text,
                        None => (*self.strings)[0][],
                    },
                };
                rectangle::draw_with_centered_label(
                    self.uic.win_w, self.uic.win_h, gl, self.uic, rect_state,
                    self.pos, self.width, self.height, maybe_frame, color,
                    text, t_size, t_color
                )
            },

            Open(draw_state) => {
                for (i, string) in self.strings.iter().enumerate() {
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
                    let idx_y = self.height * i as f64 - i as f64 * frame_w;
                    let idx_pos = self.pos + Point::new(0.0, idx_y, 0.0);
                    rectangle::draw_with_centered_label(
                        self.uic.win_w, self.uic.win_h, gl, self.uic, rect_state, idx_pos,
                        self.width, self.height, maybe_frame, color, string.as_slice(), 
                        t_size, t_color
                    )
                }
            },

        }

        set_state(self.uic, self.ui_id, new_state,
                  self.pos.x, self.pos.y, self.width, self.height);

    }
}

