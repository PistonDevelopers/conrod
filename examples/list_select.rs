#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use conrod::{Widget};
use piston_window::{EventLoop, OpenGL, PistonWindow, UpdateEvent, WindowSettings};

fn main() {

    const WIDTH: u32 = 600;
    const HEIGHT: u32 = 300;

    // Construct the window.
    let mut window: PistonWindow =
        WindowSettings::new("ListSelect Demo", [WIDTH, HEIGHT])
            .opengl(OpenGL::V3_2)
            .vsync(true)
            .samples(4)
            .exit_on_esc(true)
            .build()
            .unwrap();

    window.set_ups(60);

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new().build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // No text to draw, so we'll just create an empty text texture cache.
    let mut text_texture_cache =
        conrod::backend::piston_window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    // List of entries to display. They should implement the Display trait.
    let list_items = vec!["African Sideneck Turtle".to_string(),
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
                              "Spotted Turtle".to_string()];

    // List of selections, should be same length as list of entries. Will be updated by the widget.
    let mut list_selected = vec![false; list_items.len()];

    // Make a default selection.
    list_selected[3] = true;

    // Poll events from the window.
    while let Some(event) = window.next() {

        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            // Instantiate the conrod widgets.
            ui.set_widgets(|ref mut ui| {
                use conrod::{Canvas, color, Colorable, Positionable, Sizeable, ListSelect};

                widget_ids!(CANVAS, LIST_BOX);

                Canvas::new().color(color::BLUE).set(CANVAS, ui);

                ListSelect::multiple(&list_items, &mut list_selected)
                    .w_h(350.0, 220.0)
                    .top_left_with_margins_on(CANVAS, 40.0, 40.0)
                    .color(color::LIGHT_GREY)
                    .selected_color(color::LIGHT_BLUE)
                    .text_color(color::BLACK)
                    .selected_text_color(color::YELLOW)
                    .font_size(16)
                    .scrollbar_auto_hide(false)
                    .react(|event| {/*
                        match event {
                            Event::SelectEntry(e) 		=> { println!("Select Entry: {:?}", &e); },
                            Event::SelectEntries(vec) 	=> { println!("Select Entries: {:?}", &vec); },
                            Event::DoubleClick(vec) 	=> { println!("Double Click: {:?}", &vec); },
                            Event::KeyPress(vec, kp) 	=> { println!("Keypress: {:?}", &vec); },
                        }*/
                        println!("Got event: {:?}", event);
                    })
                    .set(LIST_BOX, ui);
                });
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
