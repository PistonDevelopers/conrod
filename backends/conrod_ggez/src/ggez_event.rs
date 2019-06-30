use ggez::input::keyboard::{KeyCode,KeyMods};
use ggez::input::mouse::{MouseButton::{self,Left,Right,Middle,Other}};
use conrod_winit::map_key;
use conrod_core::event::Input;
use conrod_core::input::Button::Keyboard;
use conrod_core::input::Button::Mouse;
use conrod_core::input::Motion;
use conrod_core::Ui;

pub fn mouse_button_down_event(ui:&mut Ui,button: MouseButton){
    let mmb  =match button{
        Left=> conrod_core::input::state::mouse::Button::Left,
        Right=> conrod_core::input::state::mouse::Button::Right,
        Middle=> conrod_core::input::state::mouse::Button::Middle,
        Other(_j) => conrod_core::input::state::mouse::Button::Unknown
    };
    ui.handle_event(Input::Press(Mouse(mmb)));
}
pub fn mouse_button_up_event(ui:&mut Ui,button: MouseButton){
    let mmb  =match button{
        Left=> conrod_core::input::state::mouse::Button::Left,
        Right=> conrod_core::input::state::mouse::Button::Right,
        Middle=> conrod_core::input::state::mouse::Button::Middle,
        Other(_j) => conrod_core::input::state::mouse::Button::Unknown
    };
    ui.handle_event(Input::Release(Mouse(mmb)));
}
pub fn mouse_motion_event(ui:&mut Ui,xx: f32, yy: f32){
    let x = xx as f64;
    let y = yy as f64;
    ui.handle_event(Input::Motion(Motion::MouseCursor{x,y}));
}
pub fn mouse_wheel_event(ui:&mut Ui,xx: f32, yy: f32){
    let x = xx as f64;
    let y = yy as f64;
    ui.handle_event(Input::Motion(Motion::Scroll{x,y}));
}
pub fn key_down_event(ui:&mut Ui,key:KeyCode,_keymod:KeyMods){
    ui.handle_event(Input::Press(Keyboard(map_key(key.clone()))));
    ui.handle_event(Input::Press(Keyboard(map_key(key.clone()))));
}
pub fn key_up_event(ui:&mut Ui,key:KeyCode,_keymod:KeyMods){
    ui.handle_event(Input::Release(Keyboard(map_key(key.clone()))));
}
pub fn text_input_event(ui:&mut Ui,ch: char){
     ui.handle_event(Input::Text(ch.to_string()));
}