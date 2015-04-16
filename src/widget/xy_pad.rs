
use color::{Color, Colorable};
use dimensions::Dimensions;
use frame::Frameable;
use graphics::{self, Graphics};
use graphics::character::CharacterCache;
use label::{self, FontSize, Labelable};
use mouse::Mouse;
use num::{Float, ToPrimitive, FromPrimitive};
use point::Point;
use position::Positionable;
use rectangle::{self, Corner};
use shape::Shapeable;
use ui::{UiId, Ui};
use utils::{clamp, map_range, val_to_string};
use vecmath::{vec2_add, vec2_sub};
use widget::Kind;

/// Represents the state of the xy_pad widget.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum State {
    Normal,
    Highlighted,
    Clicked,
}

impl State {
    /// Return the associated Rectangle state.
    fn as_rectangle_state(&self) -> rectangle::State {
        match self {
            &State::Normal => rectangle::State::Normal,
            &State::Highlighted => rectangle::State::Highlighted,
            &State::Clicked => rectangle::State::Clicked,
        }
    }
}

widget_fns!(XYPad, State, Kind::XYPad(State::Normal));

/// Check the current state of the button.
fn get_new_state(is_over: bool,
                 prev: State,
                 mouse: Mouse) -> State {
    use mouse::ButtonState::{Down, Up};
    use self::State::{Normal, Highlighted, Clicked};
    match (is_over, prev, mouse.left) {
        (true,  Normal,  Down) => Normal,
        (true,  _,       Down) => Clicked,
        (true,  _,       Up)   => Highlighted,
        (false, Clicked, Down) => Clicked,
        _                      => Normal,
    }
}

/// Draw the crosshair.
fn draw_crosshair<B: Graphics>(
    win_w: f64,
    win_h: f64,
    graphics: &mut B,
    pos: Point,
    line_width: f64,
    vert_x: f64, hori_y: f64,
    pad_dim: Dimensions,
    color: Color
) {
    let draw_state = graphics::default_draw_state();
    let transform = graphics::abs_transform(win_w, win_h);
    let line = graphics::Line::new(color.to_fsa(), 0.5 * line_width);
    line.draw(
        [vert_x, pos[1], vert_x, pos[1] + pad_dim[1]],
        draw_state,
        transform,
        graphics
    );
    line.draw(
        [pos[0], hori_y, pos[0] + pad_dim[0], hori_y],
        draw_state,
        transform,
        graphics
    );
}


/// Used for displaying and controlling a 2D point on a cartesian plane within a given range.
/// Its callback is triggered when the value is updated or if the mouse button is released while
/// the cursor is above the rectangle.
pub struct XYPad<'a, X, Y, F> {
    ui_id: UiId,
    x: X, min_x: X, max_x: X,
    y: Y, min_y: Y, max_y: Y,
    line_width: f64,
    font_size: FontSize,
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

impl<'a, X, Y, F> XYPad<'a, X, Y, F> {

    /// Construct a new XYPad widget.
    pub fn new(ui_id: UiId,
              x_val: X, min_x: X, max_x: X,
              y_val: Y, min_y: Y, max_y: Y) -> XYPad<'a, X, Y, F> {
        XYPad {
            ui_id: ui_id,
            x: x_val, min_x: min_x, max_x: max_x,
            y: y_val, min_y: min_y, max_y: max_y,
            line_width: 1.0,
            font_size: 18u32,
            pos: [0.0, 0.0],
            dim: [128.0, 128.0],
            maybe_callback: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
        }
    }

    /// Set the width of the XYPad's crosshair lines.
    #[inline]
    pub fn line_width(self, width: f64) -> XYPad<'a, X, Y, F> {
        XYPad { line_width: width, ..self }
    }

    /// Set the font size for the displayed crosshair value.
    #[inline]
    pub fn value_font_size(self, size: FontSize) -> XYPad<'a, X, Y, F> {
        XYPad { font_size: size, ..self }
    }

    /// Set the callback for the XYPad. It will be triggered when the value is updated or if the
    /// mouse button is released while the cursor is above the rectangle.
    pub fn callback(mut self, cb: F) -> Self {
        self.maybe_callback = Some(cb);
        self
    }

}

impl<'a, X, Y, F> Colorable for XYPad<'a, X, Y, F> {
    fn color(mut self, color: Color) -> Self {
        self.maybe_color = Some(color);
        self
    }
}

impl<'a, X, Y, F> Frameable for XYPad<'a, X, Y, F> {
    fn frame(mut self, width: f64) -> Self {
        self.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.maybe_frame_color = Some(color);
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
        self.maybe_label_color = Some(color);
        self
    }

    fn label_font_size(mut self, size: FontSize) -> Self {
        self.maybe_label_font_size = Some(size);
        self
    }
}

impl<'a, X, Y, F> Positionable for XYPad<'a, X, Y, F> {
    fn point(mut self, pos: Point) -> Self {
        self.pos = pos;
        self
    }
}

impl<'a, X, Y, F> Shapeable for XYPad<'a, X, Y, F> {
    fn get_dim(&self) -> Dimensions { self.dim }
    fn dim(mut self, dim: Dimensions) -> Self { self.dim = dim; self }
}

