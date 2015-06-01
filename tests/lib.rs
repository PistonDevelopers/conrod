extern crate conrod;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston;
extern crate piston_window;
extern crate viewport;
extern crate window;


use conrod::{Ui,Theme,Button, Label,Sizeable,Labelable,Widget};

use glutin_window::GlutinWindow;
use opengl_graphics::{ GlGraphics, OpenGL };
use opengl_graphics::glyph_cache::GlyphCache;
//use piston::event::*;
use piston::window::{ WindowSettings, Size };
//use piston_window::PistonWindow;
use std::path::Path;
use viewport::Viewport;
use window::Window;

pub fn setup_test<'a> () -> (Viewport,GlGraphics, Ui<GlyphCache<'a>>) {
    let opengl = OpenGL::_3_2;
    let window = GlutinWindow::new(
        opengl,
        WindowSettings::new(
            "Conrod Test".to_string(),
            Size { width: 1024, height: 768 }
            )
            .exit_on_esc(true)
            .samples(4)
            );
    
    let font_path = Path::new("./assets/fonts/NotoSans/NotoSans-Regular.ttf");
    let theme = Theme::default();
    let glyph_cache = GlyphCache::new(&font_path);
    let ui = Ui::new(glyph_cache.unwrap(), theme);

    let gl = GlGraphics::new(opengl);

    let size = window.size();
    let draw_size = window.draw_size();

    let viewport = Viewport {
        rect: [0, 0, draw_size.width as i32, draw_size.height as i32],
        window_size: [size.width, size.height],
        draw_size: [draw_size.width, draw_size.height],
    };
    
    (viewport,gl,ui)
}

#[test]
fn test_basic() {
    let (view,mut gl,mut ui) = setup_test();
    gl.draw(view, |c, g| {
        Button::new().dimensions(80.0, 80.0).label("0").react(|| { }).set(0, &mut ui);
        ui.draw(c,g);
    });
}

#[test]
#[should_panic]
fn test_basic_panic() {
    let (view,mut gl,mut ui) = setup_test();
    Label::new("0").dimensions(80.0, 80.0).set(0, &mut ui);
    gl.draw(view, |c, g| {
        Button::new().dimensions(80.0, 80.0).label("0").react(|| { }).set(0, &mut ui);
        ui.draw(c,g);
    });
}

#[test]
fn test_auto_uiid() {
    let (view,mut gl,mut ui) = setup_test();
    
    let b1 = ui.add_id();
    let b2 = ui.add_id();
    let b3 = ui.add_id();
    let l1 = ui.add_id();
    
    for _ in (0..2) {
        gl.draw(view, |c, g| {
            Button::new().dimensions(80.0, 80.0).label("b3").react(|| { }).set_if(&b3, &mut ui);
            Label::new("l1").dimensions(80.0, 80.0).set_if(&l1, &mut ui);
            Button::new().dimensions(80.0, 80.0).label("b2").react(|| { }).set_if(&b2, &mut ui);
            Button::new().dimensions(80.0, 80.0).label("b1").react(|| { }).set_if(&b1, &mut ui);
            
            ui.draw(c,g);
        });
    }
}
