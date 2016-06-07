#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;


fn main() {
    use conrod::{Labelable, Positionable, Sizeable, Theme, Widget};
    use piston_window::{EventLoop, OpenGL, PistonWindow, UpdateEvent, WindowSettings};

    // Conrod is backend agnostic. Here, we define the `piston_window` backend to use for our `Ui`.
    type Backend = (piston_window::G2dTexture<'static>, piston_window::Glyphs);
    type Ui = conrod::Ui<Backend>;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;
    
    // Construct the window.
    let mut window: PistonWindow = WindowSettings::new("Click me!", [200, 200])
        .opengl(opengl).exit_on_esc(true).build().unwrap();

    // construct our `Ui`.
    let mut ui = {
        let assets = find_folder::Search::KidsThenParents(3, 5)
            .for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        let theme = Theme::default();
        let glyph_cache = piston_window::Glyphs::new(&font_path, window.factory.clone()).unwrap();
        Ui::new(glyph_cache, theme)
    };

    let mut count = 0;

    window.set_ups(60);

    // Poll events from the window.
    while let Some(event) = window.next() {
        ui.handle_event(event.clone());
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
        window.draw_2d(&event, |c, g| ui.draw_if_changed(c, g));
    }
}