impl<'a, X, Y, F> ::draw::Drawable for XYPad<'a, X, Y, F>
    where
        X: Float + ToPrimitive + FromPrimitive + ToString,
        Y: Float + ToPrimitive + FromPrimitive + ToString,
        F: FnMut(X, Y) + 'a
{

    fn draw<B, C>(&mut self, ui: &mut Ui<C>, graphics: &mut B)
        where
            B: Graphics<Texture = <C as CharacterCache>::Texture>,
            C: CharacterCache
    {

        // Init.
        let state = *get_state(ui, self.ui_id);
        let mouse = ui.get_mouse_state();
        let frame_w = self.maybe_frame.unwrap_or(ui.theme.frame_width);
        let frame_w2 = frame_w * 2.0;
        let maybe_frame = match frame_w > 0.0 {
            true => Some((frame_w, self.maybe_frame_color.unwrap_or(ui.theme.frame_color))),
            false => None,
        };
        let pad_dim = vec2_sub(self.dim, [frame_w2; 2]);
        let pad_pos = vec2_add(self.pos, [frame_w, frame_w]);
        let is_over_pad = rectangle::is_over(pad_pos, mouse.pos, pad_dim);
        let new_state = get_new_state(is_over_pad, state, mouse);

        // Determine new values.
        let (new_x, new_y) = match (is_over_pad, new_state) {
            (_, State::Normal) | (_, State::Highlighted) => (self.x, self.y),
            (_, State::Clicked) => {
                let temp_x = clamp(mouse.pos[0], pad_pos[0], pad_pos[0] + pad_dim[0]);
                let temp_y = clamp(mouse.pos[1], pad_pos[1], pad_pos[1] + pad_dim[1]);
                (map_range(temp_x - self.pos[0], pad_dim[0], 0.0, self.min_x, self.max_x),
                 map_range(temp_y - self.pos[1], pad_dim[1], 0.0, self.min_y, self.max_y))
            }
        };

        // Callback if value is changed or the pad is clicked/released.
        match self.maybe_callback {
            Some(ref mut callback) => {
                if self.x != new_x || self.y != new_y { (*callback)(new_x, new_y) }
                else {
                    match (state, new_state) {
                        (State::Highlighted, State::Clicked)
                        | (State::Clicked, State::Highlighted) => (*callback)(new_x, new_y),
                        _ => (),
                    }
                }
            },
            None => (),
        }

        // Draw.
        let rect_state = new_state.as_rectangle_state();
        let color = self.maybe_color.unwrap_or(ui.theme.shape_color);
        rectangle::draw(ui.win_w, ui.win_h, graphics, rect_state, self.pos,
                        self.dim, maybe_frame, color);
        let (vert_x, hori_y) = match (is_over_pad, new_state) {
            (_, State::Normal) | (_, State::Highlighted) =>
                (pad_pos[0] + map_range(new_x, self.min_x, self.max_x, pad_dim[0], 0.0),
                 pad_pos[1] + map_range(new_y, self.min_y, self.max_y, pad_dim[1], 0.0)),
            (_, State::Clicked) =>
                (clamp(mouse.pos[0], pad_pos[0], pad_pos[0] + pad_dim[0]),
                 clamp(mouse.pos[1], pad_pos[1], pad_pos[1] + pad_dim[1])),
        };
        // Crosshair.
        draw_crosshair(ui.win_w, ui.win_h, graphics, pad_pos, self.line_width,
                       vert_x, hori_y, pad_dim, color.plain_contrast());
        // Label.
        if let Some(l_text) = self.maybe_label {
            let l_color = self.maybe_label_color.unwrap_or(ui.theme.label_color);
            let l_size = self.maybe_label_font_size.unwrap_or(ui.theme.font_size_medium);
            let l_w = label::width(ui, l_size, l_text);
            let l_x = pad_pos[0] + (pad_dim[0] - l_w) / 2.0;
            let l_y = pad_pos[1] + (pad_dim[1] - l_size as f64) / 2.0;
            let l_pos = [l_x, l_y];
            ui.draw_text(graphics, l_pos, l_size, l_color, l_text);
        }
        // xy value string.
        let x_string = val_to_string(self.x, self.max_x,
                                     self.max_x - self.min_x, self.dim[0] as usize);
        let y_string = val_to_string(self.y, self.max_y,
                                     self.max_y - self.min_y, self.dim[1] as usize);
        let xy_string = format!("{}, {}", x_string, y_string);
        let xy_string_w = label::width(ui, self.font_size, &xy_string);
        let xy_string_pos = {
            match rectangle::corner(pad_pos, [vert_x, hori_y], pad_dim) {
                Corner::TopLeft => [vert_x, hori_y],
                Corner::TopRight => [vert_x - xy_string_w, hori_y],
                Corner::BottomLeft => [vert_x, hori_y - self.font_size as f64],
                Corner::BottomRight => [vert_x - xy_string_w, hori_y - self.font_size as f64],
            }
        };
        ui.draw_text(graphics, xy_string_pos, self.font_size,
                    color.plain_contrast(), &xy_string);

        set_state(ui, self.ui_id, Kind::XYPad(new_state), self.pos, self.dim);

    }
}
