#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;


fn main() {
    const WIDTH: u32 = 200;
    const HEIGHT: u32 = 200;

    use conrod::{Labelable, Positionable, Sizeable, Theme, Widget};
    use piston_window::{EventLoop, G2dTexture, OpenGL, PistonWindow, UpdateEvent, WindowSettings};

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;
    
    // Construct the window.
    let mut window: PistonWindow = WindowSettings::new("Click me!", [WIDTH, HEIGHT])
        .opengl(opengl).exit_on_esc(true).build().unwrap();

    // construct our `Ui`.
    let mut ui = conrod::Ui::new(Theme::default());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    {
        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();
    }

    // Create a texture cache in which we can cache text on the GPU.
    let mut text_texture_cache: G2dTexture<'static> = {
        const BUFFER_LEN: usize = WIDTH as usize * HEIGHT as usize;
        const INIT: [u8; BUFFER_LEN] = [128; BUFFER_LEN];
        let factory = &mut window.factory;
        let settings = piston_window::TextureSettings::new();
        G2dTexture::from_memory_alpha(factory, &INIT, WIDTH, HEIGHT, &settings).unwrap()
    };

    let mut count = 0;

    window.set_ups(60);

    // Poll events from the window.
    while let Some(event) = window.next() {
        ui.handle_event(event.clone());

        // `Update` the widgets.
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

        // Draw the `Ui` if it has changed.
        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {

                // Data and functions for rendering the primitives.
                let renderer = conrod::backend::piston::draw::Renderer {
                    context: c,
                    graphics: g,
                    texture_cache: &mut text_texture_cache,
                    // A type used for passing the `texture_cache` used for caching and rendering
                    // `Text` to the function for rendering.
                    cache_queued_glyphs: |graphics: &mut piston_window::G2d,
                                          cache: &mut G2dTexture<'static>,
                                          rect: conrod::text::rt::Rect<u32>,
                                          data: &[u8]| {
                        use piston_window::texture::UpdateTexture;
                        let dim = [rect.width(), rect.height()];
                        let format = piston_window::texture::Format::Rgba8;
                        let encoder = &mut graphics.encoder;
                        UpdateTexture::update(cache, encoder, format, data, dim)
                            .expect("Failed to update texture");
                    },
                    // A function that returns some texture `T` for the given `texture::Id`. We
                    // have no `Image` widgets, so no need to implement this.
                    get_texture: |_id| None,
                };

                conrod::backend::piston::draw::primitives(primitives, renderer);
            }
        });
    }
}
