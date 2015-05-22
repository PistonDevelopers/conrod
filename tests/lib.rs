extern crate conrod;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston;
extern crate piston_window;
extern crate viewport;
extern crate window;


use conrod::{Ui,Theme,Button,Sizeable,Labelable,Widget};

use glutin_window::GlutinWindow;
use opengl_graphics::{ GlGraphics, OpenGL };
use opengl_graphics::glyph_cache::GlyphCache;
use piston::event::*;
use piston::window::{ WindowSettings, Size };
use piston_window::PistonWindow;
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
    
    let font_path = Path::new("./assets/NotoSans/NotoSans-Regular.ttf");
    let theme = Theme::default();
    let glyph_cache = GlyphCache::new(&font_path);
    let ui = Ui::new(glyph_cache.unwrap(), theme);

    let mut gl = GlGraphics::new(opengl);

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
fn test_auto_uiid() {
    let (view,mut gl,mut ui) = setup_test();

    let b1 = ui.add_uiid();
    let b2 = ui.add_uiid();
    let b3 = ui.add_uiid();
    ui.remove_uiid(&b2);
    
    gl.draw(view, |c, g| {
        if let Some(b) = ui.get_uiid(&b1) {
            Button::new().dimensions(80.0, 80.0).label("b1").react(|| { }).set(b, &mut ui);
        }
        if let Some(b) = ui.get_uiid(&b2) {
            Button::new().dimensions(80.0, 80.0).label("b2").react(|| { }).set(b, &mut ui);
        }
        ui.draw(c,g);
    });
}
