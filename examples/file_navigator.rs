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
        WindowSettings::new("FileNavigator Demo", [600, 300])
            .opengl(OpenGL::V3_2)
            .vsync(true)
            .samples(4)
            .exit_on_esc(true)
            .build()
            .unwrap();

    // Construct our `Ui`.
    let mut ui = {
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        let theme = Theme::default();
        let glyph_cache = piston_window::Glyphs::new(&font_path, window.factory.clone()).unwrap();
        Ui::new(glyph_cache, theme)
    };

    let conrod_directory = find_folder::Search::KidsThenParents(3, 5).for_folder("conrod").unwrap();

    window.set_ups(60);

    // Poll events from the window.
    while let Some(event) = window.next() {
        ui.handle_event(event.clone());
        event.update(|_| {

            // Instantiate the conrod widgets.
            ui.set_widgets(|ref mut ui| {
                use conrod::{Canvas, color, Colorable, FileNavigator, Positionable, Sizeable};

                widget_ids!(CANVAS, FILE_NAVIGATOR);

                Canvas::new().color(color::DARK_CHARCOAL).set(CANVAS, ui);

                // Navigate the conrod directory only showing `.rs` and `.toml` files.
                FileNavigator::with_extension(&conrod_directory, &["rs", "toml"])
                    .color(color::LIGHT_BLUE)
                    .font_size(18)
                    .wh_of(CANVAS)
                    .middle_of(CANVAS)
                    .react(|event| println!("{:?}", &event))
                    .set(FILE_NAVIGATOR, ui);
            });

        });
        window.draw_2d(&event, |c, g| ui.draw_if_changed(c, g));
    }

}
