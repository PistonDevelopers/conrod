#[macro_use] extern crate conrod_core;
extern crate conrod_glium;
#[macro_use] extern crate conrod_winit;
extern crate find_folder;
extern crate glium;

mod support;

use glium::Surface;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 300;

widget_ids! {
    struct Ids { canvas, list_select }
}

fn main() {
    // Build the window.
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title("ListSelect Demo")
        .with_dimensions((WIDTH, HEIGHT).into());
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &events_loop).unwrap();
    let display = support::GliumDisplayWinitWrapper(display);

    // Construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // A unique identifier for each widget.
    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    // A type used for converting `conrod_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod_glium::Renderer::new(&display.0).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();

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
    let mut event_loop = support::EventLoop::new();
    'main: loop {

        // Handle all events.
        for event in event_loop.next(&mut events_loop) {

            // Use the `winit` backend feature to convert the winit event to a conrod one.
            if let Some(event) = support::convert_event(event.clone(), &display) {
                ui.handle_event(event);
                event_loop.needs_update();
            }

            match event {
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    // Break from the loop upon `Escape`.
                    glium::glutin::WindowEvent::CloseRequested |
                    glium::glutin::WindowEvent::KeyboardInput {
                        input: glium::glutin::KeyboardInput {
                            virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } => break 'main,
                    _ => (),
                },
                _ => (),
            }
        }

        // Instantiate the conrod widgets.
        {
            use conrod_core::{widget, Borderable, Colorable, Labelable, Positionable, Sizeable, Widget};

            let ui = &mut ui.set_widgets();

            widget::Canvas::new().color(conrod_core::color::BLUE).set(ids.canvas, ui);

            // Instantiate the `ListSelect` widget.
            let num_items = list_items.len();
            let item_h = 30.0;
            let font_size = item_h as conrod_core::FontSize / 2;
            let (mut events, scrollbar) = widget::ListSelect::multiple(num_items)
                .flow_down()
                .item_size(item_h)
                .scrollbar_next_to()
                .w_h(400.0, 230.0)
                .top_left_with_margins_on(ids.canvas, 40.0, 40.0)
                .set(ids.list_select, ui);

            // Handle the `ListSelect`s events.
            while let Some(event) = events.next(ui, |i| list_selected.contains(&i)) {
                use conrod_core::widget::list_select::Event;
                match event {

                    // For the `Item` events we instantiate the `List`'s items.
                    Event::Item(item) => {
                        let label = &list_items[item.i];
                        let (color, label_color) = match list_selected.contains(&item.i) {
                            true => (conrod_core::color::LIGHT_BLUE, conrod_core::color::YELLOW),
                            false => (conrod_core::color::LIGHT_GREY, conrod_core::color::BLACK),
                        };
                        let button = widget::Button::new()
                            .border(0.0)
                            .color(color)
                            .label(label)
                            .label_font_size(font_size)
                            .label_color(label_color);
                        item.set(button, ui);
                    }

                    // The selection has changed.
                    Event::Selection(selection) => {
                        selection.update_index_set(&mut list_selected);
                        println!("selected indices: {:?}", list_selected);
                    }

                    // The remaining events indicate interactions with the `ListSelect` widget.
                    event => println!("{:?}", &event),
                }
            }

            // Instantiate the scrollbar for the list.
            if let Some(s) = scrollbar { s.set(ui); }
        }

        // Render the `Ui` and then display it on the screen.
        if let Some(primitives) = ui.draw_if_changed() {
            renderer.fill(&display.0, primitives, &image_map);
            let mut target = display.0.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            renderer.draw(&display.0, &mut target, &image_map).unwrap();
            target.finish().unwrap();
        }
    }
}
