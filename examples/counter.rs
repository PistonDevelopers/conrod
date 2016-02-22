#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;


#[cfg(feature = "backend-piston_window")]
fn main() {
    use conrod::{Labelable, Positionable, Sizeable, Theme, Widget};
    use conrod::backend::piston_window::Ui;
    use piston_window::{EventLoop, Glyphs, PistonWindow, UpdateEvent, WindowSettings};

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
        event.update(|_| ui.set_widgets(|ref mut ui| {

            // Generate the ID for the Button COUNTER.
            widget_ids!(CANVAS, COUNTER);

            // Create a background canvas upon which we'll place the button.
            conrod::Canvas::new().pad(40.0).set(CANVAS, ui);

            // Draw the button and increment `count` if pressed.
            conrod::Button::new()
                .middle_of(CANVAS)
                .w_h(80.0, 80.0)
                .label(&count.to_string())
                .react(|| count += 1)
                .set(COUNTER, ui);
        }));
        event.draw_2d(|c, g| ui.draw_if_changed(c, g));
    }

}


#[cfg(not(feature = "backend-piston_window"))]
pub fn main() {
    println!("This example requires the \"backend-piston_window\" feature. Use the feature like so \
              `cargo run --release --features \"backend-piston_window\" --example <example_name>`");
}
