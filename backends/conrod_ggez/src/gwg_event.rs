use conrod_core::input;
use conrod_core::event::Input;
use conrod_core::input::Button::Keyboard;
use conrod_core::input::Button::Mouse;
use conrod_core::input::Motion;
use conrod_core::Ui;
use stdweb::web::event::MouseButton;
pub fn map_key(keycode: String) -> input::keyboard::Key {
    input::keyboard::Key::Hash
}

pub fn mouse_button_down_event(ui:&mut Ui,button: MouseButton){
    let mmb  =match button{
        Left=> conrod_core::input::state::mouse::Button::Left,
        Right=> conrod_core::input::state::mouse::Button::Right,
        Wheel=> conrod_core::input::state::mouse::Button::Middle,
        Button4 => conrod_core::input::state::mouse::Button::Unknown,
        Button5 => conrod_core::input::state::mouse::Button::Unknown
    };
    ui.handle_event(Input::Press(Mouse(mmb)));
}
pub fn mouse_button_up_event(ui:&mut Ui,button: MouseButton){
    let mmb  =match button{
        Left=> conrod_core::input::state::mouse::Button::Left,
        Right=> conrod_core::input::state::mouse::Button::Right,
        Wheel=> conrod_core::input::state::mouse::Button::Middle,
        Button4 => conrod_core::input::state::mouse::Button::Unknown,
        Button5 => conrod_core::input::state::mouse::Button::Unknown
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
pub fn key_down_event(ui:&mut Ui,key:String){
    //ui.handle_event(Input::Press(Keyboard(map_key(key.clone()))));
}
pub fn key_up_event(ui:&mut Ui,key:String){
    //ui.handle_event(Input::Release(Keyboard(map_key(key.clone()))));
}
pub fn text_input_event(ui:&mut Ui,ch: char){
     ui.handle_event(Input::Text(ch.to_string()));
}