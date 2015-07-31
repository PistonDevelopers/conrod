//! 
//! A simple demonstration of how to construct and use Canvasses by splitting up the window.
//!

extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use conrod::{Canvas, Theme, Widget, WidgetId};
use piston_window::*;

type Ui = conrod::Ui<Glyphs>;

fn main() {

    // Construct the window.
    let window: PistonWindow =
        WindowSettings::new("Canvas Demo", [800, 600])
            .exit_on_esc(true).into();

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
    for event in window {
        ui.handle_event(&event);
        event.draw_2d(|c, g| draw_ui(&mut ui, c, g));
    }

}


// Draw the Ui.
fn draw_ui(ui: &mut Ui, c: Context, g: &mut G2d) {
    use conrod::color::{blue, light_orange, orange, dark_orange, red, white};
    use conrod::{Button, Colorable, Label, Labelable, Positionable, Sizeable, Split, Tabs,
                 WidgetMatrix};

    // Construct our Canvas tree.
    Split::new(MASTER).flow_down(&[
        Split::new(HEADER).color(blue()).pad_bottom(20.0),
        Split::new(BODY).length(300.0).flow_right(&[
            Split::new(LEFT_COLUMN).color(light_orange()).pad(20.0),
            Split::new(MIDDLE_COLUMN).color(orange()),
            Split::new(RIGHT_COLUMN).color(dark_orange()).pad(20.0),
        ]),
        Split::new(FOOTER).color(blue())
    ]).set(ui);

    Canvas::new()
        .show_title_bar(true)
        .floating(true)
        .label("Blue")
        .middle_of(LEFT_COLUMN)
        .color(blue())
        .label_color(white())
        .set(FLOATING_A, ui);

    Canvas::new()
        .show_title_bar(true)
        .floating(true)
        .label("Orange")
        .middle_of(RIGHT_COLUMN)
        .color(light_orange())
        .label_color(white())
        .set(FLOATING_B, ui);

    Tabs::new(&[(TAB_FOO, "FOO"),
                (TAB_BAR, "BAR"),
                (TAB_BAZ, "BAZ")])
        .dim(ui.widget_size(MIDDLE_COLUMN))
        .color(blue())
        .label_color(white())
        .middle_of(MIDDLE_COLUMN)
        .set(TABS, ui);

    Label::new("Fancy Title").color(light_orange()).font_size(48).middle_of(HEADER).set(TITLE, ui);
    Label::new("Subtitle").color(blue().complement()).mid_bottom_of(HEADER).set(SUBTITLE, ui);

    Label::new("Top Left")
        .color(light_orange().complement())
        .top_left_of(LEFT_COLUMN)
        .set(TOP_LEFT, ui);

    Label::new("Middle")
        .color(orange().complement())
        .middle_of(MIDDLE_COLUMN)
        .set(MIDDLE, ui);

    Label::new("Bottom Right")
        .color(dark_orange().complement())
        .bottom_right_of(RIGHT_COLUMN)
        .set(BOTTOM_RIGHT, ui);

    Label::new("Foo!").color(white()).font_size(36).middle_of(TAB_FOO).set(FOO_LABEL, ui);
    Label::new("Bar!").color(white()).font_size(36).middle_of(TAB_BAR).set(BAR_LABEL, ui);
    Label::new("BAZ!").color(white()).font_size(36).middle_of(TAB_BAZ).set(BAZ_LABEL, ui);

    WidgetMatrix::new(COLS, ROWS)
        .dim(ui.widget_size(FOOTER))
        .middle_of(FOOTER)
        .each_widget(ui, |ui, n, _col, _row, xy, dim| {
            Button::new()
                .color(blue().with_luminance(n as f32 / (COLS * ROWS) as f32))
                .dim(dim)
                .point(xy)
                .react(|| println!("Hey! {:?}", n))
                .set(BUTTON + n, ui);
        });
    
    Button::new().color(red()).dimensions(30.0, 30.0).middle_of(FLOATING_A)
        .react(|| println!("Bing!"))
        .set(BING, ui);
    Button::new().color(red()).dimensions(30.0, 30.0).middle_of(FLOATING_B)
        .react(|| println!("Bong!"))
        .set(BONG, ui);

    ui.draw(c, g);
}


// Button matrix dimensions.
const ROWS: usize = 5;
const COLS: usize = 24;

// Canvas IDs.
const MASTER: WidgetId = 0;
const HEADER: WidgetId = MASTER + 1;
const BODY: WidgetId = HEADER + 1;
const LEFT_COLUMN: WidgetId = BODY + 1;
const MIDDLE_COLUMN: WidgetId = LEFT_COLUMN + 1;
const RIGHT_COLUMN: WidgetId = MIDDLE_COLUMN + 1;
const FOOTER: WidgetId = RIGHT_COLUMN + 1;
const FLOATING_A: WidgetId = FOOTER + 1;
const FLOATING_B: WidgetId = FLOATING_A + 1;
const TABS: WidgetId = FLOATING_B + 1;
const TAB_FOO: WidgetId = TABS + 1;
const TAB_BAR: WidgetId = TAB_FOO + 1;
const TAB_BAZ: WidgetId = TAB_BAR + 1;

// Widget IDs.
const TITLE: WidgetId = TAB_BAZ + 1;
const SUBTITLE: WidgetId = TITLE + 1;
const TOP_LEFT: WidgetId = SUBTITLE + 1;
const MIDDLE: WidgetId = TOP_LEFT + 1;
const BOTTOM_RIGHT: WidgetId = MIDDLE + 1;
const FOO_LABEL: WidgetId = BOTTOM_RIGHT + 1;
const BAR_LABEL: WidgetId = FOO_LABEL + 1;
const BAZ_LABEL: WidgetId = BAR_LABEL + 1;
const BUTTON: WidgetId = BAZ_LABEL + 1;
const BING: WidgetId = BUTTON + COLS * ROWS;
const BONG: WidgetId = BING + 1;

