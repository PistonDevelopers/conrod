#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use conrod::{Theme, Widget};
use piston_window::{EventLoop, G2dTexture, OpenGL, PistonWindow, UpdateEvent, WindowSettings};


fn main() {
    const WIDTH: u32 = 1080;
    const HEIGHT: u32 = 720;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("Text Demo", [WIDTH, HEIGHT])
            .opengl(opengl).exit_on_esc(true).build().unwrap();

    // Construct our `Ui`.
    let mut ui = conrod::Ui::new(Theme::default());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets").unwrap();
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

    window.set_ups(60);

    // Poll events from the window.
    while let Some(event) = window.next() {
        ui.handle_event(event.clone());
        event.update(|_| ui.set_widgets(set_ui));
        window.draw_2d(&event, |c, g| {

            // Only re-draw if there was some change in the `Ui`.
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

                        println!("{:?}", rect);
                        println!("data: LEN={:?}, BYTES={:?}", data.len(), data);
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

                conrod::backend::draw_piston::primitives(primitives, renderer);
            }
        });
    }

}

fn set_ui(ref mut ui: conrod::UiCell) {
    use conrod::{Canvas, color, Colorable, Positionable, Scalar, Sizeable, Text};

    // Generate a unique const `WidgetId` for each widget.
    widget_ids!{
        MASTER,
        LEFT_COL,
        MIDDLE_COL,
        RIGHT_COL,
        LEFT_TEXT,
        MIDDLE_TEXT,
        RIGHT_TEXT,
    }

    // Our `Canvas` tree, upon which we will place our text widgets.
    Canvas::new().flow_right(&[
        (LEFT_COL, Canvas::new().color(color::BLACK)),
        (MIDDLE_COL, Canvas::new().color(color::DARK_CHARCOAL)),
        (RIGHT_COL, Canvas::new().color(color::CHARCOAL)),
    ]).set(MASTER, ui);

    const DEMO_TEXT: &'static str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
        Mauris aliquet porttitor tellus vel euismod. Integer lobortis volutpat bibendum. Nulla \
        finibus odio nec elit condimentum, rhoncus fermentum purus lacinia. Interdum et malesuada \
        fames ac ante ipsum primis in faucibus. Cras rhoncus nisi nec dolor bibendum pellentesque. \
        Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. \
        Quisque commodo nibh hendrerit nunc sollicitudin sodales. Cras vitae tempus ipsum. Nam \
        magna est, efficitur suscipit dolor eu, consectetur consectetur urna.";

    const PAD: Scalar = 20.0;

    Text::new(DEMO_TEXT)
        .color(color::LIGHT_RED)
        .padded_w_of(LEFT_COL, PAD)
        .mid_top_with_margin_on(LEFT_COL, PAD)
        .align_text_left()
        .line_spacing(10.0)
        .set(LEFT_TEXT, ui);

    Text::new(DEMO_TEXT)
        .color(color::LIGHT_GREEN)
        .padded_w_of(MIDDLE_COL, PAD)
        .middle_of(MIDDLE_COL)
        .align_text_middle()
        .line_spacing(2.5)
        .set(MIDDLE_TEXT, ui);

    Text::new(DEMO_TEXT)
        .color(color::LIGHT_BLUE)
        .padded_w_of(RIGHT_COL, PAD)
        .mid_bottom_with_margin_on(RIGHT_COL, PAD)
        .align_text_right()
        .line_spacing(5.0)
        .set(RIGHT_TEXT, ui);
}
