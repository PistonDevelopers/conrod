//!
//! A simple demonstration of how to construct and use Canvasses by splitting up the window.
//!


#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use conrod::{Canvas, Theme, Widget, color};
use piston_window::{EventLoop, Glyphs, PistonWindow, UpdateEvent, WindowSettings};

type Ui = conrod::Ui<Glyphs>;


fn main() {

    // Construct the window.
    let window: PistonWindow =
        WindowSettings::new("Canvas Demo", [800, 600])
            .exit_on_esc(true).vsync(true).build().unwrap();

    // construct our `Ui`.
    let mut ui = {
        let assets = find_folder::Search::ParentsThenKids(3, 3)
            .for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        let theme = Theme::default();
        let glyph_cache = Glyphs::new(&font_path, window.factory.borrow().clone());
        Ui::new(glyph_cache.unwrap(), theme)
    };

    // Poll events from the window.
    for event in window.ups(60) {
        ui.handle_event(&event);
        event.update(|_| ui.set_widgets(set_widgets));
        event.draw_2d(|c, g| ui.draw_if_changed(c, g));
    }

}


// Draw the Ui.
fn set_widgets(ui: &mut Ui) {
    use conrod::{Button, Colorable, Labelable, Positionable, Sizeable, Tabs, Text, WidgetMatrix};

    // Construct our main `Canvas` tree.
    Canvas::new().flow_down(&[
        (HEADER, Canvas::new().color(color::BLUE).pad_bottom(20.0)),
        (BODY, Canvas::new().length(300.0).flow_right(&[
            (LEFT_COLUMN, Canvas::new().color(color::LIGHT_ORANGE).pad(20.0)),
            (MIDDLE_COLUMN, Canvas::new().color(color::ORANGE)),
            (RIGHT_COLUMN, Canvas::new().color(color::DARK_ORANGE).pad(20.0)),
        ])),
        (FOOTER, Canvas::new().color(color::BLUE).vertical_scrolling(true)),
    ]).set(MASTER, ui);

    // Now we'll make a couple floating `Canvas`ses.
    let floating = Canvas::new().floating(true).w_h(110.0, 150.0).label_color(color::WHITE);
    floating.middle_of(LEFT_COLUMN).title_bar("Blue").color(color::BLUE).set(FLOATING_A, ui);
    floating.middle_of(RIGHT_COLUMN).title_bar("Orange").color(color::LIGHT_ORANGE).set(FLOATING_B, ui);

    // Here we make some canvas `Tabs` in the middle column.
    Tabs::new(&[(TAB_FOO, "FOO"), (TAB_BAR, "BAR"), (TAB_BAZ, "BAZ")])
        .wh_of(MIDDLE_COLUMN)
        .color(color::BLUE)
        .label_color(color::WHITE)
        .middle_of(MIDDLE_COLUMN)
        .set(TABS, ui);

    Text::new("Fancy Title").color(color::LIGHT_ORANGE).font_size(48).middle_of(HEADER).set(TITLE, ui);
    Text::new("Subtitle").color(color::BLUE.complement()).mid_bottom_of(HEADER).set(SUBTITLE, ui);

    Text::new("Top Left")
        .color(color::LIGHT_ORANGE.complement())
        .top_left_of(LEFT_COLUMN)
        .set(TOP_LEFT, ui);

    Text::new("Bottom Right")
        .color(color::DARK_ORANGE.complement())
        .bottom_right_of(RIGHT_COLUMN)
        .set(BOTTOM_RIGHT, ui);

    Text::new("Foo!").color(color::WHITE).font_size(36).middle_of(TAB_FOO).set(FOO_LABEL, ui);
    Text::new("Bar!").color(color::WHITE).font_size(36).middle_of(TAB_BAR).set(BAR_LABEL, ui);
    Text::new("BAZ!").color(color::WHITE).font_size(36).middle_of(TAB_BAZ).set(BAZ_LABEL, ui);

    let footer_wh = ui.wh_of(FOOTER).unwrap();
    WidgetMatrix::new(COLS, ROWS)
        .w_h(footer_wh[0], footer_wh[1] * 2.0)
        .mid_top_of(FOOTER)
        .each_widget(|n, _col, _row| {
            Button::new()
                .color(color::BLUE.with_luminance(n as f32 / (COLS * ROWS) as f32))
                .react(move || println!("Hey! {:?}", n))
        })
        .set(BUTTON_MATRIX, ui);

    Button::new().color(color::RED).w_h(30.0, 30.0).middle_of(FLOATING_A)
        .react(|| println!("Bing!"))
        .set(BING, ui);
    Button::new().color(color::RED).w_h(30.0, 30.0).middle_of(FLOATING_B)
        .react(|| println!("Bong!"))
        .set(BONG, ui);
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
