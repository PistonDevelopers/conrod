#![feature(test)]

extern crate conrod;
extern crate gfx_device_gl;
extern crate gfx_graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston;
extern crate piston_window;
extern crate test;
extern crate viewport;
extern crate window;

use conrod::{Button, Labelable, Sizeable, TextBox, Theme, Ui};
use gfx_device_gl::Resources;
use gfx_graphics::GlyphCache as GfxGlyphCache;
use glutin_window::GlutinWindow;
use opengl_graphics::{ GlGraphics, OpenGL };
use opengl_graphics::glyph_cache::GlyphCache as GlGlyphCache;
use piston::window::{ WindowSettings, Size };
use piston_window::PistonWindow;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use test::Bencher;
use viewport::Viewport;
use window::Window;

const OPENGL_VERSION: OpenGL = OpenGL::_3_2;

fn viewport_from_window(window: &PistonWindow<GlutinWindow>) -> Viewport {
    let size = window.size();
    let draw_size = window.draw_size();

    Viewport {
        rect: [0, 0, draw_size.width as i32, draw_size.height as i32],
        window_size: [size.width, size.height],
        draw_size: [draw_size.width, draw_size.height],
    }
}

fn init_window() -> PistonWindow<GlutinWindow> {
    let window = GlutinWindow::new(
        OPENGL_VERSION,
        WindowSettings::new("Conrod Bench".to_string(), Size { width: 800, height: 600 })
    );
    PistonWindow::new(Rc::new(RefCell::new(window)), piston_window::empty_app())
}

fn init_gl_ui<'a>() -> Ui<GlGlyphCache<'a>> {
    let font_path = Path::new("./assets/NotoSans/NotoSans-Regular.ttf");
    let theme = Theme::default();
    let glyph_cache = GlGlyphCache::new(&font_path);
    Ui::new(glyph_cache.unwrap(), theme)
}

fn init_gfx_ui(window: &mut PistonWindow<GlutinWindow>) -> Ui<GfxGlyphCache<Resources>> {
    let font_path = Path::new("./assets/NotoSans/NotoSans-Regular.ttf");
    let theme = Theme::default();
    let glyph_cache = GfxGlyphCache::new(&font_path, &mut window.canvas.borrow_mut().factory);
    Ui::new(glyph_cache.unwrap(), theme)
}

#[bench]
fn bench_gl_draw_counter(b: &mut Bencher) {
    let window = init_window();
    let viewport = viewport_from_window(&window);
    let ui = &mut init_gl_ui();
    let mut gl = GlGraphics::new(OPENGL_VERSION);

    b.iter(|| {
        gl.draw(viewport, |_context, g| {
            Button::new().dimensions(80.0, 80.0).label("0").react(|| { }).set(0, ui);
            ui.draw(g);
        });
    });
}

#[bench]
fn bench_gfx_draw_counter(b: &mut Bencher) {
    let mut window = init_window();
    let ui = &mut init_gfx_ui(&mut window);

    b.iter(|| {
        window.draw_2d(|_context, g| {
            Button::new().dimensions(80.0, 80.0).label("0").react(|| { }).set(0, ui);
            ui.draw(g);
        });
        ui.character_cache.update(&mut window.canvas.borrow_mut().factory);
    });
}

#[bench]
fn bench_gl_draw_textbox(b: &mut Bencher) {
    let window = init_window();
    let viewport = viewport_from_window(&window);
    let ui = &mut init_gl_ui();
    let mut gl = GlGraphics::new(OPENGL_VERSION);
    let mut string = String::new();

    b.iter(|| {
        gl.draw(viewport, |_context, g| {
            TextBox::new(&mut string).dimensions(320.0, 40.0).react(|_: &mut String| { }).set(0, ui);
            ui.draw(g);
        });
    });
}

#[bench]
fn bench_gfx_draw_textbox(b: &mut Bencher) {
    let mut window = init_window();
    let ui = &mut init_gfx_ui(&mut window);
    let mut string = String::new();

    b.iter(|| {
        window.draw_2d(|_context, g| {
            TextBox::new(&mut string).dimensions(320.0, 40.0).react(|_: &mut String| { }).set(0, ui);
            ui.draw(g);
        });
        ui.character_cache.update(&mut window.canvas.borrow_mut().factory);
    });
}

#[bench]
fn bench_gl_draw_textbox_m40(b: &mut Bencher) {
    let window = init_window();
    let viewport = viewport_from_window(&window);
    let ui = &mut init_gl_ui();
    let mut gl = GlGraphics::new(OPENGL_VERSION);
    let mut m40 = "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm".to_string();

    b.iter(|| {
        gl.draw(viewport, |_context, g| {
            TextBox::new(&mut m40).dimensions(320.0, 40.0).react(|_: &mut String| { }).set(0, ui);
            ui.draw(g);
        });
    });
}

#[bench]
fn bench_gfx_draw_textbox_m40(b: &mut Bencher) {
    let mut window = init_window();
    let ui = &mut init_gfx_ui(&mut window);
    let mut m40 = "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm".to_string();

    b.iter(|| {
        window.draw_2d(|_context, g| {
            TextBox::new(&mut m40).dimensions(320.0, 40.0).react(|_: &mut String| { }).set(0, ui);
            ui.draw(g);
        });
        ui.character_cache.update(&mut window.canvas.borrow_mut().factory);
    });
}
