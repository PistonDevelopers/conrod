#[macro_use] extern crate conrod;
extern crate find_folder;

use conrod::backend::piston::{self, Window, WindowEvents, OpenGL};
use conrod::backend::piston::event::UpdateEvent;

widget_ids! {
    struct Ids { canvas, list_select }
}

fn main() {

    const WIDTH: u32 = 600;
    const HEIGHT: u32 = 300;

    // Construct the window.
    let mut window: Window =
        piston::window::WindowSettings::new("ListSelect Demo", [WIDTH, HEIGHT])
            .opengl(OpenGL::V3_2)
            .vsync(true)
            .samples(4)
            .exit_on_esc(true)
            .build()
            .unwrap();

    // Create the event loop.
    let mut events = WindowEvents::new();

    // Construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // A unique identifier for each widget.
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // No text to draw, so we'll just create an empty text texture cache.
    let mut text_texture_cache = piston::window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    // List of entries to display. They should implement the Display trait.
    let list_items = [
        "African Sideneck Turtle".to_string(),
        "Alligator Snapping Turtle".to_string(),
        "Common Snapping Turtle".to_string(),
        "Indian Peacock Softshelled Turtle".to_string(),
        "Eastern River Cooter".to_string(),
        "Eastern Snake Necked Turtle".to_string(),
        "Diamond Terrapin".to_string(),
        "Indian Peacock Softshelled Turtle".to_string(),
        "Musk Turtle".to_string(),
        "Reeves Turtle".to_string(),
        "Eastern Spiny Softshell Turtle".to_string(),
        "Red Ear Slider Turtle".to_string(),
        "Indian Tent Turtle".to_string(),
        "Mud Turtle".to_string(),
        "Painted Turtle".to_string(),
        "Spotted Turtle".to_string()
    ];

    // List of selections, should be same length as list of entries. Will be updated by the widget.
    let mut list_selected = std::collections::HashSet::new();

    // Poll events from the window.
    while let Some(event) = window.next_event(&mut events) {

        // Convert the piston event to a conrod event.
        if let Some(e) = piston::window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            use conrod::{widget, Borderable, Colorable, Labelable, Positionable, Sizeable, Widget};

            // Instantiate the conrod widgets.
            let ui = &mut ui.set_widgets();

            widget::Canvas::new().color(conrod::color::BLUE).set(ids.canvas, ui);

            // Instantiate the `ListSelect` widget.
            let num_items = list_items.len();
            let item_h = 32.0;
            let (mut events, scrollbar) = widget::ListSelect::multiple(num_items, item_h)
                .scrollbar_next_to()
                .w_h(350.0, 220.0)
                .top_left_with_margins_on(ids.canvas, 40.0, 40.0)
                .set(ids.list_select, ui);

            // Handle the `ListSelect`s events.
            while let Some(event) = events.next(ui, |i| list_selected.contains(&i)) {
                use conrod::widget::list_select::Event;
                match event {

                    // For the `Item` events we instantiate the `List`'s items.
                    Event::Item(item) => {
                        let label = &list_items[item.i];
                        let font_size = item_h as conrod::FontSize / 2;
                        let (color, label_color) = match list_selected.contains(&item.i) {
                            true => (conrod::color::LIGHT_BLUE, conrod::color::YELLOW),
                            false => (conrod::color::LIGHT_GREY, conrod::color::BLACK),
                        };
                        let button = widget::Button::new()
                            .border(0.0)
                            .color(color)
                            .label(label)
                            .label_font_size(font_size)
                            .label_color(label_color);
                        item.set(button, ui);
                    },

                    // The selection has changed.
                    Event::Selection(selection) => {
                        selection.update_index_set(&mut list_selected);
                        println!("selected indices: {:?}", list_selected);
                    },

                    // The remaining events indicate interactions with the `ListSelect` widget.
                    event => println!("{:?}", &event),
                }
            }

            // Instantiate the scrollbar for the list.
            if let Some(s) = scrollbar { s.set(ui); }
        });

        window.draw_2d(&event, |c, g| {
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
