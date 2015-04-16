
use color::Color;
use dimensions::Dimensions;
use graphics::{self, Graphics};
use graphics::character::{Character, CharacterCache};
use label::FontSize;
use mouse::{ButtonState, Mouse};
use piston::input;
use piston::event::{
    GenericEvent,
    MouseCursorEvent,
    PressEvent,
    ReleaseEvent,
    RenderEvent,
    TextEvent,
};
use point::Point;
use std::iter::repeat;
use theme::Theme;
use widget::Kind as WidgetKind;
use widget::{Placing, Widget};

/// User Interface Identifier. Each unique `widget::draw` call
/// should pass it's own unique UIID so that UiContext can keep
/// track of it's state.
pub type UIID = u64;

/// UiContext retains the state of all widgets and
/// data relevant to the draw_widget functions.
pub struct Ui<C> {
    data: Vec<Widget>,
    pub theme: Theme,
    pub mouse: Mouse,
    pub keys_just_pressed: Vec<input::keyboard::Key>,
    pub keys_just_released: Vec<input::keyboard::Key>,
    pub text_just_entered: Vec<String>,
    character_cache: C,
    prev_event_was_render: bool,
    /// Window width.
    pub win_w: f64,
    /// Window height.
    pub win_h: f64,
    /// The UIID of the widget drawn previously.
    prev_uiid: u64,
}

impl<C> Ui<C>
    where
        C: CharacterCache
{

    /// Constructor for a UiContext.
    pub fn new(character_cache: C, theme: Theme) -> Ui<C> {
        Ui {
            data: repeat(Widget::empty()).take(512).collect(),
            theme: theme,
            mouse: Mouse::new([0.0, 0.0], ButtonState::Up, ButtonState::Up, ButtonState::Up),
            keys_just_pressed: Vec::with_capacity(10),
            keys_just_released: Vec::with_capacity(10),
            text_just_entered: Vec::with_capacity(10),
            character_cache: character_cache,
            prev_event_was_render: false,
            win_w: 0.0,
            win_h: 0.0,
            prev_uiid: 0,
        }
    }

    /// Handle game events and update the state.
    pub fn handle_event<E: GenericEvent + ::std::fmt::Debug>(&mut self, event: &E) {
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
            use piston::input::Button;
            use piston::input::MouseButton::Left;

            match button_type {
                Button::Mouse(button) => {
                    *match button {
                        Left => &mut self.mouse.left,
                        _/*input::mouse::Right*/ => &mut self.mouse.right,
                        //Middle => &mut self.mouse.middle,
                    } = ButtonState::Down;
                },
                Button::Keyboard(key) => self.keys_just_pressed.push(key),
            }
        });
        event.release(|button_type| {
            use piston::input::Button;
            use piston::input::MouseButton::Left;

            match button_type {
                Button::Mouse(button) => {
                    *match button {
                        Left => &mut self.mouse.left,
                        _/*input::mouse::Right*/ => &mut self.mouse.right,
                        //Middle => &mut self.mouse.middle,
                    } = ButtonState::Up;
                },
                Button::Keyboard(key) => self.keys_just_released.push(key),
            }
        });
        event.text(|text| {
            self.text_just_entered.push(text.to_string())
        });
    }

    /// Return a reference to a `Character` from the GlyphCache.
    pub fn get_character(&mut self,
                         size: FontSize,
                         ch: char) -> &Character<<C as CharacterCache>::Texture> {
        self.character_cache.character(size, ch)
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
    pub fn draw_text<B>(&mut self,
                        graphics: &mut B,
                        pos: Point,
                        size: FontSize,
                        color: Color,
                        text: &str)
        where
            B: Graphics<Texture = <C as CharacterCache>::Texture>
    {
        use graphics::text::Text;
        use graphics::Transformed;
        use num::Float;

        let draw_state = graphics::default_draw_state();
        let transform = graphics::abs_transform(self.win_w, self.win_h)
                        .trans(pos[0].ceil(), pos[1].ceil() + size as f64);
        Text::colored(color.to_fsa(), size).draw(
            text,
            &mut self.character_cache,
            draw_state,
            transform,
            graphics
        );
    }

}

impl<C> Ui<C> {
    /// Return the current mouse state.
    pub fn get_mouse_state(&self) -> Mouse {
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
    pub fn get_widget(&mut self, ui_id: UIID, default: WidgetKind) -> &mut WidgetKind {
        let ui_id_idx = ui_id as usize;
        if self.data.len() > ui_id_idx {
            match &mut self.data[ui_id_idx].kind {
                &mut WidgetKind::NoWidget => {
                    let mut widget = &mut self.data[ui_id_idx].kind;
                    *widget = default;
                    widget
                },
                _ => &mut self.data[ui_id_idx].kind,
            }
        } else {
            if ui_id_idx >= self.data.len() {
                let num_to_extend = ui_id_idx - self.data.len();
                self.data.extend(repeat(Widget::empty())
                    .take(num_to_extend)
                    .chain(Some(Widget::new(default)).into_iter()));
            } else {
                self.data[ui_id_idx] = Widget::new(default);
            }
            &mut self.data[ui_id_idx].kind
        }
    }

    /// Set the Placing for a particular widget.
    pub fn set_place(&mut self, ui_id: UIID, pos: Point, dim: Dimensions) {
        self.data[ui_id as usize].placing = Placing::Place(pos[0], pos[1], dim[0], dim[1]);
        self.prev_uiid = ui_id;
    }

    /// Get the UIID of the previous widget.
    pub fn get_prev_uiid(&self) -> UIID { self.prev_uiid }

    /// Get the Placing for a particular widget.
    pub fn get_placing(&self, ui_id: UIID) -> Placing {
        if ui_id as usize >= self.data.len() { Placing::NoPlace }
        else { self.data[ui_id as usize].placing }
    }
}
