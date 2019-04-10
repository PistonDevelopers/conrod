use conrod_core::event::Input;
use conrod_core::input::Button::Keyboard;
use conrod_core::input::Button::Mouse;
use conrod_core::input::Motion;
use conrod_core::Ui;
use crayon::prelude::*;
use crayon::utils::hash::FastHashSet;
use serde_json::Result;

pub fn convert_event<T>(ui:&mut Ui,closure:Box<FnMut(&mut T)>,app:&mut T){
    let mouse_presses = input::mouse_presses();
    for mp in mouse_presses.iter(){
        let e = match mp{
            crayon::input::mouse::MouseButton::Left => conrod_core::input::state::mouse::Button::Left,
            crayon::input::mouse::MouseButton::Right => conrod_core::input::state::mouse::Button::Right,
            crayon::input::mouse::MouseButton::Middle => conrod_core::input::state::mouse::Button::Middle,
            crayon::input::mouse::MouseButton::Other(_j) => conrod_core::input::state::mouse::Button::Unknown
        };
        ui.handle_event(Input::Press(conrod_core::input::Button::Mouse(e)));
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
    }
    let key_presses = input::key_presses();
    for kp in key_presses.iter(){
        let e = serde_json::to_string(kp).unwrap();
        let ee:conrod_core::input::keyboard::Key = serde_json::from_str(&e).unwrap();
        ui.handle_event(Input::Press(Keyboard(ee)));
    }
    let key_releases = input::key_releases();
    for kp in key_releases.iter(){
        let e = serde_json::to_string(kp).unwrap();
        let ee:conrod_core::input::keyboard::Key = serde_json::from_str(&e).unwrap();
        ui.handle_event(Input::Release(Keyboard(ee)));
    }
    let k = input::mouse_movement();
    if k.x >0.0 || k.y >0.0 {
        let j = input::mouse_position();
        ui.handle_event(Input::Motion(Motion::MouseCursor{x:j.x as f64,y:j.y as f64}));
    }
    let j = input::mouse_scroll();
    if j.x > 0.0 || j.y >0.0{
        ui.handle_event(Input::Motion(Motion::Scroll{x:j.x as f64,y:j.y as f64}));
    }

}
