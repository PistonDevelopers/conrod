#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use piston_window::{EventLoop, G2dTexture, OpenGL, PistonWindow, UpdateEvent, Window};


fn main() {
    const WIDTH: u32 = 360;
    const HEIGHT: u32 = 720;

    // Construct the window.
    let mut window: PistonWindow =
        piston_window::WindowSettings::new("Text Demo", [WIDTH, HEIGHT])
            .opengl(OpenGL::V3_2).exit_on_esc(true).build().unwrap();

    // construct our `Ui`.
    let mut ui = conrod::Ui::new(conrod::Theme::default());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Create a texture cache in which we can cache text on the GPU.
    let mut text_texture_cache: G2dTexture<'static> = {
        const BUFFER_LEN: usize = WIDTH as usize * HEIGHT as usize;
        const INIT: [u8; BUFFER_LEN] = [128; BUFFER_LEN];
        let factory = &mut window.factory;
        let settings = piston_window::TextureSettings::new();
        G2dTexture::from_memory_alpha(factory, &INIT, WIDTH, HEIGHT, &settings).unwrap()
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

        // Convert the piston event to a conrod event.
        let size = window.size();
        let (w, h) = (size.width as conrod::Scalar, size.height as conrod::Scalar);
        if let Some(e) = conrod::backend::event_piston::convert_event(event.clone(), w, h) {
            ui.handle_event(e);
        }

        event.update(|_| ui.set_widgets(|ui_cell| set_ui(ui_cell, &mut demo_text)));

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {

                // Data and functions for rendering the primitives.
                let renderer = conrod::backend::draw_piston::Renderer {
                    context: c,
                    graphics: g,
                    texture_cache: &mut text_texture_cache,
                    // A type used for passing the `texture_cache` used for caching and rendering
                    // `Text` to the function for rendering.
                    cache_queued_glyphs: |graphics: &mut piston_window::G2d,
                                          cache: &mut G2dTexture<'static>,
                                          rect: conrod::text::RtRect<u32>,
                                          data: &[u8]| {
                        use piston_window::texture::UpdateTexture;
                        let dim = [rect.width(), rect.height()];
                        let format = piston_window::texture::Format::Rgba8;
                        let encoder = &mut graphics.encoder;
                        UpdateTexture::update(cache, encoder, format, data, dim)
                            .expect("Failed to update texture");
                    },
                    // We're drawing no images, so just return `None`.
                    get_texture: |_id| None,
                };

                conrod::backend::draw_piston::primitives(primitives, renderer);
            }
        });
    }

}

// Declare the `WidgetId`s and instantiate the widgets.
fn set_ui(ref mut ui: conrod::UiCell, demo_text: &mut String) {
    use conrod::{Canvas, color, Colorable, Positionable, Sizeable, TextEdit, Widget};

    widget_ids!{CANVAS, TEXT_EDIT};

    Canvas::new().color(color::DARK_CHARCOAL).set(CANVAS, ui);

    TextEdit::new(demo_text)
        .color(color::LIGHT_BLUE)
        .padded_wh_of(CANVAS, 20.0)
        .mid_top_of(CANVAS)
        .align_text_x_middle()
        .line_spacing(2.5)
        .react(|_: &mut String| {})
        .set(TEXT_EDIT, ui);
}
