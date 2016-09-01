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

    // Instantiate the generated list of widget identifiers.
    let ids = &mut Ids::new();

    // Poll events from the window.
    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            set_widgets(ui.set_widgets(), ids);
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
fn set_widgets(ref mut ui: conrod::UiCell, ids: &mut Ids) {
    use conrod::{color, widget, Colorable, Labelable, Positionable, Sizeable, Widget};

    // Construct our main `Canvas` tree.
    let header = ids.header.get(ui);
    let left_column = ids.left_column.get(ui);
    let middle_column = ids.middle_column.get(ui);
    let right_column = ids.right_column.get(ui);
    let footer = ids.footer.get(ui);
    widget::Canvas::new().flow_down(&[
        (header, widget::Canvas::new().color(color::BLUE).pad_bottom(20.0)),
        (ids.body.get(ui), widget::Canvas::new().length(300.0).flow_right(&[
            (left_column, widget::Canvas::new().color(color::LIGHT_ORANGE).pad(20.0)),
            (middle_column, widget::Canvas::new().color(color::ORANGE)),
            (right_column, widget::Canvas::new().color(color::DARK_ORANGE).pad(20.0)),
        ])),
        (footer, widget::Canvas::new().color(color::BLUE).scroll_kids_vertically()),
    ]).set(ids.master.get(ui), ui);

    // A scrollbar for the `FOOTER` canvas.
    widget::Scrollbar::y_axis(footer).auto_hide(true).set(ids.footer_scrollbar.get(ui), ui);

    // Now we'll make a couple floating `Canvas`ses.
    let floating = widget::Canvas::new().floating(true).w_h(110.0, 150.0).label_color(color::WHITE);
    let (floating_a, floating_b) = (ids.floating_a.get(ui), ids.floating_b.get(ui));
    floating.middle_of(left_column).title_bar("Blue").color(color::BLUE).set(floating_a, ui);
    floating.middle_of(right_column).title_bar("Orange").color(color::LIGHT_ORANGE).set(floating_b, ui);

    // Here we make some canvas `Tabs` in the middle column.
    let (tab_foo, tab_bar, tab_baz) = (ids.tab_foo.get(ui), ids.tab_bar.get(ui), ids.tab_baz.get(ui));
    widget::Tabs::new(&[(tab_foo, "FOO"), (tab_bar, "BAR"), (tab_baz, "BAZ")])
        .wh_of(middle_column)
        .color(color::BLUE)
        .label_color(color::WHITE)
        .middle_of(middle_column)
        .set(ids.tabs.get(ui), ui);

    widget::Text::new("Fancy Title")
        .color(color::LIGHT_ORANGE)
        .font_size(48)
        .middle_of(header)
        .set(ids.title.get(ui), ui);
    widget::Text::new("Subtitle")
        .color(color::BLUE.complement())
        .mid_bottom_of(header)
        .set(ids.subtitle.get(ui), ui);

    widget::Text::new("Top Left")
        .color(color::LIGHT_ORANGE.complement())
        .top_left_of(left_column)
        .set(ids.top_left.get(ui), ui);

    widget::Text::new("Bottom Right")
        .color(color::DARK_ORANGE.complement())
        .bottom_right_of(right_column)
        .set(ids.bottom_right.get(ui), ui);

    fn text (text: widget::Text) -> widget::Text { text.color(color::WHITE).font_size(36) }
    text(widget::Text::new("Foo!")).middle_of(tab_foo).set(ids.foo_label.get(ui), ui);
    text(widget::Text::new("Bar!")).middle_of(tab_bar).set(ids.bar_label.get(ui), ui);
    text(widget::Text::new("BAZ!")).middle_of(tab_baz).set(ids.baz_label.get(ui), ui);

    let footer_wh = ui.wh_of(footer).unwrap();
    let mut elements = widget::Matrix::new(COLS, ROWS)
        .w_h(footer_wh[0], footer_wh[1] * 2.0)
        .mid_top_of(footer)
        .set(ids.button_matrix.get(ui), ui);
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
    for _click in button.clone().middle_of(floating_a).set(ids.bing.get(ui), ui) {
        println!("Bing!");
    }
    for _click in button.middle_of(floating_b).set(ids.bong.get(ui), ui) {
        println!("Bong!");
    }
}


// Button matrix dimensions.
const ROWS: usize = 10;
const COLS: usize = 24;

// Generate a unique `WidgetId` for each widget.
widget_ids! {
    Ids {
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
