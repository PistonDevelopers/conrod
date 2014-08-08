
#![feature(phase)]

#[phase(plugin, link)]
extern crate conrod;
extern crate graphics;
extern crate piston;
extern crate sdl2_game_window;
extern crate opengl_graphics;

use canvas::Canvas;
use sdl2_game_window::GameWindowSDL2;
use opengl_graphics::Gl;
use piston::{
    Game,
    GameEvent,
    GameWindowSettings,
    GameIterator,
    GameIteratorSettings,
    RenderArgs,
    Render,
};
pub use widget = conrod::widget;
use conrod::{
    Widget,
};
use graphics::{
    Context,
    AddColor,
    Draw,
};

mod canvas;

fn main() {

    // Create a SDL2 window.
    let mut window = GameWindowSDL2::new(
        GameWindowSettings {
            title: "Hello Conrod".to_string(),
            size: [600, 600],
            fullscreen: false,
            exit_on_esc: true
        }
    );

    // Some settings for how the game should be run.
    let game_iter_settings = GameIteratorSettings {
        updates_per_second: 180,
        max_frames_per_second: 60
    };

    // Create GameIterator to begin the event iteration loop.
    let mut game_iter = GameIterator::new(&mut window, &game_iter_settings);
    // Create OpenGL instance.
    let mut gl = Gl::new();
    // Create UI app.
    let mut canvas = Canvas::new();

    loop {
        match game_iter.next() {
            None => break,
            Some(mut e) => handle_event(&mut e, &mut canvas, &mut gl),
        }
    }

}

/// Match the game event.
fn handle_event(event: &mut GameEvent,
                canvas: &mut Canvas,
                gl: &mut Gl) {
    canvas.event(event);
    match *event {
        Render(ref mut args) => {
            draw_background(args, gl);
            canvas.draw(args, gl)
        },
        _ => (),
    }
}

/// Draw the window background.
fn draw_background(args: &RenderArgs, gl: &mut Gl) {
    // Set up a context to draw into.
    let context = &Context::abs(args.width as f64, args.height as f64);
    // Draw the background.
    context.rgba(0.075,0.05,0.1,1.0).draw(gl);
}

