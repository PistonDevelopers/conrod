//! A simple example demonstrating the `RangeSlider` widget.

#[macro_use] extern crate conrod_core;
extern crate conrod_glium;
#[macro_use] extern crate conrod_winit;
extern crate find_folder;
extern crate glium;

mod support;

use glium::Surface;

widget_ids! {
    struct Ids { canvas, oval, range_slider }
}

fn main() {
    const WIDTH: u32 = 360;
    const HEIGHT: u32 = 360;

    // Build the window.
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_title("RangeSlider Demo")
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

    let mut oval_range = (0.25, 0.75);

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

        set_ui(ui.set_widgets(), &ids, &mut oval_range);

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

// Declare the `WidgetId`s and instantiate the widgets.
fn set_ui(
    ref mut ui: conrod_core::UiCell,
    ids: &Ids,
    oval_range: &mut (conrod_core::Scalar, conrod_core::Scalar),
) {
    use conrod_core::{color, widget, Colorable, Positionable, Sizeable, Widget};

    widget::Canvas::new().color(color::DARK_CHARCOAL).set(ids.canvas, ui);

    const PAD: conrod_core::Scalar = 20.0;
    let (ref mut start, ref mut end) = *oval_range;
    let min = 0.0;
    let max = 1.0;
    for (edge, value) in widget::RangeSlider::new(*start, *end, min, max)
        .color(color::LIGHT_BLUE)
        .padded_w_of(ids.canvas, PAD)
        .h(30.0)
        .mid_top_with_margin_on(ids.canvas, PAD)
        .set(ids.range_slider, ui)
        {
            match edge {
                widget::range_slider::Edge::Start => *start = value,
                widget::range_slider::Edge::End => *end = value,
            }
        }

    let range_slider_w = ui.w_of(ids.range_slider).unwrap();
    let w = (*end - *start) * range_slider_w;
    let h = 200.0;
    widget::Oval::fill([w, h])
        .mid_left_with_margin_on(ids.canvas, PAD + *start * range_slider_w)
        .color(color::LIGHT_BLUE)
        .down(50.0)
        .set(ids.oval, ui);
}
