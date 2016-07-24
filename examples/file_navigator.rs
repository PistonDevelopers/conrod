#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use piston_window::{EventLoop, G2dTexture, PistonWindow, UpdateEvent, WindowSettings};


fn main() {
    const WIDTH: u32 = 600;
    const HEIGHT: u32 = 300;

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("FileNavigator Demo", [WIDTH, HEIGHT])
            .opengl(piston_window::OpenGL::V3_2)
            .vsync(true)
            .samples(4)
            .exit_on_esc(true)
            .build()
            .unwrap();

    // Construct our `Ui`.
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

    let conrod_directory = find_folder::Search::KidsThenParents(3, 5).for_folder("conrod").unwrap();

    window.set_ups(60);

    // Poll events from the window.
    while let Some(event) = window.next() {
        ui.handle_event(event.clone());

        event.update(|_| {

            // Instantiate the conrod widgets.
            ui.set_widgets(|ref mut ui| {
                use conrod::{Canvas, Colorable, FileNavigator, Positionable, Sizeable, Widget};

                widget_ids!(CANVAS, FILE_NAVIGATOR);

                Canvas::new().color(conrod::color::DARK_CHARCOAL).set(CANVAS, ui);

                // Navigate the conrod directory only showing `.rs` and `.toml` files.
                FileNavigator::with_extension(&conrod_directory, &["rs", "toml"])
                    .color(conrod::color::LIGHT_BLUE)
                    .font_size(16)
                    .wh_of(CANVAS)
                    .middle_of(CANVAS)
                    .react(|event| println!("{:?}", &event))
                    .set(FILE_NAVIGATOR, ui);
            });

        });

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {

                // A function used for caching glyphs from `Text` widgets.
                fn cache_queued_glyphs(graphics: &mut piston_window::G2d,
                                       cache: &mut G2dTexture<'static>,
                                       rect: conrod::text::rt::Rect<u32>,
                                       data: &[u8])
                {
                    use piston_window::texture::UpdateTexture;
                    let dim = [rect.width(), rect.height()];
                    let format = piston_window::texture::Format::Rgba8;
                    let encoder = &mut graphics.encoder;
                    UpdateTexture::update(cache, encoder, format, data, dim)
                        .expect("Failed to update texture");
                }

                // Data and functions for rendering the primitives.
                let renderer = conrod::backend::piston::draw::Renderer {
                    context: c,
                    graphics: g,
                    texture_cache: &mut text_texture_cache,
                    cache_queued_glyphs: cache_queued_glyphs,
                    // A function that returns some texture `T` for the given `texture::Id`. We
                    // have no `Image` widgets, so no need to implement this.
                    get_texture: |_id| None,
                };

                conrod::backend::piston::draw::primitives(primitives, renderer);
            }
        });
    }

}
