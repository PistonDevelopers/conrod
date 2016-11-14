#[macro_use] extern crate conrod;
extern crate find_folder;

use conrod::backend::piston::{self, Window, WindowEvents, OpenGL};
use conrod::backend::piston::event::UpdateEvent;


fn main() {
    const WIDTH: u32 = 1080;
    const HEIGHT: u32 = 720;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Construct the window.
    let mut window: Window =
        piston::window::WindowSettings::new("Text Demo", [WIDTH, HEIGHT])
            .opengl(opengl).exit_on_esc(true).build().unwrap();

    // Create the event loop.
    let mut events = WindowEvents::new();

    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // A unique identifier for each widget.
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let noto_sans = assets.join("fonts/NotoSans");
    let regular = ui.fonts.insert_from_file(noto_sans.join("NotoSans-Regular.ttf")).unwrap();
    let italic = ui.fonts.insert_from_file(noto_sans.join("NotoSans-Italic.ttf")).unwrap();
    let bold = ui.fonts.insert_from_file(noto_sans.join("NotoSans-Bold.ttf")).unwrap();

    // Store our `font::Id`s in a list for easy access in the `set_ui` function.
    let font_ids = [regular, italic, bold];

    // Specify the default font to use when none is specified by the widget.
    //
    // By default, the theme's font_id field is `None`. In this case, the first font that is found
    // within the `Ui`'s `font::Map` will be used.
    ui.theme.font_id = Some(regular);

    // Create a texture cache in which we can cache text on the GPU.
    //
    // Note that the dimensions of the `GlyphCache` don't need to be the dimensions of the window,
    // they just need to be at least large enough to cache the maximum amount of text that might be
    // drawn in a single frame.
    let mut text_texture_cache = piston::window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    // Poll events from the window.
    while let Some(event) = window.next_event(&mut events) {

        // Convert the piston event to a conrod event.
        if let Some(e) = piston::window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| set_ui(ui.set_widgets(), &ids, &font_ids));

        window.draw_2d(&event, |c, g| {
            // Only re-draw if there was some change in the `Ui`.
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T { img };
                piston::window::draw(c, g, primitives,
                                     &mut text_texture_cache,
                                     &image_map,
                                     texture_from_image);
            }
        });
    }

}

// Generate a unique const `WidgetId` for each widget.
widget_ids!{
    struct Ids {
        master,
        left_col,
        middle_col,
        right_col,
        left_text,
        middle_text,
        right_text,
    }
}

fn set_ui(ref mut ui: conrod::UiCell, ids: &Ids, font_ids: &[conrod::text::font::Id; 3]) {
    use conrod::{color, widget, Colorable, Positionable, Scalar, Sizeable, Widget};

    // Our `Canvas` tree, upon which we will place our text widgets.
    widget::Canvas::new().flow_right(&[
        (ids.left_col, widget::Canvas::new().color(color::BLACK)),
        (ids.middle_col, widget::Canvas::new().color(color::DARK_CHARCOAL)),
        (ids.right_col, widget::Canvas::new().color(color::CHARCOAL)),
    ]).set(ids.master, ui);

    const DEMO_TEXT: &'static str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
        Mauris aliquet porttitor tellus vel euismod. Integer lobortis volutpat bibendum. Nulla \
        finibus odio nec elit condimentum, rhoncus fermentum purus lacinia. Interdum et malesuada \
        fames ac ante ipsum primis in faucibus. Cras rhoncus nisi nec dolor bibendum pellentesque. \
        Cum sociis natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. \
        Quisque commodo nibh hendrerit nunc sollicitudin sodales. Cras vitae tempus ipsum. Nam \
        magna est, efficitur suscipit dolor eu, consectetur consectetur urna.";

    const PAD: Scalar = 20.0;

    const FONT_REGULAR: usize = 0;
    const FONT_ITALIC: usize = 1;
    const FONT_BOLD: usize = 2;

    widget::Text::new(DEMO_TEXT)
        .font_id(font_ids[FONT_REGULAR])
        .color(color::LIGHT_RED)
        .padded_w_of(ids.left_col, PAD)
        .mid_top_with_margin_on(ids.left_col, PAD)
        .align_text_left()
        .line_spacing(10.0)
        .set(ids.left_text, ui);

    widget::Text::new(DEMO_TEXT)
        .font_id(font_ids[FONT_ITALIC])
        .color(color::LIGHT_GREEN)
        .padded_w_of(ids.middle_col, PAD)
        .middle_of(ids.middle_col)
        .align_text_middle()
        .line_spacing(2.5)
        .set(ids.middle_text, ui);

    widget::Text::new(DEMO_TEXT)
        .font_id(font_ids[FONT_BOLD])
        .color(color::LIGHT_BLUE)
        .padded_w_of(ids.right_col, PAD)
        .mid_bottom_with_margin_on(ids.right_col, PAD)
        .align_text_right()
        .line_spacing(5.0)
        .set(ids.right_text, ui);
}
