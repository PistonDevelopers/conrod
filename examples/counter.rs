#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use conrod::{Labelable, Positionable, Sizeable, Theme, Ui, Widget};
use piston_window::{EventLoop, Glyphs, PistonWindow, UpdateEvent, WindowSettings};

fn main() {

    // Construct the window.
    let window: PistonWindow = WindowSettings::new("Click me!", [200, 200])
        .exit_on_esc(true).build().unwrap();

    // construct our `Ui`.
    let mut ui = {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        let theme = Theme::default();
        let glyph_cache = Glyphs::new(&font_path, window.factory.borrow().clone());
        Ui::new(glyph_cache.unwrap(), theme)
    };

    let mut count = 0;

    // Poll events from the window.
    for event in window.ups(60) {
        ui.handle_event(&event);
        event.update(|_| ui.set_widgets(|ui| {

            // Generate the ID for the Button COUNTER.
            widget_ids!(CANVAS, COUNTER);

            // Create a background canvas upon which we'll place the button.
            conrod::Split::new(CANVAS).pad(40.0).set(ui);

            // Draw the button and increment `count` if pressed.
            conrod::Button::new()
                .middle_of(CANVAS)
                .dimensions(80.0, 80.0)
                .label(&count.to_string())
                .react(|| count += 1)
                .set(COUNTER, ui);
        }));
        event.draw_2d(|c, g| ui.draw_if_changed(c, g));
    }

}
