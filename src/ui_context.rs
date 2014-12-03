use Color;
use dimensions::Dimensions;
use opengl_graphics::glyph_cache::{
    GlyphCache,
    Character,
};
use opengl_graphics::Gl;
use label::FontSize;
use mouse_state::{
    MouseButtonState,
    MouseState,
};
use input;
use event::{
    GenericEvent,
    MouseCursorEvent,
    PressEvent,
    ReleaseEvent,
    RenderEvent,
    TextEvent,
};
use point::Point;
use theme::Theme;
use widget;
use widget::Widget;

/// User Interface Identifier. Each unique `widget::draw` call
/// should pass it's own unique UIID so that UiContext can keep
/// track of it's state.
pub type UIID = u64;

/// UiContext retains the state of all widgets and
/// data relevant to the draw_widget functions.
pub struct UiContext {
    data: Vec<(Widget, widget::Placing)>,
    pub theme: Theme,
    pub mouse: MouseState,
    pub keys_just_pressed: Vec<input::keyboard::Key>,
    pub keys_just_released: Vec<input::keyboard::Key>,
    pub text_just_entered: Vec<String>,
    glyph_cache: GlyphCache,
    prev_event_was_render: bool,
    /// Window width.
    pub win_w: f64,
    /// Window height.
    pub win_h: f64,
    /// The UIID of the widget drawn previously.
    prev_uiid: u64,
}

impl UiContext {

    /// Constructor for a UiContext.
    pub fn new(glyph_cache: GlyphCache, theme: Theme) -> UiContext {
        UiContext {
            data: Vec::from_elem(512, (widget::Widget::NoWidget, widget::Placing::NoPlace)),
            theme: theme,
            mouse: MouseState::new([0f64, 0f64], MouseButtonState::Up, MouseButtonState::Up, MouseButtonState::Up),
            keys_just_pressed: Vec::with_capacity(10u),
            keys_just_released: Vec::with_capacity(10u),
            text_just_entered: Vec::with_capacity(10u),
            glyph_cache: glyph_cache,
            prev_event_was_render: false,
            win_w: 0f64,
            win_h: 0f64,
            prev_uiid: 0u64,
        }
    }

    /// Handle game events and update the state.
    pub fn handle_event<E: GenericEvent>(&mut self, event: &E) {
        if self.prev_event_was_render {
            self.flush_input();
            self.prev_event_was_render = false;
        }
        event.render(|args| {
            self.win_w = args.width as f64;
            self.win_h = args.height as f64;
            self.prev_event_was_render = true;
        });
        event.mouse_cursor(|x, y| {
            self.mouse.pos = [x, y];
        });
        event.press(|button_type| {
            use input::Button;
            use input::mouse::Button::Left;

            match button_type {
                Button::Mouse(button) => {
                    *match button {
                        Left => &mut self.mouse.left,
                        _/*input::mouse::Right*/ => &mut self.mouse.right,
                        //Middle => &mut self.mouse.middle,
                    } = MouseButtonState::Down;
                },
                Button::Keyboard(key) => self.keys_just_pressed.push(key),
            }
        });
        event.release(|button_type| {
            use input::Button;
            use input::mouse::Button::Left;

            match button_type {
                Button::Mouse(button) => {
                    *match button {
                        Left => &mut self.mouse.left,
                        _/*input::mouse::Right*/ => &mut self.mouse.right,
                        //Middle => &mut self.mouse.middle,
                    } = MouseButtonState::Up;
                },
                Button::Keyboard(key) => self.keys_just_released.push(key),
            }
        });
        event.text(|text| {
            self.text_just_entered.push(text.to_string())
        });
    }

    /// Return the current mouse state.
    pub fn get_mouse_state(&self) -> MouseState {
        self.mouse
    }

    /// Return the vector of recently pressed keys.
    pub fn get_pressed_keys(&self) -> Vec<input::keyboard::Key> {
        self.keys_just_pressed.clone()
    }

    /// Return the vector of recently entered text.
    pub fn get_entered_text(&self) -> Vec<String> {
        self.text_just_entered.clone()
    }

    /// Return a mutable reference to the widget that matches the given ui_id
    pub fn get_widget(&mut self, ui_id: UIID, default: Widget) -> &mut Widget {
        let ui_id_idx = ui_id as uint;
        if self.data.len() > ui_id_idx {
            match &mut self.data[ui_id_idx] {
                &(widget::Widget::NoWidget, _) => {
                    match &mut self.data[ui_id_idx] {
                        &(ref mut widget, _) => {
                            *widget = default; widget
                        }
                    }
                },
                _ => {
                    match &mut self.data[ui_id_idx] {
                        &(ref mut widget, _) => widget
                    }
                },
            }
        } else {
            if ui_id_idx >= self.data.len() {
                let num_to_push = ui_id_idx - self.data.len();
                let mut vec = Vec::from_elem(num_to_push, (widget::Widget::NoWidget, widget::Placing::NoPlace));
                vec.push((default, widget::Placing::NoPlace));
                self.data.extend(vec.into_iter());
            } else {
                self.data[ui_id_idx] = (default, widget::Placing::NoPlace);
            }
            match &mut self.data[ui_id_idx] {
                &(ref mut widget, _) => widget,
            }
        }
    }

    /// Set the Placing for a particular widget.
    pub fn set_place(&mut self, ui_id: UIID, pos: Point, dim: Dimensions) {
        match &mut self.data[ui_id as uint] {
            &(_, ref mut placing) => {
                *placing = widget::Placing::Place(pos[0], pos[1], dim[0], dim[1])
            }
        }
        self.prev_uiid = ui_id;
    }

    /// Get the UIID of the previous widget.
    pub fn get_prev_uiid(&self) -> UIID { self.prev_uiid }

    /// Get the Placing for a particular widget.
    pub fn get_placing(&self, ui_id: UIID) -> widget::Placing {
        if ui_id as uint >= self.data.len() { widget::Placing::NoPlace }
        else {
            match self.data[ui_id as uint] { (_, ref placing) => *placing }
        }
    }

    /// Return a reference to a `Character` from the GlyphCache.
    pub fn get_character(&mut self, size: FontSize, ch: char) -> &Character {
        use graphics::character::CharacterCache;        

        self.glyph_cache.character(size, ch)
    }

    /// Return the width of a 'Character'.
    pub fn get_character_w(&mut self, size: FontSize, ch: char) -> f64 {
        self.get_character(size, ch).width()
    }

    /// Flush all stored keys.
    pub fn flush_input(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.text_just_entered.clear();
    }

    /// Draws text
    pub fn draw_text(
        &mut self,
        graphics: &mut Gl,
        pos: Point,
        size: FontSize,
        color: Color,
        text: &str
    ) {
        use graphics::Context;
        use graphics::text::Text;
        use graphics::RelativeTransform;

        let (r, g, b, a) = color.as_tuple();
        let context = Context::abs(self.win_w, self.win_h)
                        .trans(pos[0], pos[1] + size as f64);
        Text::colored([r, g, b, a], size).draw(
            text,
            &mut self.glyph_cache,
            &context,
            graphics
        );
    }

}
