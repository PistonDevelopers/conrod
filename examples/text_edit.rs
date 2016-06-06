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
        WindowSettings::new("Text Demo", [360, 720])
            .opengl(OpenGL::V3_2).exit_on_esc(true).build().unwrap();

    // Construct our `Ui`.
    let mut ui = {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        let theme = Theme::default();
        let glyph_cache = piston_window::Glyphs::new(&font_path, window.factory.clone()).unwrap();
        Ui::new(glyph_cache, theme)
    };

    // Some starting text to edit.
    let mut demo_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
        Mauris aliquet porttitor tellus vel euismod. Integer lobortis volutpat bibendum. Nulla \
        finibus odio nec elit condimentum, rhoncus fermentum purus lacinia. Interdum et malesuada \
        fames ac ante ipsum primis in faucibus. Cras rhoncus nisi nec dolor bibendum pellentesque. \
        Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. \
        Quisque commodo nibh hendrerit nunc sollicitudin sodales. Cras vitae tempus ipsum. Nam \
        magna est, efficitur suscipit dolor eu, consectetur consectetur urna.".to_owned();

    window.set_ups(60);

    // Poll events from the window.
    while let Some(event) = window.next() {
        ui.handle_event(event.clone());
        event.update(|_| ui.set_widgets(|ui_cell| set_ui(ui_cell, &mut demo_text)));
        window.draw_2d(&event, |c, g| ui.draw(c, g));
    }

}

// Declare the `WidgetId`s and instantiate the widgets.
fn set_ui(ref mut ui: UiCell, demo_text: &mut String) {
    use conrod::{Canvas, color, Colorable, Positionable, Sizeable, TextEdit};

    widget_ids!{CANVAS, TEXT_EDIT};

    Canvas::new().color(color::DARK_CHARCOAL).set(CANVAS, ui);

    TextEdit::new(demo_text)
        .color(color::LIGHT_BLUE)
        .padded_w_of(CANVAS, 20.0)
        .middle_of(CANVAS)
        .align_text_middle()
        .line_spacing(2.5)
        .react(|_: &mut String| {})
        .set(TEXT_EDIT, ui);
}
