//! A simple example demonstrating the `List` widget.

#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use conrod::{Theme, Widget};
use piston_window::{EventLoop, OpenGL, PistonWindow, UpdateEvent, WindowSettings};


/// Conrod is backend agnostic. Here, we define the `piston_window` backend to use for our `Ui`.
type Backend = (piston_window::G2dTexture<'static>, piston_window::Glyphs);
type Ui = conrod::Ui<Backend>;
type UiCell<'a> = conrod::UiCell<'a, Backend>;


fn main() {

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("List Demo", [100, 600])
            .opengl(OpenGL::V3_2).exit_on_esc(true).samples(4).vsync(true).build().unwrap();

    // Construct our `Ui`.
    let mut ui = {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        let theme = Theme::default();
        let glyph_cache = piston_window::Glyphs::new(&font_path, window.factory.clone()).unwrap();
        Ui::new(glyph_cache, theme)
    };

    window.set_ups(60);

    let mut list = vec![true; 8];

    // Poll events from the window.
    while let Some(event) = window.next() {
        ui.handle_event(event.clone());
        event.update(|_| ui.set_widgets(|ui_cell| set_ui(ui_cell, &mut list)));
        window.draw_2d(&event, |c, g| ui.draw_if_changed(c, g));
    }

}

// Declare the `WidgetId`s and instantiate the widgets.
fn set_ui(ref mut ui: UiCell, list: &mut [bool]) {
    use conrod::{Canvas, color, Colorable, List, Oval, Positionable, Sizeable};

    widget_ids!{CANVAS, LIST};

    Canvas::new().color(color::DARK_CHARCOAL).set(CANVAS, ui);

    const ITEM_HEIGHT: conrod::Scalar = 25.0;

    List::new(list.len(), ITEM_HEIGHT)
        .color(color::LIGHT_BLUE)
        .scroll_kids_vertically()
        .item(|item| {

            item.layout(Toggle::new(list[item.list_idx]))
                .set(item.widget_idx, &mut ui);

        })
        .set(LIST, ui);
}
