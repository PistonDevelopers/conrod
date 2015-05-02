//! 
//! A simple demonstration of how to construct and use Canvasses by splitting up the window.
//!

extern crate conrod;
extern crate gfx_device_gl;
extern crate gfx_graphics;
extern crate glutin_window;
extern crate graphics;
extern crate piston;
extern crate piston_window;

use conrod::{CanvasId, Theme, Ui, UiId};
use gfx_device_gl::Resources;
use gfx_graphics::{GlyphCache, Texture};
use glutin_window::{GlutinWindow, OpenGL};
use graphics::Graphics;
use piston::window::{WindowSettings, Size};
use piston_window::PistonWindow;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;


fn main() {

    // Construct the window.
    let window = {
        let window = GlutinWindow::new(
            OpenGL::_3_2,
            WindowSettings::new("Canvas Demo".to_string(), Size { width: 800, height: 600 })
                .exit_on_esc(true)
        );
        PistonWindow::new(Rc::new(RefCell::new(window)), piston_window::empty_app())
    };

    // construct our `Ui`.
    let mut ui = {
        let font_path = Path::new("./assets/NotoSans/NotoSans-Regular.ttf");
        let theme = Theme::default();
        let glyph_cache = GlyphCache::new(&font_path, &mut window.canvas.borrow_mut().factory);
        Ui::new(glyph_cache.unwrap(), theme)
    };

    // Poll events from the window.
    for event in window {
        ui.handle_event(&event);
        event.draw_2d(|_c, g| draw_ui(&mut ui, g));
        ui.character_cache.update(&mut event.canvas.borrow_mut().factory);
    }

}


// Draw the Ui.
fn draw_ui<G: Graphics<Texture=Texture<Resources>>>(ui: &mut Ui<GlyphCache<Resources>>, g: &mut G) {
    use conrod::color::{blue, light_orange, orange, dark_orange};
    use conrod::{Button, Colorable, Label, Positionable, Sizeable, Split, WidgetMatrix};

    // Construct our Canvas tree.
    Split::new(MASTER).flow_down(&[
        Split::new(HEADER).color(blue()).pad_bottom(20.0),
        Split::new(BODY).flow_right(&[
            Split::new(LEFT_COLUMN).length(400.0).color(light_orange()).pad(20.0),
            Split::new(MIDDLE_COLUMN).color(orange()),
            Split::new(RIGHT_COLUMN).color(dark_orange()).pad(20.0),
        ]),
        Split::new(FOOTER).color(blue())
    ]).set(ui);

    Label::new("Fancy Title").color(light_orange()).size(48).middle_of(HEADER).set(TITLE, ui);
    Label::new("Subtitle").color(blue().complement()).mid_bottom_of(HEADER).set(SUBTITLE, ui);

    Label::new("Top Left")
        .color(light_orange().complement())
        .top_left_of(LEFT_COLUMN)
        .set(TOP_LEFT, ui);

    Label::new("Middle")
        .color(orange().complement())
        .middle_of(MIDDLE_COLUMN)
        .set(MIDDLE, ui);

    Label::new("Bottom Right")
        .color(dark_orange().complement())
        .bottom_right_of(RIGHT_COLUMN)
        .set(BOTTOM_RIGHT, ui);

    WidgetMatrix::new(24, 8)
        .dim(ui.canvas_size(FOOTER))
        .middle_of(FOOTER)
        .each_widget(ui, |ui, n, _col, _row, xy, dim| {
            Button::new()
                .color(blue().with_luminance(n as f32 / (24 * 8) as f32))
                .dim(dim)
                .point(xy)
                .react(|| println!("Hey! {:?}", n))
                .set(BUTTON + n, ui);
        });

    ui.draw(g);
}


// Canvas IDs.
const MASTER: CanvasId = 0;
const HEADER: CanvasId = MASTER + 1;
const BODY: CanvasId = HEADER + 1;
const LEFT_COLUMN: CanvasId = BODY + 1;
const MIDDLE_COLUMN: CanvasId = LEFT_COLUMN + 1;
const RIGHT_COLUMN: CanvasId = MIDDLE_COLUMN + 1;
const FOOTER: CanvasId = RIGHT_COLUMN + 1;

// Widget IDs.
const TITLE: UiId = 0;
const SUBTITLE: UiId = TITLE + 1;
const TOP_LEFT: UiId = SUBTITLE + 1;
const MIDDLE: UiId = TOP_LEFT + 1;
const BOTTOM_RIGHT: UiId = MIDDLE + 1;
const BUTTON: UiId = BOTTOM_RIGHT + 1;

