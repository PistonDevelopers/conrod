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

use conrod::{CanvasId, Floating, Theme, Widget, WidgetId};
use gfx_device_gl::{CommandBuffer, Factory, Resources, Output};
use gfx_graphics::{GfxGraphics, GlyphCache};
use glutin_window::{GlutinWindow, OpenGL};
use graphics::Context;
use piston::window::{WindowSettings, Size};
use piston_window::PistonWindow;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;


type Ui = conrod::Ui<GlyphCache<Resources, Factory>>;
type Graphics<'a> = GfxGraphics<'a, Resources, CommandBuffer, Output>;


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
        let font_path = Path::new("./assets/fonts/NotoSans/NotoSans-Regular.ttf");
        let theme = Theme::default();
        let glyph_cache = GlyphCache::new(&font_path, window.device.borrow().spawn_factory());
        Ui::new(glyph_cache.unwrap(), theme)
    };

    // Poll events from the window.
    for event in window {
        ui.handle_event(&event);
        event.draw_2d(|c, g| draw_ui(&mut ui, c, g));
    }

}


// Draw the Ui.
fn draw_ui<'a>(ui: &mut Ui, c: Context, g: &mut Graphics<'a>) {
    use conrod::color::{blue, light_orange, orange, dark_orange, red, white};
    use conrod::{Button, Colorable, Label, Labelable, Positionable, Sizeable, Split, WidgetMatrix};

    // Construct our Canvas tree.
    Split::new(MASTER).flow_down(&[
        Split::new(HEADER).color(blue()).pad_bottom(20.0),
        Split::new(BODY).length(300.0).flow_right(&[
            Split::new(LEFT_COLUMN).color(light_orange()).pad(20.0),
            Split::new(MIDDLE_COLUMN).color(orange()),
            Split::new(RIGHT_COLUMN).color(dark_orange()).pad(20.0),
        ]),
        Split::new(FOOTER).color(blue())
    ]).set(ui);

    Floating::new()
        .label("Blue")
        .middle_of(LEFT_COLUMN)
        .color(blue())
        .label_color(white())
        .set(FLOATING_A, ui);

    Floating::new()
        .label("Orange")
        .middle_of(RIGHT_COLUMN)
        .color(light_orange())
        .label_color(white())
        .set(FLOATING_B, ui);

    Label::new("Fancy Title").color(light_orange()).font_size(48).middle_of(HEADER).set(TITLE, ui);
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

    WidgetMatrix::new(COLS, ROWS)
        .dim(ui.canvas_size(FOOTER))
        .middle_of(FOOTER)
        .each_widget(ui, |ui, n, _col, _row, xy, dim| {
            Button::new()
                .color(blue().with_luminance(n as f32 / (COLS * ROWS) as f32))
                .dim(dim)
                .point(xy)
                .react(|| println!("Hey! {:?}", n))
                .set(BUTTON + n, ui);
        });
    
    Button::new().color(red()).dimensions(30.0, 30.0).middle_of(FLOATING_A)
        .react(|| println!("Bing!"))
        .set(BING, ui);
    Button::new().color(red()).dimensions(30.0, 30.0).middle_of(FLOATING_B)
        .react(|| println!("Bong!"))
        .set(BONG, ui);

    ui.draw(c, g);
}


// Canvas IDs.
const MASTER: CanvasId = 0;
const HEADER: CanvasId = MASTER + 1;
const BODY: CanvasId = HEADER + 1;
const LEFT_COLUMN: CanvasId = BODY + 1;
const MIDDLE_COLUMN: CanvasId = LEFT_COLUMN + 1;
const RIGHT_COLUMN: CanvasId = MIDDLE_COLUMN + 1;
const FOOTER: CanvasId = RIGHT_COLUMN + 1;
const FLOATING_A: CanvasId = FOOTER + 1;
const FLOATING_B: CanvasId = FLOATING_A + 1;

// Button matrix dimensions.
const ROWS: usize = 5;
const COLS: usize = 24;

// Widget IDs.
const TITLE: WidgetId = 0;
const SUBTITLE: WidgetId = TITLE + 1;
const TOP_LEFT: WidgetId = SUBTITLE + 1;
const MIDDLE: WidgetId = TOP_LEFT + 1;
const BOTTOM_RIGHT: WidgetId = MIDDLE + 1;
const BUTTON: WidgetId = BOTTOM_RIGHT + 1;
const BING: WidgetId = BUTTON + COLS * ROWS;
const BONG: WidgetId = BING + 1;

