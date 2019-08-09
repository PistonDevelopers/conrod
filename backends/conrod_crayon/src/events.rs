use conrod_core::event::Input;
use conrod_core::input::Button::Keyboard;
use conrod_core::input::Motion;
use conrod_core::Ui;
use crayon::prelude::*;
use instant::Instant;
pub fn convert_event(ui:&mut Ui)->Option<Instant>{
    let mouse_presses = input::mouse_presses();
    let w = ui.win_w;
    let h = ui.win_h;
    let mut action_time = None;
    for mp in mouse_presses.iter(){
        let e = match mp{
            crayon::input::mouse::MouseButton::Left => conrod_core::input::state::mouse::Button::Left,
            crayon::input::mouse::MouseButton::Right => conrod_core::input::state::mouse::Button::Right,
            crayon::input::mouse::MouseButton::Middle => conrod_core::input::state::mouse::Button::Middle,
            crayon::input::mouse::MouseButton::Other(_j) => conrod_core::input::state::mouse::Button::Unknown
        };
        ui.handle_event(Input::Press(conrod_core::input::Button::Mouse(e)));
        if let None = action_time{
            action_time = Some(Instant::now());
        }
    }
    let mouse_releases = input::mouse_releases();
    for mp in mouse_releases.iter(){
        let e = match mp{
            crayon::input::mouse::MouseButton::Left => conrod_core::input::state::mouse::Button::Left,
            crayon::input::mouse::MouseButton::Right => conrod_core::input::state::mouse::Button::Right,
            crayon::input::mouse::MouseButton::Middle => conrod_core::input::state::mouse::Button::Middle,
            crayon::input::mouse::MouseButton::Other(_j) => conrod_core::input::state::mouse::Button::Unknown
        };
        ui.handle_event(Input::Release(conrod_core::input::Button::Mouse(e)));
        if let None = action_time{
            action_time = Some(Instant::now());
        }
    }
    let key_presses = input::key_presses();
    for kp in key_presses.iter(){
        let e = key_convert(serde_json::to_string(kp).unwrap());
        let ee:conrod_core::input::keyboard::Key = serde_json::from_str(&e).unwrap();
        ui.handle_event(Input::Press(Keyboard(ee)));
        if let None = action_time{
            action_time = Some(Instant::now());
        }
    }
    let key_releases = input::key_releases();
    for kp in key_releases.iter(){
        let e = key_convert(serde_json::to_string(kp).unwrap());
        let ee:conrod_core::input::keyboard::Key = serde_json::from_str(&e).unwrap();
        ui.handle_event(Input::Release(Keyboard(ee)));
        if let None = action_time{
            action_time = Some(Instant::now());
        }
    }

    let j = input::mouse_position();
    ui.handle_event(Input::Motion(Motion::MouseCursor{x:(j.x as f64)-w/2.0,y:(j.y as f64)-h/2.0}));
    let j = input::mouse_scroll();
    if j.x > 0.0 || j.y >0.0{
        ui.handle_event(Input::Motion(Motion::Scroll{x:(j.x as f64)-w/2.0,y:(j.y as f64)-h/2.0}));
        if let None = action_time{
            action_time = Some(Instant::now());
        }
    }

    for k in input::chars(){
        if k != '\u{8}'{
            ui.handle_event(Input::Text(k.to_string()));
        }
        if let None = action_time{
            action_time = Some(Instant::now());
        }
    }
    action_time
}

pub fn key_convert(j:String)->String{
    j.replace("Key","D").replace("Control","Ctrl").replace("LBracket","LeftBracket").replace("RBracket","RightBracket")
    .replace("Subtract","Minus").replace("Add","Plus").replace("Back","Backspace").replace("Backspaceslash","Backslash")
}
