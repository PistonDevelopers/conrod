//! A simple demonstration of how to construct and use Canvasses by splitting up the window.

#[macro_use] extern crate conrod;
extern crate find_folder;

use conrod::backend::piston::{window, Window, WindowEvents, OpenGL};
use conrod::backend::piston::event::UpdateEvent;

fn main() {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;
    
    // Construct the window.
    let mut window: Window =
        window::WindowSettings::new("Canvas Demo", [WIDTH, HEIGHT])
            .opengl(opengl).exit_on_esc(true).vsync(true).build().unwrap();

    // Create the event loop.
    let mut events = WindowEvents::new();

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache = window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    // Instantiate the generated list of widget identifiers.
    let ids = &mut Ids::new(ui.widget_id_generator());

    // Poll events from the window.
    while let Some(event) = window.next_event(&mut events) {

        // Convert the piston event to a conrod event.
        if let Some(e) = window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            set_widgets(ui.set_widgets(), ids);
        });

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T { img };
                window::draw(c, g, primitives,
                             &mut text_texture_cache,
                             &image_map,
                             texture_from_image);
            }
        });
    }

}


// Draw the Ui.
fn set_widgets(ref mut ui: conrod::UiCell, ids: &mut Ids) {
    use conrod::{color, widget, Colorable, Labelable, Positionable, Sizeable, Widget};

    // Construct our main `Canvas` tree.
    widget::Canvas::new().flow_down(&[
        (ids.header, widget::Canvas::new().color(color::BLUE).pad_bottom(20.0)),
        (ids.body, widget::Canvas::new().length(300.0).flow_right(&[
            (ids.left_column, widget::Canvas::new().color(color::LIGHT_ORANGE).pad(20.0)),
            (ids.middle_column, widget::Canvas::new().color(color::ORANGE)),
            (ids.right_column, widget::Canvas::new().color(color::DARK_ORANGE).pad(20.0)),
        ])),
        (ids.footer, widget::Canvas::new().color(color::BLUE).scroll_kids_vertically()),
    ]).set(ids.master, ui);

    // A scrollbar for the `FOOTER` canvas.
    widget::Scrollbar::y_axis(ids.footer).auto_hide(true).set(ids.footer_scrollbar, ui);

    // Now we'll make a couple floating `Canvas`ses.
    let floating = widget::Canvas::new().floating(true).w_h(110.0, 150.0).label_color(color::WHITE);
    floating.middle_of(ids.left_column).title_bar("Blue").color(color::BLUE).set(ids.floating_a, ui);
    floating.middle_of(ids.right_column).title_bar("Orange").color(color::LIGHT_ORANGE).set(ids.floating_b, ui);

    // Here we make some canvas `Tabs` in the middle column.
    widget::Tabs::new(&[(ids.tab_foo, "FOO"), (ids.tab_bar, "BAR"), (ids.tab_baz, "BAZ")])
        .wh_of(ids.middle_column)
        .color(color::BLUE)
        .label_color(color::WHITE)
        .middle_of(ids.middle_column)
        .set(ids.tabs, ui);

    widget::Text::new("Fancy Title")
        .color(color::LIGHT_ORANGE)
        .font_size(48)
        .middle_of(ids.header)
        .set(ids.title, ui);
    widget::Text::new("Subtitle")
        .color(color::BLUE.complement())
        .mid_bottom_of(ids.header)
        .set(ids.subtitle, ui);

    widget::Text::new("Top Left")
        .color(color::LIGHT_ORANGE.complement())
        .top_left_of(ids.left_column)
        .set(ids.top_left, ui);

    widget::Text::new("Bottom Right")
        .color(color::DARK_ORANGE.complement())
        .bottom_right_of(ids.right_column)
        .set(ids.bottom_right, ui);

    fn text (text: widget::Text) -> widget::Text { text.color(color::WHITE).font_size(36) }
    text(widget::Text::new("Foo!")).middle_of(ids.tab_foo).set(ids.foo_label, ui);
    text(widget::Text::new("Bar!")).middle_of(ids.tab_bar).set(ids.bar_label, ui);
    text(widget::Text::new("BAZ!")).middle_of(ids.tab_baz).set(ids.baz_label, ui);

    let footer_wh = ui.wh_of(ids.footer).unwrap();
    let mut elements = widget::Matrix::new(COLS, ROWS)
        .w_h(footer_wh[0], footer_wh[1] * 2.0)
        .mid_top_of(ids.footer)
        .set(ids.button_matrix, ui);
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
    for _click in button.clone().middle_of(ids.floating_a).set(ids.bing, ui) {
        println!("Bing!");
    }
    for _click in button.middle_of(ids.floating_b).set(ids.bong, ui) {
        println!("Bong!");
    }
}


// Button matrix dimensions.
const ROWS: usize = 10;
const COLS: usize = 24;

// Generate a unique `WidgetId` for each widget.
widget_ids! {
    struct Ids {
        master,
        header,
        body,
        left_column,
        middle_column,
        right_column,
        footer,
        footer_scrollbar,
        floating_a,
        floating_b,
        tabs,
        tab_foo,
        tab_bar,
        tab_baz,

        title,
        subtitle,
        top_left,
        bottom_right,
        foo_label,
        bar_label,
        baz_label,
        button_matrix,
        bing,
        bong,
    }
}
