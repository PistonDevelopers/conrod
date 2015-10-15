#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use conrod::{Colorable, Labelable, Sizeable, Theme, Ui, Widget};
use piston_window::*;

fn main() {

    // Construct the window.
    let window: PistonWindow = WindowSettings::new("Click me!", [200, 100])
        .exit_on_esc(true).build().unwrap();

    // construct our `Ui`.
    let mut ui = {
        let assets = find_folder::Search::ParentsThenKids(3, 3)
            .for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        let theme = Theme::default();
        let glyph_cache = Glyphs::new(&font_path, window.factory.borrow().clone());
        Ui::new(glyph_cache.unwrap(), theme)
    };

    let mut count = 0;

    // Poll events from the window.
    for event in window {
        ui.handle_event(&event);
        event.draw_2d(|c, g| {

            // Set the background color to use for clearing the screen.
            conrod::Background::new().rgb(0.2, 0.25, 0.4).set(&mut ui);

            // Generate the ID for the Button COUNTER.
            widget_ids!(COUNTER);

            // Draw the button and increment `count` if pressed.
            conrod::Button::new()
                .color(conrod::color::red())
                .dimensions(80.0, 80.0)
                .label(&count.to_string())
                .react(|| count += 1)
                .set(COUNTER, &mut ui);

            // Draw our Ui!
            ui.draw_if_changed(c, g);
        });
    }

}
