//! A simple demonstration of how to construct and use Canvasses by splitting up the window.

#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use piston_window::{EventLoop, OpenGL, PistonWindow, UpdateEvent, WindowSettings};

fn main() {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;
    
    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("Canvas Demo", [WIDTH, HEIGHT])
            .opengl(opengl).exit_on_esc(true).vsync(true).build().unwrap();
    window.set_ups(60);

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new().build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache =
        conrod::backend::piston_window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    // Poll events from the window.
    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            set_widgets(ui.set_widgets());
        });

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T { img };
                conrod::backend::piston_window::draw(c, g, primitives,
                                                     &mut text_texture_cache,
                                                     &image_map,
                                                     texture_from_image);
            }
        });
    }

}


// Draw the Ui.
fn set_widgets(ref mut ui: conrod::UiCell) {
    use conrod::{color, widget, Colorable, Labelable, Positionable, Sizeable, Widget};

    // Construct our main `Canvas` tree.
    widget::Canvas::new().flow_down(&[
        (HEADER, widget::Canvas::new().color(color::BLUE).pad_bottom(20.0)),
        (BODY, widget::Canvas::new().length(300.0).flow_right(&[
            (LEFT_COLUMN, widget::Canvas::new().color(color::LIGHT_ORANGE).pad(20.0)),
            (MIDDLE_COLUMN, widget::Canvas::new().color(color::ORANGE)),
            (RIGHT_COLUMN, widget::Canvas::new().color(color::DARK_ORANGE).pad(20.0)),
        ])),
        (FOOTER, widget::Canvas::new().color(color::BLUE).scroll_kids_vertically()),
    ]).set(MASTER, ui);

    // A scrollbar for the `FOOTER` canvas.
    widget::Scrollbar::y_axis(FOOTER).auto_hide(true).set(FOOTER_SCROLLBAR, ui);

    // Now we'll make a couple floating `Canvas`ses.
    let floating = widget::Canvas::new().floating(true).w_h(110.0, 150.0).label_color(color::WHITE);
    floating.middle_of(LEFT_COLUMN).title_bar("Blue").color(color::BLUE).set(FLOATING_A, ui);
    floating.middle_of(RIGHT_COLUMN).title_bar("Orange").color(color::LIGHT_ORANGE).set(FLOATING_B, ui);

    // Here we make some canvas `Tabs` in the middle column.
    widget::Tabs::new(&[(TAB_FOO, "FOO"), (TAB_BAR, "BAR"), (TAB_BAZ, "BAZ")])
        .wh_of(MIDDLE_COLUMN)
        .color(color::BLUE)
        .label_color(color::WHITE)
        .middle_of(MIDDLE_COLUMN)
        .set(TABS, ui);

    widget::Text::new("Fancy Title").color(color::LIGHT_ORANGE).font_size(48).middle_of(HEADER).set(TITLE, ui);
    widget::Text::new("Subtitle").color(color::BLUE.complement()).mid_bottom_of(HEADER).set(SUBTITLE, ui);

    widget::Text::new("Top Left")
        .color(color::LIGHT_ORANGE.complement())
        .top_left_of(LEFT_COLUMN)
        .set(TOP_LEFT, ui);

    widget::Text::new("Bottom Right")
        .color(color::DARK_ORANGE.complement())
        .bottom_right_of(RIGHT_COLUMN)
        .set(BOTTOM_RIGHT, ui);

    widget::Text::new("Foo!").color(color::WHITE).font_size(36).middle_of(TAB_FOO).set(FOO_LABEL, ui);
    widget::Text::new("Bar!").color(color::WHITE).font_size(36).middle_of(TAB_BAR).set(BAR_LABEL, ui);
    widget::Text::new("BAZ!").color(color::WHITE).font_size(36).middle_of(TAB_BAZ).set(BAZ_LABEL, ui);

    let footer_wh = ui.wh_of(FOOTER).unwrap();
    let mut elements = widget::Matrix::new(COLS, ROWS)
        .w_h(footer_wh[0], footer_wh[1] * 2.0)
        .mid_top_of(FOOTER)
        .set(BUTTON_MATRIX, ui);
    while let Some(elem) = elements.next(ui) {
        let (r, c) = (elem.row, elem.col);
        let n = c + r * c;
        let luminance = n as f32 / (COLS * ROWS) as f32;
        let button = widget::Button::new().color(color::BLUE.with_luminance(luminance));
        for _click in elem.set(button, ui) {
            println!("Hey! {:?}", (r, c));
        }
    }

    let button = widget::Button::new().color(color::RED).w_h(30.0, 30.0);
    for _click in button.clone().middle_of(FLOATING_A).set(BING, ui) {
        println!("Bing!");
    }
    for _click in button.middle_of(FLOATING_B).set(BONG, ui) {
        println!("Bong!");
    }
}


// Button matrix dimensions.
const ROWS: usize = 10;
const COLS: usize = 24;


// Generate a unique `WidgetId` for each widget.
widget_ids! {

    // Canvas IDs.
    MASTER,
    HEADER,
    BODY,
    LEFT_COLUMN,
    MIDDLE_COLUMN,
    RIGHT_COLUMN,
    FOOTER,
    FOOTER_SCROLLBAR,
    FLOATING_A,
    FLOATING_B,
    TABS,
    TAB_FOO,
    TAB_BAR,
    TAB_BAZ,

    // Widget IDs.
    TITLE,
    SUBTITLE,
    TOP_LEFT,
    BOTTOM_RIGHT,
    FOO_LABEL,
    BAR_LABEL,
    BAZ_LABEL,
    BUTTON_MATRIX,
    BING,
    BONG,

}
