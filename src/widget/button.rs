
use color::{Color, Colorable};
use dimensions::Dimensions;
use frame::Frameable;
use graphics::Graphics;
use graphics::character::CharacterCache;
use label::{FontSize, Labelable};
use mouse::Mouse;
use point::Point;
use position::{Position, Positionable};
use rectangle;
use shape::Shapeable;
use ui::{UiId, Ui};
use widget::{Kind, Widget};

/// Represents the state of the Button widget.
#[derive(PartialEq, Clone, Copy)]
pub enum State {
    Normal,
    Highlighted,
    Clicked,
}

impl State {
    /// Alter the widget color depending on the state.
    fn color(&self, color: Color) -> Color {
        match *self {
            State::Normal => color,
            State::Highlighted => color.highlighted(),
            State::Clicked => color.clicked(),
        }
    }
}

widget_fns!(Button, State, Kind::Button(State::Normal));

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

/// A pressable button widget whose callback is triggered upon release.
pub struct Button<'a, F> {
    ui_id: UiId,
    pos: Point,
    dim: Dimensions,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_label: Option<&'a str>,
    maybe_label_color: Option<Color>,
    maybe_label_font_size: Option<u32>,
    maybe_callback: Option<F>,
}

impl<'a, F> Button<'a, F> {

    /// Create a button context to be built upon.
    pub fn new(ui_id: UiId) -> Button<'a, F> {
        Button {
            ui_id: ui_id,
            pos: Position::default(),
            dim: [64.0, 64.0],
            maybe_callback: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
        }
    }

    /// Set the callback for the Button. The callback will be triggered upon release of the button.
    pub fn callback(mut self, cb: F) -> Button<'a, F> {
        self.maybe_callback = Some(cb);
        self
    }

    /// After building the Button, use this method to set its current state into the given `Ui`.
    /// It will use this state for rendering the next time `ui.draw(graphics)` is called.
    pub fn set<C>(mut self, ui: &mut Ui<C>) {
        use elmesque::form::{collage, Form, group, rect, text};
        use utils::is_over_rect;

        let state = *get_state(ui, self.ui_id);
        let xy = ui.get_absolute_xy(self.pos, self.dim);
        let mouse = ui.get_mouse_state();
        let is_over = is_over_rect(xy, mouse.xy, self.dim);
        let new_state = get_new_state(is_over, state, mouse);
        let (w, h) = (self.dim[0], self.dim[1]);

        // Callback.
        if let (true, State::Clicked, State::Highlighted) = (is_over, state, new_state) {
            if let Some(ref mut callback) = self.maybe_callback { callback() }
        }

        // Consruct the frame and pressable forms.
        let frame_w = self.maybe_frame.unwrap_or(ui.theme.frame_width);
        let frame_color = button.maybe_frame_color.unwrap_or(ui.theme.frame_color);
        let (inner_w, inner_h) = (button.dim[0] - frame_w,  button.dim[1] - frame_w);
        let frame_form = rect(w, h).filled(frame_color);
        let color = new_state.color(button.maybe_color.unwrap_or(ui.theme.shape_color));
        let pressable_form = rect(inner_w, inner_h).filled(color);

        // Construct the label's Form.
        let maybe_label_form = self.maybe_label.map(|label_text| {
            use elmesque::text::Text;
            let text_color = self.maybe_label_color.unwrap_or(ui.theme.label_color);
            let size = self.maybe_label_font_size.unwrap_or(ui.theme.font_size_medium);
            text(Text::from_string(label_text.to_string()).color(text_color).height(size))
        });

        // Construct the button's Form.
        let form = group(Some(frame_form).into_iter()
            .chain(Some(pressable_form).into_iter())
            .chain(maybe_label_form.into_iter())
            .collect());

        // Construct the button's element.
        let element = collage(dim[0] as i32, dim[1] as i32, vec![form.shift(xy[0], xy[1])]);

        // Store the widget's new state in the Ui.
        ui.set_widget(self.ui_id, Widget {
            kind: Kind::Button(new_state),
            xy: xy,
            dim: dim,
            depth: depth,
            form: Some(form),
        });

    }

}

impl<'a, F> Colorable for Button<'a, F> {
    fn color(mut self, color: Color) -> Self {
        self.maybe_color = Some(color);
        self
    }
}

impl<'a, F> Frameable for Button<'a, F> {
    fn frame(mut self, width: f64) -> Self {
        self.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, F> Labelable<'a> for Button<'a, F> {
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

impl<'a, F> Positionable for Button<'a, F> {
    fn position(mut self, pos: Position) -> Self {
        self.pos = pos;
        self
    }
}

impl<'a, F> Shapeable for Button<'a, F> {
    fn get_dim(&self) -> Dimensions { self.dim }
    fn dim(mut self, dim: Dimensions) -> Self { self.dim = dim; self }
}

// impl<'a, F> ::draw::Drawable for Button<'a, F>
//     where
//         F: FnMut() + 'a
// {
// 
//     fn draw<B, C>(&mut self, ui: &mut Ui<C>, graphics: &mut B)
//         where
//             B: Graphics<Texture = <C as CharacterCache>::Texture>,
//             C: CharacterCache
//     {
// 
//         let state = *get_state(ui, self.ui_id);
//         let mouse = ui.get_mouse_state();
//         let is_over = rectangle::is_over(self.pos, mouse.pos, self.dim);
//         let new_state = get_new_state(is_over, state, mouse);
// 
//         // Callback.
//         match (is_over, state, new_state) {
//             (true, State::Clicked, State::Highlighted) => match self.maybe_callback {
//                 Some(ref mut callback) => (*callback)(), None => (),
//             }, _ => (),
//         }
// 
//         // Draw.
//         let rect_state = new_state.as_rectangle_state();
//         let color = self.maybe_color.unwrap_or(ui.theme.shape_color);
//         let frame_w = self.maybe_frame.unwrap_or(ui.theme.frame_width);
//         let maybe_frame = match frame_w > 0.0 {
//             true => Some((frame_w, self.maybe_frame_color.unwrap_or(ui.theme.frame_color))),
//             false => None,
//         };
//         match self.maybe_label {
//             None => {
//                 rectangle::draw(
//                     ui.win_w, ui.win_h, graphics, rect_state, self.pos,
//                     self.dim, maybe_frame, color
//                 )
//             },
//             Some(text) => {
//                 let text_color = self.maybe_label_color.unwrap_or(ui.theme.label_color);
//                 let size = self.maybe_label_font_size.unwrap_or(ui.theme.font_size_medium);
//                 rectangle::draw_with_centered_label(
//                     ui.win_w, ui.win_h, graphics, ui, rect_state,
//                     self.pos, self.dim, maybe_frame, color,
//                     text, size, text_color
//                 )
//             },
//         }
// 
//         set_state(ui, self.ui_id, Kind::Button(new_state), self.pos, self.dim);
// 
//     }
// }
